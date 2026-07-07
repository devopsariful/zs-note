from pathlib import Path
from typing import Optional

from .base import BaseConverter, ConversionResult
from ..exceptions import MissingDependencyError


class DocxConverter(BaseConverter):
    """Convert DOCX documents to Markdown using mammoth."""

    EXTENSIONS = {".docx"}
    MIME_TYPES = {
        "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
    }

    @classmethod
    def accepts(cls, mime_type: str, extension: Optional[str] = None) -> bool:
        if mime_type in cls.MIME_TYPES:
            return True
        if extension and extension.lower() in cls.EXTENSIONS:
            return True
        return False

    def convert(self, path: str, **kwargs) -> ConversionResult:
        path_obj = Path(path)
        if not path_obj.exists():
            return ConversionResult(
                text="",
                error="File not found",
                error_message=f"{path} does not exist",
            )

        try:
            import mammoth
        except ImportError:
            raise MissingDependencyError(
                "mammoth is required for DOCX conversion. "
                "Install: pip install zarishnote-ingest[docx]"
            )

        try:
            with open(str(path_obj), "rb") as f:
                result = mammoth.convert_to_markdown(f)

            text = result.value

            title = path_obj.stem
            for line in text.split("\n"):
                stripped = line.lstrip("# ").strip()
                if line.startswith("#") and stripped:
                    title = stripped
                    break

            warnings = [str(w) for w in result.messages] if result.messages else []

            return ConversionResult(
                text=text,
                title=title,
                metadata={
                    "source": str(path_obj),
                    "warnings": warnings,
                },
            )
        except Exception as e:
            return ConversionResult(
                text="",
                error="DOCX conversion failed",
                error_message=str(e),
            )
