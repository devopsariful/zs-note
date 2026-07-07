import csv
import io
from pathlib import Path
from typing import Optional

from .base import BaseConverter, ConversionResult


class CsvConverter(BaseConverter):
    """Convert CSV files to Markdown tables."""

    EXTENSIONS = {".csv"}
    MIME_TYPES = {"text/csv", "text/comma-separated-values"}

    @classmethod
    def accepts(cls, mime_type: str, extension: Optional[str] = None) -> bool:
        if mime_type in cls.MIME_TYPES:
            return True
        if extension and extension.lower() in cls.EXTENSIONS:
            return True
        return False

    def _detect_delimiter(self, first_line: str) -> str:
        """Heuristically detect the CSV delimiter."""
        candidates = [",", "\t", ";", "|", ":"]
        best = ","
        best_count = 0
        for c in candidates:
            count = first_line.count(c)
            if count > best_count:
                best_count = count
                best = c
        return best

    def convert(self, path: str, **kwargs) -> ConversionResult:
        path_obj = Path(path)
        if not path_obj.exists():
            return ConversionResult(
                text="",
                error="File not found",
                error_message=f"{path} does not exist",
            )

        try:
            raw = path_obj.read_bytes()

            try:
                text = raw.decode("utf-8-sig")
            except UnicodeDecodeError:
                try:
                    from charset_normalizer import from_bytes
                    result = from_bytes(raw)
                    text = str(result.best())
                except ImportError:
                    text = raw.decode("latin-1")

            delimiter = kwargs.get("delimiter")
            if not delimiter:
                first_line = text.split("\n")[0] if text else ""
                delimiter = self._detect_delimiter(first_line)

            reader = csv.reader(io.StringIO(text), delimiter=delimiter)
            rows = list(reader)

            if not rows:
                return ConversionResult(
                    text="",
                    title=path_obj.stem,
                    metadata={"rows": 0, "columns": 0},
                )

            header = rows[0]
            md_lines: list[str] = []
            md_lines.append("| " + " | ".join(header) + " |")
            md_lines.append("| " + " | ".join(["---"] * len(header)) + " |")

            for row in rows[1:]:
                while len(row) < len(header):
                    row.append("")
                md_lines.append(
                    "| " + " | ".join(row[: len(header)]) + " |"
                )

            md_table = "\n".join(md_lines)

            return ConversionResult(
                text=md_table,
                title=path_obj.stem,
                metadata={
                    "source": str(path_obj),
                    "rows": len(rows) - 1,
                    "columns": len(header),
                    "delimiter": delimiter,
                },
            )
        except Exception as e:
            return ConversionResult(
                text="",
                error="CSV conversion failed",
                error_message=str(e),
            )
