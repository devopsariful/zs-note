"""Tests for the ZarishNote ingestion pipeline.

Covers MIME detection, the ``ZarishNoteIngester`` class, and individual
converters (CSV, HTML).
"""

from __future__ import annotations

import textwrap
from pathlib import Path
from unittest.mock import patch

import pytest

from zarishnote_ingest.mime_detect import MimeDetector
from zarishnote_ingest.markitdown import ZarishNoteIngester, ConversionResult
from zarishnote_ingest.converters.csv_converter import CsvConverter
from zarishnote_ingest.converters.html import HtmlConverter


# ===================================================================
# MimeDetect
# ===================================================================


class TestMimeDetect:
    """Tests for :class:`MimeDetector`."""

    def setup_method(self):
        self.detector = MimeDetector()

    def test_detect_by_extension_markdown(self):
        assert self.detector.detect("readme.md") == "text/markdown"

    def test_detect_by_extension_html(self):
        assert self.detector.detect("index.html") == "text/html"
        assert self.detector.detect("page.htm") == "text/html"

    def test_detect_by_extension_csv(self):
        assert self.detector.detect("data.csv") == "text/csv"

    def test_detect_by_extension_pdf(self):
        assert self.detector.detect("doc.pdf") == "application/pdf"

    def test_detect_by_extension_json(self):
        assert self.detector.detect("config.json") == "application/json"

    def test_detect_by_extension_unknown(self):
        assert self.detector.detect("file.xyz") == "application/octet-stream"

    def test_detect_by_magic_pdf(self, tmp_data_dir: Path):
        """Detect a PDF-like file by magic bytes."""
        p = tmp_data_dir / "test.pdf"
        p.write_bytes(b"%PDF-1.4\n...")
        assert self.detector.detect(str(p)) == "application/pdf"

    def test_detect_by_magic_png(self, tmp_data_dir: Path):
        p = tmp_data_dir / "image.png"
        p.write_bytes(b"\x89PNG\r\n\x1a\n...")
        assert self.detector.detect(str(p)) == "image/png"

    def test_detect_by_magic_xml(self, tmp_data_dir: Path):
        p = tmp_data_dir / "data.xml"
        p.write_bytes(b"<?xml version='1.0'?>")
        assert self.detector.detect(str(p)) == "application/xml"

    def test_detect_by_magic_gzip(self, tmp_data_dir: Path):
        p = tmp_data_dir / "archive.gz"
        p.write_bytes(b"\x1f\x8b...")
        assert self.detector.detect(str(p)) == "application/gzip"

    def test_detect_unknown_binary(self, tmp_data_dir: Path, binary_file: Path):
        mime = self.detector.detect(str(binary_file))
        assert mime == "application/octet-stream"

    def test_detect_url_youtube(self):
        urls = [
            "https://www.youtube.com/watch?v=dQw4w9WgXcQ",
            "https://youtu.be/dQw4w9WgXcQ",
        ]
        for url in urls:
            assert self.detector.detect(url) == "video/youtube"

    def test_detect_url_wikipedia(self):
        url = "https://en.wikipedia.org/wiki/Rust_(programming_language)"
        assert self.detector.detect(url) == "text/wikipedia"

    def test_detect_url_unknown(self):
        url = "https://example.com/page"
        assert self.detector.detect(url) == "text/html"

    def test_detect_mime_hint_takes_priority(self):
        assert self.detector.detect("file.xyz", mime_hint="text/plain") == "text/plain"

    def test_detect_empty_path(self):
        assert self.detector.detect("") == "application/octet-stream"

    def test_detect_refine_zip_ooxml(self, tmp_data_dir: Path):
        """ZIP magic bytes + OOXML content types."""
        import zipfile
        p = tmp_data_dir / "test.docx"
        with zipfile.ZipFile(p, "w") as zf:
            zf.writestr("[Content_Types].xml",
                        '<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">'
                        '<Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>'
                        '<Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>'
                        '</Types>')
        mime = self.detector.detect(str(p))
        assert "wordprocessing" in mime


# ===================================================================
# ZarishNoteIngester
# ===================================================================


class TestMarkitdownIngester:
    """Tests for :class:`ZarishNoteIngester`."""

    def test_ingest_markdown_local(self, sample_markdown_file: Path):
        """Convert a local Markdown file."""
        ingester = ZarishNoteIngester(plugins=False)
        result = ingester.convert_local(str(sample_markdown_file))
        assert result.error is None, f"unexpected error: {result.error_message}"
        assert "# Sample Document" in result.text
        assert result.title == "sample"

    def test_ingest_html_local(self, sample_html_file: Path):
        """Convert a local HTML file."""
        ingester = ZarishNoteIngester(plugins=False)
        result = ingester.convert_local(str(sample_html_file))
        assert result.error is None, f"unexpected error: {result.error_message}"
        assert "Hello World" in result.text
        assert result.title == "Test Page"

    def test_ingest_plain_text(self, sample_text_file: Path):
        """Read a plain text file."""
        ingester = ZarishNoteIngester(plugins=False)
        result = ingester.convert_local(str(sample_text_file))
        assert result.error is None
        assert "Hello, ZarishNote!" in result.text

    def test_ingest_nonexistent_file(self, nonexistent_path: Path):
        """Non‑existent file should return an error result, not crash."""
        ingester = ZarishNoteIngester(plugins=False)
        result = ingester.convert_local(str(nonexistent_path))
        assert result.error == "File not found"
        assert result.text == ""

    def test_ingest_empty(self, empty_file: Path):
        """An empty file should produce an empty result."""
        ingester = ZarishNoteIngester(plugins=False)
        result = ingester.convert_local(str(empty_file))
        assert result.error is None
        assert result.text == ""

    def test_ingest_with_mime_hint(self, sample_text_file: Path):
        """An explicit MIME hint should be honoured."""
        ingester = ZarishNoteIngester(plugins=False)
        result = ingester.convert_local(str(sample_text_file), mime_hint="text/markdown")
        assert result.error is None

    def test_conversion_result_fields(self):
        """Make sure ConversionResult dataclass fields work."""
        r = ConversionResult(
            text="# Hello",
            title="Test",
            metadata={"key": "value"},
            source_url="https://example.com",
        )
        assert r.text == "# Hello"
        assert r.title == "Test"
        assert r.metadata["key"] == "value"
        assert r.source_url == "https://example.com"

    def test_conversion_result_error(self):
        r = ConversionResult(
            text="",
            error="Something broke",
            error_message="Detailed reason",
        )
        assert r.error == "Something broke"
        assert r.error_message == "Detailed reason"

    def test_ingester_list_converters(self):
        """list_converters should return metadata dicts."""
        ingester = ZarishNoteIngester(plugins=False)
        converters = ingester.list_converters()
        assert isinstance(converters, list)
        assert len(converters) > 0
        names = [c["name"] for c in converters]
        assert "CsvConverter" in names
        assert "HtmlConverter" in names
        assert "PdfConverter" in names

    def test_ingester_with_markitdown_unavailable(self):
        """When markitdown is not available, the ingester should still work."""
        with patch("zarishnote_ingest.markitdown.ZarishNoteIngester._init_markitdown"):
            ingester = ZarishNoteIngester(plugins=False)
            # The ingester should still be usable
            assert ingester is not None


# ===================================================================
# Individual converters
# ===================================================================


class TestConverters:
    """Tests for specific converter implementations."""

    def test_csv_converter(self, sample_csv_file: Path):
        """CSV → Markdown table conversion."""
        converter = CsvConverter()
        result = converter.convert(str(sample_csv_file))
        assert result.error is None, f"CSV conversion error: {result.error_message}"
        assert "| Name" in result.text
        assert "| ---" in result.text
        assert "Alice" in result.text
        assert "Bob" in result.text
        assert result.title == "sample"
        assert result.metadata["rows"] == 2
        assert result.metadata["columns"] == 3

    def test_csv_converter_with_tabs(self, tmp_data_dir: Path):
        """Tab‑separated values should be auto‑detected."""
        p = tmp_data_dir / "tsv_data.tsv"
        p.write_text("Name\tAge\nAlice\t30\nBob\t25\n")
        converter = CsvConverter()
        result = converter.convert(str(p))
        assert result.error is None
        assert "Alice" in result.text
        assert "Bob" in result.text

    def test_csv_converter_empty(self, tmp_data_dir: Path):
        """Empty CSV file."""
        p = tmp_data_dir / "empty.csv"
        p.write_text("")
        converter = CsvConverter()
        result = converter.convert(str(p))
        assert result.error is None

    def test_csv_converter_single_column(self, tmp_data_dir: Path):
        """Single‑column CSV."""
        p = tmp_data_dir / "single.csv"
        p.write_text("Item\nA\nB\nC\n")
        converter = CsvConverter()
        result = converter.convert(str(p))
        assert result.error is None
        assert "| Item" in result.text
        assert "| A" in result.text

    def test_csv_converter_nonexistent(self, nonexistent_path: Path):
        converter = CsvConverter()
        result = converter.convert(str(nonexistent_path))
        assert result.error == "File not found"

    def test_html_converter(self, sample_html_file: Path):
        """HTML → Markdown conversion."""
        converter = HtmlConverter()
        result = converter.convert(str(sample_html_file))
        assert result.error is None, f"HTML conversion error: {result.error_message}"
        assert "Hello World" in result.text
        assert result.title == "Test Page"

    def test_html_converter_without_title(self, tmp_data_dir: Path):
        """HTML without a <title> tag should fall back to the filename."""
        p = tmp_data_dir / "untitled.html"
        p.write_text("<html><body><p>No title.</p></body></html>")
        converter = HtmlConverter()
        result = converter.convert(str(p))
        assert result.error is None
        assert result.title == "untitled"

    def test_html_converter_strips_scripts(self, tmp_data_dir: Path):
        p = tmp_data_dir / "page.html"
        p.write_text(
            "<html><body><script>alert('xss')</script><p>Safe content.</p></body></html>"
        )
        converter = HtmlConverter()
        result = converter.convert(str(p))
        assert result.error is None
        assert "alert" not in result.text
        assert "Safe content" in result.text

    def test_html_converter_nonexistent(self, nonexistent_path: Path):
        converter = HtmlConverter()
        result = converter.convert(str(nonexistent_path))
        assert result.error == "File not found"

    def test_converter_accepts_mime(self):
        """CsvConverter should accept csv MIME types."""
        assert CsvConverter.accepts("text/csv")
        assert CsvConverter.accepts("text/comma-separated-values")
        assert not CsvConverter.accepts("text/html")

    def test_converter_accepts_extension(self):
        assert CsvConverter.accepts("text/plain", ".csv")
        assert HtmlConverter.accepts("text/plain", ".html")
        assert HtmlConverter.accepts("text/plain", ".xhtml")
        assert not HtmlConverter.accepts("text/plain", ".csv")

    def test_converter_accepts_no_match(self):
        assert not CsvConverter.accepts("application/pdf")

    def test_get_converter_for(self):
        from zarishnote_ingest.converters import get_converter_for
        cls = get_converter_for("text/csv")
        assert cls is CsvConverter
        cls = get_converter_for("text/html")
        assert cls is HtmlConverter
        cls = get_converter_for("application/unknown")
        assert cls is None

    def test_get_converter_for_url(self):
        from zarishnote_ingest.converters import get_converter_for_url
        cls = get_converter_for_url("https://www.youtube.com/watch?v=test")
        assert cls is not None
        assert cls.__name__ == "YoutubeConverter"
        cls = get_converter_for_url("https://en.wikipedia.org/wiki/Python")
        assert cls is not None
        assert cls.__name__ == "WikipediaConverter"
        cls = get_converter_for_url("https://example.com")
        assert cls is None
