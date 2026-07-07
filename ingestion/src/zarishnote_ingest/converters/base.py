from dataclasses import dataclass, field
from abc import ABC, abstractmethod
from typing import Optional, Any


@dataclass
class ConversionResult:
    """Result of a document conversion.

    Attributes:
        text: The converted Markdown text content.
        title: Optional document title extracted during conversion.
        metadata: Arbitrary key-value metadata from the conversion.
        source_url: Original source URL if converted from a URI.
        error: Short error category string (e.g. 'File not found').
        error_message: Detailed error description.
    """
    text: str
    title: Optional[str] = None
    metadata: dict[str, Any] = field(default_factory=dict)
    source_url: Optional[str] = None
    error: Optional[str] = None
    error_message: Optional[str] = None


class BaseConverter(ABC):
    """Abstract base class for all document converters.

    Subclasses must define EXTENSIONS and/or MIME_TYPES, and implement
    ``accepts()`` and ``convert()``.
    """

    EXTENSIONS: set[str] = set()
    MIME_TYPES: set[str] = set()

    @classmethod
    @abstractmethod
    def accepts(cls, mime_type: str, extension: Optional[str] = None) -> bool:
        """Return True if this converter handles the given MIME type or file extension."""

    @abstractmethod
    def convert(self, path: str, **kwargs) -> ConversionResult:
        """Convert a document at *path* to Markdown and return a ConversionResult."""
