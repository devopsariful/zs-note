import re
from typing import Optional
from urllib.parse import quote

from .base import BaseConverter, ConversionResult
from ..exceptions import MissingDependencyError


class WikipediaConverter(BaseConverter):
    """Convert Wikipedia articles to Markdown via the REST API."""

    URL_PATTERNS = [
        re.compile(r"^https?://([a-z]+)\.wikipedia\.org/wiki/(.+)$"),
        re.compile(r"^https?://([a-z]+)\.m\.wikipedia\.org/wiki/(.+)$"),
    ]

    @classmethod
    def accepts(cls, mime_type: str, extension: Optional[str] = None) -> bool:
        return False

    def _parse_url(self, url: str) -> tuple[str, str]:
        for p in self.URL_PATTERNS:
            m = p.match(url)
            if m:
                return m.group(1), m.group(2)
        return "en", ""

    def convert(self, url: str, **kwargs) -> ConversionResult:
        lang, page_title = self._parse_url(url)
        if not page_title:
            return ConversionResult(
                text="",
                source_url=url,
                error="Invalid Wikipedia URL",
                error_message=f"Could not parse URL: {url}",
            )

        page_title = page_title.split("#")[0]
        page_title_quoted = quote(page_title, safe="")

        try:
            import requests
        except ImportError:
            raise MissingDependencyError(
                "requests is required for Wikipedia conversion"
            )

        ua = "ZarishNote-Ingest/0.1.0"

        # 1) summary
        summary_url = (
            f"https://{lang}.wikipedia.org/api/rest_v1/page/summary/"
            f"{page_title_quoted}"
        )
        try:
            resp = requests.get(summary_url, timeout=15, headers={"User-Agent": ua})
            resp.raise_for_status()
            data = resp.json()
        except requests.HTTPError as e:
            status = e.response.status_code if e.response is not None else 0
            if status == 404:
                return ConversionResult(
                    text="",
                    source_url=url,
                    error="Page not found",
                    error_message=f'Wikipedia page "{page_title}" not found',
                )
            return ConversionResult(
                text="",
                source_url=url,
                error="HTTP error",
                error_message=str(e),
            )
        except Exception as e:
            return ConversionResult(
                text="",
                source_url=url,
                error="Wikipedia request failed",
                error_message=str(e),
            )

        article_title = data.get("title", page_title)
        extract = data.get("extract", "")

        md_lines: list[str] = []
        md_lines.append(f"# {article_title}")
        md_lines.append(f"*Source: {url}*")
        md_lines.append("")

        # 2) try to fetch full HTML for infobox
        try:
            html_url = (
                f"https://{lang}.wikipedia.org/api/rest_v1/page/html/"
                f"{page_title_quoted}"
            )
            html_resp = requests.get(html_url, timeout=15, headers={"User-Agent": ua})
            if html_resp.status_code == 200:
                from bs4 import BeautifulSoup

                soup = BeautifulSoup(html_resp.text, "html.parser")
                infobox = soup.find("table", class_="infobox")
                if infobox:
                    md_lines.append("## Infobox")
                    md_lines.append("")
                    for row in infobox.find_all("tr"):
                        th = row.find("th")
                        td = row.find("td")
                        if th and td:
                            md_lines.append(
                                f"| **{th.get_text(strip=True)}** | "
                                f"{td.get_text(strip=True)} |"
                            )
                    md_lines.append("")
                    md_lines.append("---")
                    md_lines.append("")
        except Exception:
            pass

        # 3) main extract
        if extract:
            md_lines.append(extract)
            md_lines.append("")

        # 4) sections
        try:
            sections_url = (
                f"https://{lang}.wikipedia.org/api/rest_v1/page/sections-bare/"
                f"{page_title_quoted}"
            )
            sec_resp = requests.get(
                sections_url, timeout=10, headers={"User-Agent": ua}
            )
            if sec_resp.status_code == 200:
                sections = sec_resp.json().get("items", [])
                skip_sections = {"References", "External links", "See also", "Notes"}

                for section in sections:
                    line = section.get("line", "")
                    level = min(section.get("level", 2), 6)
                    if line in skip_sections or not line:
                        continue

                    md_lines.append(f"{'#' * level} {line}")
                    md_lines.append("")

                    sec_id = section.get("id", "")
                    content_url = (
                        f"https://{lang}.wikipedia.org/api/rest_v1/page/html/"
                        f"{page_title_quoted}/{sec_id}"
                    )
                    try:
                        c_resp = requests.get(
                            content_url, timeout=10, headers={"User-Agent": ua}
                        )
                        if c_resp.status_code == 200:
                            from bs4 import BeautifulSoup

                            c_soup = BeautifulSoup(c_resp.text, "html.parser")
                            for el in c_soup.find_all(["p", "li"]):
                                txt = el.get_text(strip=True)
                                if txt:
                                    md_lines.append(txt)
                                    md_lines.append("")
                    except Exception:
                        pass
        except Exception:
            pass

        text = "\n".join(md_lines)

        return ConversionResult(
            text=text,
            title=article_title,
            source_url=url,
            metadata={
                "language": lang,
                "title": article_title,
                "source": url,
            },
        )
