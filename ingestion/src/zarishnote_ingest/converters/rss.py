from typing import Optional

from .base import BaseConverter, ConversionResult
from ..exceptions import MissingDependencyError


class RssConverter(BaseConverter):
    """Convert RSS / Atom feeds to Markdown using feedparser."""

    EXTENSIONS = {".rss", ".atom"}
    MIME_TYPES = {
        "application/rss+xml",
        "application/atom+xml",
    }

    URL_PATTERNS = [
        # Common feed endpoint patterns
        __import__("re").compile(r"https?://[^/]+/(rss|feed|atom)"),
        __import__("re").compile(r"https?://[^/]+/.*\.rss$"),
        __import__("re").compile(r"https?://[^/]+/.*\.atom$"),
    ]

    @classmethod
    def accepts(cls, mime_type: str, extension: Optional[str] = None) -> bool:
        if mime_type in cls.MIME_TYPES:
            return True
        if extension and extension.lower() in cls.EXTENSIONS:
            return True
        return False

    def convert(self, path_or_url: str, **kwargs) -> ConversionResult:
        max_entries = int(kwargs.get("max_entries", 50))

        try:
            import feedparser
        except ImportError:
            raise MissingDependencyError(
                "feedparser is required for RSS/Atom feed conversion"
            )

        try:
            feed = feedparser.parse(path_or_url)

            if feed.bozo and not feed.entries:
                exc = getattr(feed, "bozo_exception", None)
                return ConversionResult(
                    text="",
                    error="Feed parse error",
                    error_message=str(exc) if exc else "Failed to parse feed",
                )

            feed_title = feed.feed.get("title", "Untitled Feed")
            feed_link = feed.feed.get("link", path_or_url)
            feed_description = feed.feed.get(
                "subtitle", feed.feed.get("description", "")
            )

            md_lines: list[str] = []
            md_lines.append(f"# {feed_title}")
            if feed_description:
                md_lines.append("")
                md_lines.append(feed_description)
            md_lines.append("")
            md_lines.append(f"**Source:** [{feed_link}]({feed_link})")
            md_lines.append(f"**Total entries:** {len(feed.entries)}")
            md_lines.append("")
            md_lines.append("---")
            md_lines.append("")

            for i, entry in enumerate(feed.entries[:max_entries]):
                entry_title = entry.get("title", f"Entry {i + 1}")
                entry_link = entry.get("link", "")
                published = entry.get("published", entry.get("updated", ""))
                summary = entry.get("summary", entry.get("description", ""))
                author = entry.get("author", "")

                md_lines.append(f"## [{entry_title}]({entry_link})")
                if published:
                    md_lines.append("")
                    md_lines.append(f"*Published: {published}*")
                if author:
                    md_lines.append("")
                    md_lines.append(f"*By: {author}*")
                if summary:
                    md_lines.append("")
                    md_lines.append(summary)
                md_lines.append("")
                md_lines.append("---")
                md_lines.append("")

            text = "\n".join(md_lines)

            return ConversionResult(
                text=text,
                title=feed_title,
                source_url=feed_link,
                metadata={
                    "feed_url": path_or_url,
                    "feed_title": feed_title,
                    "total_entries": len(feed.entries),
                    "returned_entries": min(len(feed.entries), max_entries),
                },
            )

        except Exception as e:
            return ConversionResult(
                text="",
                source_url=path_or_url,
                error="Feed conversion failed",
                error_message=str(e),
            )
