import re
from pathlib import Path
from typing import Optional
from html.parser import HTMLParser

from .base import BaseConverter, ConversionResult
from ..exceptions import MissingDependencyError


class _EpubTextExtractor(HTMLParser):
    """Strip HTML tags and extract clean text from EPUB content."""

    def __init__(self) -> None:
        super().__init__()
        self._parts: list[str] = []
        self._skip = False

    def handle_starttag(self, tag: str, attrs: list[tuple[str, str | None]]) -> None:
        t = tag.lower()
        if t in ("script", "style"):
            self._skip = True
        if t in ("p", "br", "div", "h1", "h2", "h3", "h4", "h5", "h6", "li", "tr"):
            self._parts.append("\n")

    def handle_endtag(self, tag: str) -> None:
        t = tag.lower()
        if t in ("script", "style"):
            self._skip = False
        if t in ("p", "h1", "h2", "h3", "h4", "h5", "h6", "li", "td", "th"):
            self._parts.append("\n")

    def handle_data(self, data: str) -> None:
        if not self._skip:
            self._parts.append(data)

    def get_text(self) -> str:
        return re.sub(r"\n{3,}", "\n\n", "".join(self._parts)).strip()


class EpubConverter(BaseConverter):
    """Convert EPUB e-books to Markdown using ebooklib."""

    EXTENSIONS = {".epub"}
    MIME_TYPES = {"application/epub+zip"}

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
            import ebooklib
            from ebooklib import epub
        except ImportError:
            raise MissingDependencyError(
                "ebooklib is required for EPUB conversion. "
                "Install: pip install zarishnote-ingest[epub]"
            )

        try:
            book = epub.read_epub(str(path_obj))

            title_meta = book.get_metadata("DC", "title")
            title = title_meta[0][0] if title_meta else path_obj.stem

            chapters: list[str] = []
            items = list(book.get_items())

            for item in items:
                if item.get_type() != ebooklib.ITEM_DOCUMENT:
                    continue
                raw = item.get_content()
                if isinstance(raw, bytes):
                    raw = raw.decode("utf-8", errors="replace")
                extractor = _EpubTextExtractor()
                extractor.feed(raw)
                text = extractor.get_text()
                if text.strip():
                    chapters.append(text)

            text = "\n\n".join(chapters)

            return ConversionResult(
                text=text,
                title=title,
                metadata={
                    "source": str(path_obj),
                    "chapters": len(chapters),
                },
            )
        except Exception as e:
            return ConversionResult(
                text="",
                error="EPUB conversion failed",
                error_message=str(e),
            )
