from pathlib import Path
from typing import Optional

from .base import BaseConverter, ConversionResult
from ..exceptions import MissingDependencyError


class PdfConverter(BaseConverter):
    """Convert PDF documents to Markdown using pdfminer.six."""

    EXTENSIONS = {".pdf"}
    MIME_TYPES = {"application/pdf"}

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
            from pdfminer.high_level import extract_text
        except ImportError:
            raise MissingDependencyError(
                "pdfminer.six is required for PDF conversion. "
                "Install: pip install zarishnote-ingest[pdf]"
            )

        try:
            text = extract_text(str(path_obj))

            page_count = text.count("\f") + 1 if text else 0

            # Try pdfplumber for richer extraction if available
            try:
                import pdfplumber

                with pdfplumber.open(str(path_obj)) as pdf:
                    metadata = {
                        "source": str(path_obj),
                        "pages": len(pdf.pages),
                        "page_count": len(pdf.pages),
                    }
                    if pdf.metadata:
                        metadata.update(
                            {k: str(v) for k, v in pdf.metadata.items() if v}
                        )
            except ImportError:
                metadata = {"source": str(path_obj), "pages": page_count}

            # Filter out empty pages
            clean_text = "\n\n".join(
                p.strip()
                for p in text.split("\f")
                if p.strip()
            )

            return ConversionResult(
                text=clean_text,
                title=path_obj.stem,
                metadata=metadata,
            )
        except Exception as e:
            return ConversionResult(
                text="",
                error="PDF conversion failed",
                error_message=str(e),
            )
