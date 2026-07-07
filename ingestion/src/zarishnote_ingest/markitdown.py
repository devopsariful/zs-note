from __future__ import annotations

from dataclasses import dataclass, field
from pathlib import Path
from typing import Optional, Any, IO, List
import logging
import os
import tempfile

import requests

from .mime_detect import MimeDetector
from .charset import CharsetDetector
from .exceptions import MissingDependencyError

logger = logging.getLogger(__name__)


@dataclass
class ConversionResult:
    """Result of a single document-to-Markdown conversion."""

    text: str
    title: Optional[str] = None
    metadata: dict[str, Any] = field(default_factory=dict)
    source_url: Optional[str] = None
    error: Optional[str] = None
    error_message: Optional[str] = None


class ZarishNoteIngester:
    """Main document ingester that wraps Microsoft's ``markitdown`` library.

    Falls back to the custom converters shipped in :mod:`zarishnote_ingest.converters`
    when the upstream library is unavailable or fails for a particular format.
    """

    def __init__(
        self,
        plugins: bool = True,
        vision_model: Optional[str] = None,
    ) -> None:
        self._plugins = plugins
        self._vision_model = vision_model
        self._mime_detector = MimeDetector()
        self._charset_detector = CharsetDetector()
        self._markitdown: Any = None
        self._init_markitdown()

    # ------------------------------------------------------------------
    # Internal helpers
    # ------------------------------------------------------------------

    def _init_markitdown(self) -> None:
        if not self._plugins:
            return
        try:
            from markitdown import MarkItDown  # type: ignore[import-untyped]

            kwargs: dict[str, Any] = {}
            if self._vision_model:
                kwargs["vision_model"] = self._vision_model
            self._markitdown = MarkItDown(**kwargs)
            logger.debug("Initialised Microsoft markitdown library")
        except ImportError:
            logger.info("markitdown not available — using custom converters only")
        except Exception as exc:
            logger.warning("Failed to initialise markitdown: %s", exc)

    def _get_converter(self, mime_type: str, extension: Optional[str] = None):
        from .converters import get_converter_for

        return get_converter_for(mime_type, extension)

    # ------------------------------------------------------------------
    # Public API
    # ------------------------------------------------------------------

    def convert_local(
        self,
        path: str,
        mime_hint: Optional[str] = None,
        charset: Optional[str] = None,
    ) -> ConversionResult:
        """Convert a local file to Markdown."""
        path_obj = Path(path)
        if not path_obj.exists():
            return ConversionResult(
                text="",
                error="File not found",
                error_message=f"{path} does not exist",
            )

        ext = path_obj.suffix.lower()
        mime_type = mime_hint or self._mime_detector.detect(path)

        # 1) Try upstream markitdown
        if self._markitdown is not None:
            try:
                result = self._markitdown.convert(str(path_obj))
                return ConversionResult(
                    text=result.text_content,
                    title=path_obj.stem,
                    metadata={
                        "source": str(path_obj),
                        "mime_type": mime_type,
                    },
                )
            except Exception as exc:
                logger.debug("markitdown.convert() failed for %s: %s", path, exc)

        # 2) Fall back to custom converter
        converter_cls = self._get_converter(mime_type, ext)
        if converter_cls is not None:
            try:
                converter = converter_cls()
                return converter.convert(str(path_obj), charset=charset)
            except MissingDependencyError:
                raise
            except Exception as exc:
                return ConversionResult(
                    text="",
                    error="Conversion failed",
                    error_message=str(exc),
                )

        # 3) Last resort — read as plain text
        try:
            raw = path_obj.read_bytes()
            text = self._charset_detector.decode(raw, explicit=charset)
            return ConversionResult(
                text=text,
                title=path_obj.stem,
                metadata={"source": str(path_obj), "mime_type": "text/plain"},
            )
        except Exception as exc:
            return ConversionResult(
                text="",
                error="Conversion failed",
                error_message=str(exc),
            )

    def convert_uri(
        self,
        url: str,
        vision_model: Optional[str] = None,
    ) -> ConversionResult:
        """Convert a URI (URL) to Markdown.

        URL-specific converters (YouTube, Wikipedia, …) are checked first,
        then the upstream markitdown library, and finally a plain HTTP fetch.
        """
        from .converters import get_converter_for_url

        # 1) URL-specific converter (YouTube, Wikipedia …)
        converter_cls = get_converter_for_url(url)
        if converter_cls is not None:
            try:
                converter = converter_cls()
                return converter.convert(url, vision_model=vision_model)
            except MissingDependencyError:
                raise
            except Exception as exc:
                return ConversionResult(
                    text="",
                    source_url=url,
                    error="URL conversion failed",
                    error_message=str(exc),
                )

        # 2) Upstream markitdown
        if self._markitdown is not None:
            try:
                result = self._markitdown.convert(url)
                return ConversionResult(
                    text=result.text_content,
                    source_url=url,
                    metadata={"source_url": url},
                )
            except Exception as exc:
                logger.debug("markitdown URL conversion failed: %s", exc)

        # 3) Plain HTTP fetch + HTML conversion
        try:
            resp = requests.get(
                url,
                timeout=30,
                headers={"User-Agent": "ZarishNote-Ingest/0.1.0"},
            )
            resp.raise_for_status()
            return self.convert_response(resp)
        except requests.RequestException as exc:
            return ConversionResult(
                text="",
                source_url=url,
                error="URL fetch failed",
                error_message=str(exc),
            )

    def convert_stream(self, stream: IO, mime_type: str) -> ConversionResult:
        """Convert an open binary/text stream to Markdown."""
        if self._markitdown is not None:
            try:
                result = self._markitdown.convert(stream)
                return ConversionResult(
                    text=result.text_content,
                    metadata={"mime_type": mime_type},
                )
            except Exception as exc:
                logger.debug("markitdown stream conversion failed: %s", exc)

        try:
            raw = stream.read()
            if isinstance(raw, bytes):
                text = self._charset_detector.decode(raw)
            else:
                text = raw
            return ConversionResult(
                text=text,
                metadata={"mime_type": mime_type},
            )
        except Exception as exc:
            return ConversionResult(
                text="",
                error="Stream conversion failed",
                error_message=str(exc),
            )

    def convert_response(self, response) -> ConversionResult:
        """Convert a ``requests.Response`` to Markdown.

        For HTML responses the content is run through
        :class:`~zarishnote_ingest.converters.html.HtmlConverter`.
        """
        content_type = response.headers.get("Content-Type", "")
        mime_type = content_type.split(";")[0].strip().lower()
        charset = self._charset_detector.from_http_header(content_type)
        url = str(response.url)

        # Check URL-specific converter
        from .converters import get_converter_for_url

        converter_cls = get_converter_for_url(url)
        if converter_cls is not None:
            try:
                converter = converter_cls()
                return converter.convert(url)
            except Exception:
                pass

        # HTML path
        if mime_type in ("text/html", "application/xhtml+xml"):
            from .converters.html import HtmlConverter

            try:
                with tempfile.NamedTemporaryFile(
                    suffix=".html",
                    delete=False,
                    mode="w",
                    encoding="utf-8",
                ) as f:
                    f.write(response.text)
                    tmp = f.name

                try:
                    conv = HtmlConverter()
                    result = conv.convert(tmp, url=url, charset=charset)
                    result.source_url = url
                    return result
                finally:
                    os.unlink(tmp)
            except Exception as exc:
                return ConversionResult(
                    text=response.text,
                    source_url=url,
                    error="HTML conversion failed",
                    error_message=str(exc),
                )

        return ConversionResult(
            text=response.text,
            source_url=url,
            metadata={"mime_type": mime_type},
        )

    def list_converters(self) -> List[dict]:
        """Return metadata for every available converter."""
        from .converters import list_converters

        return list_converters()
