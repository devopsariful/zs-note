class ZarishNoteIngestError(Exception):
    """Base exception for ZarishNote Ingestion Engine."""


class MissingDependencyError(ZarishNoteIngestError):
    """Raised when an optional dependency is not installed.

    Example::
        >>> raise MissingDependencyError("pdfminer.six is required for PDF conversion")
    """


class ConversionError(ZarishNoteIngestError):
    """Raised when document conversion fails for a given input."""


class UnsupportedFormatError(ZarishNoteIngestError):
    """Raised when the input format is not supported by any converter."""
