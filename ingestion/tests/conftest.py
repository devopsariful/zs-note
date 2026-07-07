"""Shared fixtures for the ZarishNote ingestion test suite."""

from __future__ import annotations

import csv
import textwrap
from pathlib import Path
from typing import Iterator

import pytest


@pytest.fixture
def tmp_data_dir(tmp_path: Path) -> Path:
    """A temporary directory for test data files."""
    d = tmp_path / "data"
    d.mkdir(parents=True, exist_ok=True)
    return d


@pytest.fixture
def sample_markdown_file(tmp_data_dir: Path) -> Path:
    """Create a simple Markdown file for conversion tests."""
    p = tmp_data_dir / "sample.md"
    p.write_text(
        textwrap.dedent("""\
        # Sample Document

        This is a paragraph with **bold** and *italic* text.

        - List item one
        - List item two
        """)
    )
    return p


@pytest.fixture
def sample_html_file(tmp_data_dir: Path) -> Path:
    """Create a simple HTML file for HTML conversion tests."""
    p = tmp_data_dir / "sample.html"
    p.write_text(
        textwrap.dedent("""\
        <!DOCTYPE html>
        <html>
        <head><title>Test Page</title></head>
        <body>
          <h1>Hello World</h1>
          <p>This is a <strong>test</strong> paragraph.</p>
        </body>
        </html>
        """)
    )
    return p


@pytest.fixture
def sample_csv_file(tmp_data_dir: Path) -> Path:
    """Create a simple CSV file."""
    p = tmp_data_dir / "sample.csv"
    with open(p, "w", newline="") as f:
        writer = csv.writer(f)
        writer.writerow(["Name", "Age", "City"])
        writer.writerow(["Alice", "30", "New York"])
        writer.writerow(["Bob", "25", "London"])
    return p


@pytest.fixture
def empty_file(tmp_data_dir: Path) -> Path:
    """An empty plain-text file."""
    p = tmp_data_dir / "empty.txt"
    p.write_text("")
    return p


@pytest.fixture
def utf16_file(tmp_data_dir: Path) -> Path:
    """A UTF-16 encoded text file."""
    p = tmp_data_dir / "utf16.txt"
    p.write_bytes("Hello UTF-16\n".encode("utf-16-le"))
    return p


@pytest.fixture
def binary_file(tmp_data_dir: Path) -> Path:
    """A binary file (non-text)."""
    p = tmp_data_dir / "data.bin"
    p.write_bytes(b"\x00\x01\x02\x03\xff\xfe\xfd\xfc")
    return p


@pytest.fixture
def nonexistent_path(tmp_data_dir: Path) -> Path:
    """A path that does not exist on disk."""
    return tmp_data_dir / "does_not_exist.md"


@pytest.fixture
def sample_text_file(tmp_data_dir: Path) -> Path:
    """A plain text file for charset detection tests."""
    p = tmp_data_dir / "hello.txt"
    p.write_text("Hello, ZarishNote!")
    return p
