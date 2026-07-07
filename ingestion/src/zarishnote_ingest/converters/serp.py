import re
from pathlib import Path
from typing import Optional

from .base import BaseConverter, ConversionResult
from ..exceptions import MissingDependencyError


class SerpConverter(BaseConverter):
    """Convert Bing / generic search engine result pages to Markdown.

    Works with local HTML files or URLs containing search results.
    """

    EXTENSIONS = {".html", ".htm"}
    MIME_TYPES = {"text/html"}

    @classmethod
    def accepts(cls, mime_type: str, extension: Optional[str] = None) -> bool:
        if mime_type in cls.MIME_TYPES:
            return True
        if extension and extension.lower() in cls.EXTENSIONS:
            return True
        return False

    def convert(self, path_or_url: str, **kwargs) -> ConversionResult:
        try:
            from bs4 import BeautifulSoup
        except ImportError:
            raise MissingDependencyError(
                "beautifulsoup4 is required for search results conversion"
            )

        try:
            if path_or_url.startswith(("http://", "https://")):
                import requests

                resp = requests.get(
                    path_or_url,
                    timeout=15,
                    headers={
                        "User-Agent": (
                            "Mozilla/5.0 (Windows NT 10.0; Win64; x64) "
                            "AppleWebKit/537.36 (KHTML, like Gecko) "
                            "Chrome/120.0.0.0 Safari/537.36"
                        )
                    },
                )
                resp.raise_for_status()
                html_content = resp.text
                source_url = path_or_url
            else:
                p = Path(path_or_url)
                if not p.exists():
                    return ConversionResult(
                        text="",
                        error="File not found",
                        error_message=f"{path_or_url} does not exist",
                    )
                html_content = p.read_text(encoding="utf-8", errors="replace")
                source_url = ""

            soup = BeautifulSoup(html_content, "html.parser")

            title_tag = soup.find("title")
            query = ""
            if title_tag:
                m = re.match(
                    r"^(.+?)\s*[–\-]\s*Search", title_tag.get_text(strip=True)
                )
                if m:
                    query = m.group(1)

            md_lines: list[str] = []
            md_lines.append("# Search Results")
            if query:
                md_lines.append(f"**Query:** {query}")
            md_lines.append("**Source:** Bing")
            if source_url:
                md_lines.append(f"**URL:** {source_url}")
            md_lines.append("")

            # Bing uses <li class="b_algo">
            results = soup.find_all("li", class_="b_algo")
            if not results:
                results = soup.select(
                    "li.b_algo, .b_algo, .g, .result, .search-result, "
                    "div[class*=result], div[class*=search]"
                )
            if not results:
                results = soup.find_all(
                    ["h2", "h3"],
                    class_=lambda c: c and "title" in c.lower() if c else False,
                )

            for i, result in enumerate(results[:20]):
                link = result.find("a") if hasattr(result, "find") else None
                if link:
                    href = link.get("href", "")
                    if href and not href.startswith("http"):
                        href = ""
                    title_text = link.get_text(strip=True)
                else:
                    href = ""
                    title_text = (
                        result.get_text(strip=True)[:100]
                        if hasattr(result, "get_text")
                        else ""
                    )

                snippet = ""
                for klass in ("st", "desc", "description", "snippet"):
                    tag = result.find(["p", "span", "div"], class_=klass)
                    if tag:
                        snippet = tag.get_text(strip=True)
                        break
                if not snippet:
                    p_tag = result.find("p")
                    if p_tag:
                        snippet = p_tag.get_text(strip=True)

                if title_text:
                    link_part = f"]({href})" if href else "]()"
                    md_lines.append(
                        f"{i + 1}. **[{title_text}{link_part}**"
                    )
                    if snippet:
                        md_lines.append(f"   > {snippet}")
                    md_lines.append("")

            has_results = any(
                line.strip().startswith(("1.", "1)")) for line in md_lines
            )
            if not has_results:
                md_lines.append("*No structured results extracted.*")
                md_lines.append("")
                body = soup.find("body")
                if body:
                    for tag in body.find_all(["script", "style", "nav", "footer"]):
                        tag.decompose()
                    raw = body.get_text(separator="\n", strip=True)
                    md_lines.append(raw[:5000])

            text = "\n".join(md_lines)

            return ConversionResult(
                text=text,
                title=f"Search Results{' - ' + query if query else ''}",
                source_url=source_url,
                metadata={
                    "query": query,
                    "results_count": len(results),
                    "source": "bing",
                },
            )

        except Exception as e:
            return ConversionResult(
                text="",
                error="Search results conversion failed",
                error_message=str(e),
            )
