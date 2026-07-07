import re
import zipfile
from pathlib import Path
from typing import Optional


class MimeDetector:
    """Detect MIME types with priority ordering.

    Priority: explicit hint → URL pattern → extension → magic bytes → heuristic.
    """

    EXTENSION_MAP: dict[str, str] = {
        ".pdf": "application/pdf",
        ".doc": "application/msword",
        ".docx": "application/vnd.openxmlformats-officedocument.wordprocessingml.document",
        ".pptx": "application/vnd.openxmlformats-officedocument.presentationml.presentation",
        ".xlsx": "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet",
        ".xls": "application/vnd.ms-excel",
        ".csv": "text/csv",
        ".tsv": "text/tab-separated-values",
        ".html": "text/html",
        ".htm": "text/html",
        ".xhtml": "application/xhtml+xml",
        ".xml": "application/xml",
        ".json": "application/json",
        ".md": "text/markdown",
        ".markdown": "text/markdown",
        ".txt": "text/plain",
        ".ipynb": "application/x-ipynb+json",
        ".epub": "application/epub+zip",
        ".rss": "application/rss+xml",
        ".atom": "application/atom+xml",
        ".png": "image/png",
        ".jpg": "image/jpeg",
        ".jpeg": "image/jpeg",
        ".gif": "image/gif",
        ".svg": "image/svg+xml",
        ".webp": "image/webp",
        ".mp3": "audio/mpeg",
        ".mp4": "video/mp4",
        ".wav": "audio/wav",
        ".ogg": "audio/ogg",
        ".zip": "application/zip",
        ".gz": "application/gzip",
        ".tar": "application/x-tar",
        ".yaml": "application/x-yaml",
        ".yml": "application/x-yaml",
        ".rtf": "application/rtf",
    }

    MAGIC_PREFIXES: list[tuple[bytes, str]] = [
        (b"%PDF", "application/pdf"),
        (b"\x25\x50\x44\x46", "application/pdf"),
        (b"PK\x03\x04", "application/zip"),
        (b"PK\x05\x06", "application/zip"),
        (b"PK\x07\x08", "application/zip"),
        (b"\x89PNG\r\n\x1a\n", "image/png"),
        (b"\xff\xd8\xff", "image/jpeg"),
        (b"GIF87a", "image/gif"),
        (b"GIF89a", "image/gif"),
        (b"<?xml", "application/xml"),
        (b"<!DOC", "text/html"),
        (b"<html", "text/html"),
        (b"{\n", "application/json"),
        (b"[\n", "application/json"),
        (b"\x1f\x8b", "application/gzip"),
        (b"BZh", "application/x-bzip2"),
    ]

    URL_PATTERNS: list[tuple[re.Pattern[str], str]] = [
        (re.compile(r"^https?://(?:www\.)?youtube\.com/watch"), "video/youtube"),
        (re.compile(r"^https?://youtu\.be/"), "video/youtube"),
        (re.compile(r"^https?://(?:[a-z]+)\.wikipedia\.org/"), "text/wikipedia"),
    ]

    def detect(self, path_or_url: str, mime_hint: Optional[str] = None) -> str:
        """Detect MIME type with full priority chain."""
        if mime_hint:
            return mime_hint

        if path_or_url.startswith(("http://", "https://")):
            mime = self._from_url(path_or_url)
            if mime:
                return mime
            return "text/html"

        ext = Path(path_or_url).suffix.lower()
        if ext in self.EXTENSION_MAP:
            return self.EXTENSION_MAP[ext]

        try:
            if Path(path_or_url).is_file():
                mime = self._from_magic(path_or_url)
                if mime:
                    if mime == "application/zip":
                        return self._refine_zip(path_or_url)
                    return mime
        except (OSError, IOError):
            pass

        return self._heuristic(path_or_url)

    def _from_url(self, url: str) -> Optional[str]:
        for pattern, mime in self.URL_PATTERNS:
            if pattern.match(url):
                return mime
        return None

    def _from_magic(self, path: str) -> Optional[str]:
        try:
            with open(path, "rb") as f:
                header = f.read(16)
            for magic, mime in self.MAGIC_PREFIXES:
                if header.startswith(magic):
                    return mime
        except (OSError, IOError):
            pass
        return None

    def _refine_zip(self, path: str) -> str:
        """Distinguish OOXML / EPUB from generic ZIP."""
        try:
            with zipfile.ZipFile(path) as z:
                names = z.namelist()
                if "[Content_Types].xml" in names:
                    ct = z.read("[Content_Types].xml")
                    if b"wordprocessing" in ct:
                        return "application/vnd.openxmlformats-officedocument.wordprocessingml.document"
                    if b"spreadsheet" in ct:
                        return "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
                    if b"presentation" in ct:
                        return "application/vnd.openxmlformats-officedocument.presentationml.presentation"
                if "mimetype" in names:
                    mt = z.read("mimetype")
                    if b"epub" in mt.lower():
                        return "application/epub+zip"
                return "application/zip"
        except Exception:
            return "application/zip"

    @staticmethod
    def _heuristic(path: str) -> str:
        name = Path(path).name.lower()
        if name.startswith(("rss", "feed", "atom")) or name.endswith(
            (".rss", ".atom")
        ):
            return "application/rss+xml"
        return "application/octet-stream"
