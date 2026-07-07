import re
from typing import Optional

from .base import BaseConverter, ConversionResult
from ..exceptions import MissingDependencyError


class YoutubeConverter(BaseConverter):
    """Convert YouTube video transcripts to Markdown.

    Accepts youtube.com/watch, youtu.be, youtube.com/embed, and
    youtube.com/shorts URLs.
    """

    URL_PATTERNS = [
        re.compile(r"^https?://(?:www\.)?youtube\.com/watch\?v=([\w-]+)"),
        re.compile(r"^https?://(?:www\.)?youtu\.be/([\w-]+)"),
        re.compile(r"^https?://(?:www\.)?youtube\.com/embed/([\w-]+)"),
        re.compile(r"^https?://(?:www\.)?youtube\.com/shorts/([\w-]+)"),
    ]

    @classmethod
    def accepts(cls, mime_type: str, extension: Optional[str] = None) -> bool:
        return False

    def _extract_video_id(self, url: str) -> Optional[str]:
        for p in self.URL_PATTERNS:
            m = p.match(url)
            if m:
                return m.group(1)
        return None

    def convert(self, url: str, **kwargs) -> ConversionResult:
        video_id = self._extract_video_id(url)
        if not video_id:
            return ConversionResult(
                text="",
                source_url=url,
                error="Invalid YouTube URL",
                error_message=f"Could not extract video ID from: {url}",
            )

        try:
            from youtube_transcript_api import YouTubeTranscriptApi
        except ImportError:
            raise MissingDependencyError(
                "youtube-transcript-api is required for YouTube conversion. "
                "Install: pip install zarishnote-ingest[youtube]"
            )

        try:
            transcript_list = YouTubeTranscriptApi.get_transcript(video_id)

            title = f"YouTube Video {video_id}"
            channel = ""

            try:
                import requests
                oembed = (
                    "https://www.youtube.com/oembed?"
                    f"url=https://www.youtube.com/watch?v={video_id}&format=json"
                )
                resp = requests.get(oembed, timeout=10)
                if resp.status_code == 200:
                    data = resp.json()
                    title = data.get("title", title)
                    channel = data.get("author_name", "")
            except Exception:
                pass

            md_lines: list[str] = []
            md_lines.append(f"# {title}")
            if channel:
                md_lines.append(f"**Channel:** {channel}")
            md_lines.append(f"**Source:** {url}")
            md_lines.append(f"**Video ID:** {video_id}")
            md_lines.append("")
            md_lines.append("## Transcript")
            md_lines.append("")

            for entry in transcript_list:
                ts = int(entry["start"])
                mm, ss = divmod(ts, 60)
                md_lines.append(f"[{mm:02d}:{ss:02d}] {entry['text']}")

            text = "\n".join(md_lines)

            return ConversionResult(
                text=text,
                title=title,
                source_url=url,
                metadata={
                    "video_id": video_id,
                    "channel": channel,
                    "transcript_entries": len(transcript_list),
                },
            )

        except Exception as e:
            msg = str(e)
            if "TranscriptsDisabled" in msg:
                msg = "Transcripts are disabled for this video"
            elif "NoTranscriptFound" in msg:
                msg = "No transcript found for this video"
            return ConversionResult(
                text="",
                source_url=url,
                error="YouTube transcript fetch failed",
                error_message=msg,
                metadata={"video_id": video_id},
            )
