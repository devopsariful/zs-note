import re
from typing import Optional
from io import BytesIO


class CharsetDetector:
    """Detect character encoding with priority ordering.

    Priority: explicit flag → HTTP header → HTML meta → charset-normalizer → UTF-8.
    """

    FALLBACK_ENCODINGS = [
        "utf-8",
        "latin-1",
        "cp1252",
        "iso-8859-15",
        "windows-1252",
        "shift_jis",
        "euc-jp",
        "gbk",
        "big5",
    ]

    def detect(
        self,
        data: bytes,
        explicit: Optional[str] = None,
        http_header: Optional[str] = None,
    ) -> str:
        """Return the best-guess encoding name."""
        # 1 — explicit flag
        if explicit:
            if self._valid_encoding(explicit, data):
                return explicit

        # 2 — HTTP Content-Type header
        if http_header:
            charset = self.from_http_header(http_header)
            if charset and self._valid_encoding(charset, data):
                return charset

        # 3 — HTML <meta> / XML declaration
        meta_charset = self.from_html_meta(data)
        if meta_charset and self._valid_encoding(meta_charset, data):
            return meta_charset

        # 4 — charset-normalizer library
        try:
            from charset_normalizer import from_bytes

            result = from_bytes(data)
            if result.best():
                return str(result.best().encoding)
        except ImportError:
            pass

        # 5 — try common encodings in order
        for enc in self.FALLBACK_ENCODINGS:
            if self._valid_encoding(enc, data):
                return enc

        return "utf-8"

    def decode(
        self,
        data: bytes,
        explicit: Optional[str] = None,
        http_header: Optional[str] = None,
    ) -> str:
        """Detect encoding and decode bytes to str."""
        charset = self.detect(data, explicit, http_header)
        return data.decode(charset, errors="replace")

    @staticmethod
    def from_http_header(header: str) -> Optional[str]:
        """Extract charset from a Content-Type header value."""
        m = re.search(r"charset=([^\s;]+)", header, re.IGNORECASE)
        return m.group(1).strip().lower() if m else None

    @staticmethod
    def from_html_meta(data: bytes) -> Optional[str]:
        """Extract charset from HTML <meta> or XML declaration (first 2048 bytes)."""
        head = data[:2048]

        # XML declaration: <?xml ... encoding="..." ?>
        m = re.search(
            rb"""encoding\s*=\s*["']([^"']+)["']""", head, re.IGNORECASE
        )
        if m:
            try:
                return m.group(1).decode("ascii").strip().lower()
            except (UnicodeDecodeError, ValueError):
                pass

        # HTML5: <meta charset="utf-8">
        m = re.search(
            rb"""<meta\s[^>]*charset\s*=\s*["']?([^"'>\s]+)""",
            head,
            re.IGNORECASE,
        )
        if m:
            try:
                return m.group(1).decode("ascii").strip().lower()
            except (UnicodeDecodeError, ValueError):
                pass

        # XHTML/HTML4: <meta http-equiv="Content-Type" content="...charset=...">
        m = re.search(
            rb"""<meta\s[^>]*http-equiv\s*=\s*["']?content-type["']?[^>]*"""
            rb"""content\s*=\s*["'][^"']*charset=([^"'\s]+)""",
            head,
            re.IGNORECASE,
        )
        if m:
            try:
                return m.group(1).decode("ascii").strip().lower()
            except (UnicodeDecodeError, ValueError):
                pass

        return None

    @staticmethod
    def safe_xml_parse(data: bytes):
        """Parse XML safely with defusedxml (XXE prevention).

        Returns the root element on success, or *None* on parse failure.
        """
        try:
            from defusedxml.ElementTree import parse, fromstring
        except ImportError:
            return None
        try:
            tree = parse(BytesIO(data))
            return tree.getroot()
        except Exception:
            pass
        try:
            return fromstring(data)
        except Exception:
            return None

    @staticmethod
    def _valid_encoding(name: str, data: bytes) -> bool:
        """Return *True* if *name* is a known codec and decodes *data*."""
        try:
            data.decode(name)
            return True
        except (LookupError, UnicodeDecodeError):
            return False
