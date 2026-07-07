from pathlib import Path
from typing import Optional, Any

from .base import BaseConverter, ConversionResult
from ..exceptions import MissingDependencyError


class XlsxConverter(BaseConverter):
    """Convert XLSX/XLS spreadsheets to Markdown tables using pandas."""

    EXTENSIONS = {".xlsx", ".xls"}
    MIME_TYPES = {
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        "application/vnd.ms-excel",
    }

    @classmethod
    def accepts(cls, mime_type: str, extension: Optional[str] = None) -> bool:
        if mime_type in cls.MIME_TYPES:
            return True
        if extension and extension.lower() in cls.EXTENSIONS:
            return True
        return False

    def _df_to_markdown(self, df: "Any") -> str:
        """Convert a pandas DataFrame to a GitHub-flavoured Markdown table."""
        import pandas as pd

        if df.empty:
            return "*Empty sheet*"

        headers = [str(col) if col is not None else "" for col in df.columns]
        rows: list[list[str]] = []
        for _, row in df.iterrows():
            rows.append(
                [str(v) if pd.notna(v) else "" for v in row]
            )

        lines: list[str] = []
        lines.append("| " + " | ".join(headers) + " |")
        lines.append("| " + " | ".join(["---"] * len(headers)) + " |")
        for row in rows:
            lines.append("| " + " | ".join(row) + " |")
        return "\n".join(lines)

    def convert(self, path: str, **kwargs) -> ConversionResult:
        path_obj = Path(path)
        if not path_obj.exists():
            return ConversionResult(
                text="",
                error="File not found",
                error_message=f"{path} does not exist",
            )

        try:
            import pandas as pd
        except ImportError:
            raise MissingDependencyError(
                "pandas is required for XLSX conversion. "
                "Install: pip install zarishnote-ingest[xlsx]"
            )

        try:
            excel_file = pd.ExcelFile(str(path_obj))
            sheet_names = excel_file.sheet_names

            md_lines: list[str] = []
            md_lines.append(f"# {path_obj.stem}")
            md_lines.append(f"**Sheets:** {len(sheet_names)}")
            md_lines.append("")

            for sheet_name in sheet_names:
                md_lines.append(f"## {sheet_name}")
                md_lines.append("")

                try:
                    df = pd.read_excel(
                        str(path_obj),
                        sheet_name=sheet_name,
                        header=0,
                    )
                    md_lines.append(self._df_to_markdown(df))
                except Exception as e:
                    md_lines.append(f"*Error reading sheet: {e}*")
                md_lines.append("")

            text = "\n".join(md_lines)

            return ConversionResult(
                text=text,
                title=path_obj.stem,
                metadata={
                    "source": str(path_obj),
                    "sheets": sheet_names,
                    "sheet_count": len(sheet_names),
                },
            )
        except Exception as e:
            return ConversionResult(
                text="",
                error="XLSX conversion failed",
                error_message=str(e),
            )
