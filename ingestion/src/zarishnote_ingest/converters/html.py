import re
from pathlib import Path
from typing import Optional

from .base import BaseConverter, ConversionResult
from ..exceptions import MissingDependencyError


class HtmlConverter(BaseConverter):
    """Convert HTML documents to Markdown using BeautifulSoup + markdownify."""

    EXTENSIONS = {".html", ".htm", ".xhtml"}
    MIME_TYPES = {"text/html", "application/xhtml+xml"}

    @classmethod
    def accepts(cls, mime_type: str, extension: Optional[str] = None) -> bool:
        if mime_type in cls.MIME_TYPES:
            return True
        if extension and extension.lower() in cls.EXTENSIONS:
            return True
        return False

    def convert(self, path: str, **kwargs) -> ConversionResult:
        path_obj = Path(path)
        url = kwargs.get("url", "")
        charset = kwargs.get("charset")

        if not path_obj.exists():
            return ConversionResult(
                text="",
                error="File not found",
                error_message=f"{path} does not exist",
            )

        try:
            from bs4 import BeautifulSoup
            from markdownify import markdownify as md
        except ImportError:
            raise MissingDependencyError(
                "beautifulsoup4 and markdownify are required for HTML conversion"
            )

        try:
            raw = path_obj.read_bytes()

            if charset:
                html_content = raw.decode(charset, errors="replace")
            else:
                html_text = raw.decode("utf-8", errors="replace")
                charset_match = re.search(
                    rb'charset[=:]\s*([^\s";<>]+)', raw, re.IGNORECASE
                )
                if charset_match:
                    try:
                        detected = charset_match.group(1).decode("ascii")
                        html_content = raw.decode(detected, errors="replace")
                    except (LookupError, UnicodeDecodeError):
                        html_content = html_text
                else:
                    try:
                        from charset_normalizer import from_bytes
                        result = from_bytes(raw)
                        html_content = str(result.best())
                    except ImportError:
                        html_content = html_text

            soup = BeautifulSoup(html_content, "html.parser")

            title_tag = soup.find("title")
            title = title_tag.get_text(strip=True) if title_tag else path_obj.stem

            for tag in soup(["script", "style", "nav", "footer", "header", "aside"]):
                tag.decompose()

            main_content = (
                soup.find("article")
                or soup.find("main")
                or soup.find('[role="main"]')
                or soup.find("body")
                or soup
            )

            html_str = str(main_content)
            markdown_text = md(
                html_str,
                heading_style="ATX",
                strip=["script", "style"],
            )

            markdown_text = re.sub(r"\n{3,}", "\n\n", markdown_text).strip()

            return ConversionResult(
                text=markdown_text,
                title=title,
                source_url=url or str(path_obj),
                metadata={
                    "source": str(path_obj),
                    "url": url,
                },
            )
        except Exception as e:
            return ConversionResult(
                text="",
                error="HTML conversion failed",
                error_message=str(e),
            )
