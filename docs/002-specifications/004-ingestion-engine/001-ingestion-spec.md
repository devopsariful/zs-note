# 001-ingestion-spec.md
## ZarishNote Ingestion Engine Specification
### Universal document-to-Markdown converter

**Document type:** Specification — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## Table of Contents

1. [Overview](#1-overview)
2. [Converter Registry](#2-converter-registry)
3. [MIME Detection Strategy](#3-mime-detection-strategy)
4. [Python API](#4-python-api)
5. [CLI Interface](#5-cli-interface)
6. [Error Handling](#6-error-handling)
7. [Charset and Security](#7-charset-and-security)
8. [Dependency Management](#8-dependency-management)
9. [Integration with ZarishNote Desktop](#9-integration-with-zarishnote-desktop)

---

## 1. Overview

ZarishNote's Ingestion Engine is built on **Microsoft's MarkItDown** (MIT license, 139K+ GitHub stars, v0.1.x) as its primary conversion backbone, extended with:
- Web content converters (YouTube, Wikipedia, RSS, SERP)
- Vision descriptions for images (via LLM)
- Audio transcription (via Whisper)
- ZarishNote-specific output formatting

The engine is exposed as:
- A Python library (`zarishnote-ingest`)
- A CLI (`zarishnote-ingest <file>`)
- A Tauri command (`ingest_file`) callable from the Rust backend

**Implementation language:** Python 3.10+ (CLI/library), called from Rust via subprocess or embedded Python (PyO3).

---

## 2. Converter Registry

The registry is a priority-ordered list of converters. Each converter:
- Declares a `priority` (float)
- Implements `accepts(file_path, mime_type) -> bool`
- Implements `convert(file_path, context) -> ConversionResult`

### 2.1 Priority Order

| Priority | Converter type | Example |
|---|---|---|
| `-1.0` | Custom plugins (override built-ins) | User-defined converters |
| `0.0` | Specific format converters | PDFConverter, DOCXConverter |
| `5.0` | Web content converters | YouTubeConverter, WikipediaConverter |
| `10.0` | Generic fallbacks | HTMLConverter, PlainTextConverter |

### 2.2 Built-in Converters

| Converter | Priority | MIME types handled |
|---|---|---|
| `PDFConverter` | 0.0 | `application/pdf` |
| `DOCXConverter` | 0.0 | `application/vnd.openxmlformats-officedocument.wordprocessingml.document` |
| `PPTXConverter` | 0.0 | `application/vnd.openxmlformats-officedocument.presentationml.presentation` |
| `XLSXConverter` | 0.0 | `application/vnd.openxmlformats-officedocument.spreadsheetml.sheet` |
| `XLSConverter` | 0.0 | `application/vnd.ms-excel` |
| `EPUBConverter` | 0.0 | `application/epub+zip` |
| `CSVConverter` | 0.0 | `text/csv` |
| `JupyterConverter` | 0.0 | `application/x-ipynb+json` |
| `OutlookMSGConverter` | 0.0 | `application/vnd.ms-outlook` |
| `ZIPConverter` | 0.0 | `application/zip` |
| `YouTubeConverter` | 5.0 | URL: `youtube.com/watch` |
| `WikipediaConverter` | 5.0 | URL: `*.wikipedia.org/wiki/*` |
| `RSSConverter` | 5.0 | `application/rss+xml`, `application/atom+xml` |
| `BingSERPConverter` | 5.0 | URL: `bing.com/search` |
| `ImageConverter` | 5.0 | `image/*` |
| `AudioConverter` | 5.0 | `audio/*` |
| `HTMLConverter` | 10.0 | `text/html` (generic fallback) |
| `PlainTextConverter` | 10.0 | `text/plain` (last resort) |

### 2.3 Converter Interface (Python)

```python
from abc import ABC, abstractmethod
from dataclasses import dataclass
from typing import Optional

@dataclass
class ConversionResult:
    markdown: str
    title: Optional[str] = None
    metadata: dict = None
    source_url: Optional[str] = None
    error: bool = False
    error_message: Optional[str] = None

class BaseConverter(ABC):
    priority: float = 0.0

    @abstractmethod
    def accepts(self, source: str, mime_type: Optional[str] = None) -> bool:
        """Returns True if this converter can handle the input."""
        pass

    @abstractmethod
    def convert(self, source: str, **kwargs) -> ConversionResult:
        """Convert source to Markdown. source can be path or URL."""
        pass
```

---

## 3. MIME Detection Strategy

Detection is attempted in order:

1. **Explicit hint** — `--mime` CLI flag or `mime_type` API argument
2. **URL pattern** — if source is a URL, match against known URL patterns (YouTube, Wikipedia, etc.)
3. **File extension** — map `.pdf` → `application/pdf`
4. **Magic bytes** — use `python-magic` if installed, else lightweight byte inspection
5. **Content heuristic** — read first 4KB, attempt text/binary detection

---

## 4. Python API

```python
from zarishnote_ingest import MarkItDown

# Basic usage
mid = MarkItDown()

# Convert local file
result = mid.convert_local("report.docx")
print(result.markdown)
print(result.title)        # "Quarterly Report 2026"
print(result.metadata)     # {"author": "...", "created": "..."}

# Convert URL
result = mid.convert_uri("https://en.wikipedia.org/wiki/Rohingya_people")

# Convert stream (for server contexts — most secure)
with open("data.pdf", "rb") as f:
    result = mid.convert_stream(f, mime_type="application/pdf")

# Convert with vision description (requires LLM client)
from openai import OpenAI
mid_vision = MarkItDown(
    vision_client=OpenAI(),
    vision_model="gpt-4o"
)
result = mid_vision.convert_local("diagram.png")
# result.markdown includes AI-generated description

# Convert response object (for HTTP contexts)
import requests
response = requests.get("https://example.com/doc.pdf")
result = mid.convert_response(response)

# List available converters
for converter in mid.list_converters():
    print(f"{converter.name} (priority={converter.priority})")
```

### 4.1 ConversionResult Fields

| Field | Type | Description |
|---|---|---|
| `markdown` | str | The converted Markdown text |
| `title` | str? | Extracted document title |
| `metadata` | dict? | Author, date, keywords, etc. |
| `source_url` | str? | Original URL (for web sources) |
| `error` | bool | True if conversion partially failed |
| `error_message` | str? | Description of any error |

---

## 5. CLI Interface

```bash
# Basic conversion (output to stdout)
zarishnote-ingest document.pdf

# Output to file
zarishnote-ingest document.pdf --output document.md

# Force MIME type
zarishnote-ingest data --mime application/pdf

# Specify charset
zarishnote-ingest legacy.txt --charset windows-1252

# Use extension hint when file has no extension
zarishnote-ingest attachment --ext docx

# Show all installed converters
zarishnote-ingest --list-converters

# Enable vision descriptions (needs API key in environment)
zarishnote-ingest image.jpg --vision-model gpt-4o

# Convert a URL
zarishnote-ingest "https://www.youtube.com/watch?v=dQw4w9WgXcQ"

# Disable plugins (use only built-ins)
zarishnote-ingest document.pdf --no-plugins

# Read from stdin
cat document.html | zarishnote-ingest --ext html
```

---

## 6. Error Handling

### 6.1 Missing Dependencies

Each converter catches `ImportError` on import and provides a clear message:

```python
try:
    import pdfminer
except ImportError:
    raise MissingDependencyError(
        converter="PDFConverter",
        message="PDF conversion requires pdfminer.six.",
        install_command="pip install 'zarishnote-ingest[pdf]'"
    )
```

### 6.2 Corrupt or Unsupported Files

- Converter catches the exception
- Returns `ConversionResult(error=True, error_message=..., markdown="")`
- ZarishNote shows a user-facing error notification, not a crash
- User can try a different converter via "Force converter" option

### 6.3 Recursion in Deep HTML

markdownify can hit Python's recursion limit on deeply nested HTML:

```python
try:
    markdown = markdownify(html)
except RecursionError:
    import warnings
    warnings.warn(
        "HTML too deeply nested for markdownify. Falling back to BeautifulSoup text extraction.",
        stacklevel=2
    )
    from bs4 import BeautifulSoup
    markdown = BeautifulSoup(html, "html.parser").get_text(separator="\n")
```

---

## 7. Charset and Security

### 7.1 Charset Detection

Order of priority:
1. Explicit `--charset` flag or `charset` argument
2. HTTP `Content-Type` header (for URL sources)
3. HTML `<meta charset>` tag
4. `charset-normalizer` library (statistical detection)
5. Fallback: UTF-8

### 7.2 XML Security

All XML/HTML parsing uses `defusedxml` to prevent XXE (XML External Entity) injection:

```python
import defusedxml.ElementTree as ET
# Never: import xml.etree.ElementTree as ET  ← XXE vulnerable
```

### 7.3 Base64 Data URI Handling

In HTML-to-Markdown conversion:
- Base64 data URIs (images) are **truncated by default** (replaced with `[base64 image]`)
- Full preservation available via `--preserve-base64` flag or `preserve_base64=True` API argument
- This prevents bloated Markdown output that's unreadable by LLMs

---

## 8. Dependency Management

Core (no extras needed):
- `markitdown[all]` — Microsoft's converter (MIT)
- `defusedxml` — XML security
- `charset-normalizer` — encoding detection
- `markdownify` — HTML to Markdown

Optional extras (installed on demand):

| Extra | Packages | Activates |
|---|---|---|
| `[pdf]` | `pdfminer.six`, `pdfplumber` | PDF text + table extraction |
| `[docx]` | `mammoth` | DOCX → HTML → Markdown |
| `[pptx]` | `python-pptx` | PPTX slide extraction |
| `[xlsx]` | `pandas`, `openpyxl` | XLSX/XLS → Markdown table |
| `[xls]` | `pandas`, `xlrd` | Legacy XLS |
| `[epub]` | `ebooklib` | EPUB chapter extraction |
| `[notebook]` | none extra | JSON parsing only |
| `[msg]` | `olefile` | Outlook .msg files |
| `[audio]` | `SpeechRecognition` | Audio transcription |
| `[youtube]` | `youtube-transcript-api` | YouTube transcript |
| `[vision]` | `openai` (or other) | Image AI description |
| `[all]` | All of the above | Everything |

---

## 9. Integration with ZarishNote Desktop

The Python ingestion engine is invoked from Rust via subprocess:

```rust
// src-tauri/src/ingest.rs
#[tauri::command]
pub async fn ingest_file(
    source: String,
    output_path: Option<String>,
    mime_hint: Option<String>,
) -> Result<IngestResult, String> {
    let mut cmd = Command::new("zarishnote-ingest");
    cmd.arg(&source);
    if let Some(mime) = mime_hint {
        cmd.args(["--mime", &mime]);
    }
    cmd.arg("--output-json");  // returns JSON to stdout

    let output = cmd.output().await
        .map_err(|e| format!("Ingest engine not found: {}", e))?;

    let result: IngestResult = serde_json::from_slice(&output.stdout)
        .map_err(|e| format!("Parse error: {}", e))?;

    Ok(result)
}
```

### 9.1 GUI Integration

- Drag-and-drop file onto editor window → auto-ingestion dialog
- "Import" button in file manager sidebar
- URL paste detection → "Convert to Markdown?" prompt
- Bulk import: select multiple files → converts all to vault `/inbox/` folder

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*