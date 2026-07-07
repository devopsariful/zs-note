from typing import Optional, Type, List

from .base import BaseConverter, ConversionResult
from .pdf import PdfConverter
from .docx import DocxConverter
from .pptx import PptxConverter
from .xlsx import XlsxConverter
from .epub import EpubConverter
from .csv_converter import CsvConverter
from .jupyter import JupyterConverter
from .html import HtmlConverter
from .youtube import YoutubeConverter
from .wikipedia import WikipediaConverter
from .rss import RssConverter
from .serp import SerpConverter

_all_converters: List[Type[BaseConverter]] = [
    PdfConverter,
    DocxConverter,
    PptxConverter,
    XlsxConverter,
    EpubConverter,
    CsvConverter,
    JupyterConverter,
    HtmlConverter,
    YoutubeConverter,
    WikipediaConverter,
    RssConverter,
    SerpConverter,
]


def get_converter_for(
    mime_type: str, extension: Optional[str] = None
) -> Optional[Type[BaseConverter]]:
    """Return the first converter that accepts the given MIME type and/or extension."""
    for cls in _all_converters:
        try:
            if cls.accepts(mime_type, extension):
                return cls
        except Exception:
            continue
    return None


def get_converter_for_url(url: str) -> Optional[Type[BaseConverter]]:
    """Return the first converter whose URL patterns match *url*."""
    for cls in _all_converters:
        url_patterns = getattr(cls, "URL_PATTERNS", None)
        if url_patterns:
            for pattern in url_patterns:
                if pattern.match(url):
                    return cls
    return None


def list_converters() -> List[dict]:
    """Return metadata for every registered converter."""
    result: List[dict] = []
    for cls in _all_converters:
        mime_types = sorted(getattr(cls, "MIME_TYPES", set()))
        extensions = sorted(getattr(cls, "EXTENSIONS", set()))
        has_url = bool(getattr(cls, "URL_PATTERNS", None))
        result.append({
            "name": cls.__name__,
            "description": (cls.__doc__ or "").strip(),
            "mime_types": mime_types,
            "extensions": extensions,
            "supports_urls": has_url,
        })
    return result


__all__ = [
    "BaseConverter",
    "ConversionResult",
    "get_converter_for",
    "get_converter_for_url",
    "list_converters",
    "_all_converters",
]
