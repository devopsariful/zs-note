# 002-format-matrix.md
## ZarishNote Ingestion Engine — Supported Format Matrix

**Document type:** Reference — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## Document Formats

| Format | Extension | Library | Strategy | Tables | Images | Metadata | Phase |
|---|---|---|---|---|---|---|---|
| PDF (text) | `.pdf` | `pdfminer.six` | Text extraction, layout hints | via pdfplumber | Alt text | Author, date, title | V1 |
| PDF (scanned) | `.pdf` | `pdfminer.six` + OCR | Fallback to image → OCR | ❌ | OCR text | Limited | Phase 2 |
| Word | `.docx` | `mammoth` | DOCX → HTML → Markdown | ✅ | Extracted to assets/ | Author, title, date | V1 |
| Legacy Word | `.doc` | `antiword` (system) | Text extraction | Limited | ❌ | Limited | Phase 2 |
| PowerPoint | `.pptx` | `python-pptx` | Slide titles + body + tables | ✅ | Extracted to assets/ | Author, title | V1 |
| Excel | `.xlsx` | `pandas` + `openpyxl` | Each sheet = Markdown table | N/A | ❌ | Sheet names | V1 |
| Legacy Excel | `.xls` | `pandas` + `xlrd` | Same as XLSX | N/A | ❌ | Sheet names | V1 |
| EPUB | `.epub` | `zipfile` + `lxml` | Unpack → spine order → chapter HTML | ✅ | Extracted | Title, author, TOC | V1 |
| CSV | `.csv` | stdlib `csv` | Render as Markdown table | N/A | ❌ | None | V1 |
| TSV | `.tsv` | stdlib `csv` | Same as CSV | N/A | ❌ | None | V1 |
| Jupyter Notebook | `.ipynb` | stdlib `json` | Code cells → fenced blocks, outputs → text | ✅ | Inline | Kernel, language | V1 |
| Outlook Message | `.msg` | `olefile` | Extract headers (To, From, Subject, Date) + body | ❌ | ❌ | Email headers | V1 |
| ZIP Archive | `.zip` | stdlib `zipfile` | Recursively convert each member | Depends | Depends | Archive metadata | V1 |
| Plain text | `.txt`, `.md` | stdlib | Pass-through | ✅ | ✅ | None | V1 |
| HTML | `.html`, `.htm` | `markdownify` | HTML → Markdown | ✅ | Extract | Title, meta | V1 |
| RTF | `.rtf` | `striprtf` | Text extraction | Limited | ❌ | None | Phase 2 |
| ODT (LibreOffice) | `.odt` | `odfpy` | HTML export → Markdown | ✅ | Extracted | Author, title | Phase 2 |
| Markdown | `.md` | Pass-through | Already Markdown | ✅ | ✅ | Front matter | V1 |

---

## Web Sources

| Source | URL Pattern | Strategy | Transcript | Images | Phase |
|---|---|---|---|---|---|
| YouTube | `youtube.com/watch?v=*`, `youtu.be/*` | Title + description + transcript (if `youtube-transcript-api`) | ✅ optional | Thumbnail URL | V1 |
| Wikipedia | `*.wikipedia.org/wiki/*` | Title + summary + structured body via Wikipedia API | N/A | Infobox image URL | V1 |
| Bing SERP | `bing.com/search?q=*` | Result titles, snippets, URLs as Markdown list | N/A | N/A | V1 |
| RSS Feed | Any RSS/Atom URL | Feed title + each entry as section | N/A | Featured image | V1 |
| Generic HTML | Any URL | `requests` → `markdownify` with script/style stripping | N/A | Inline images | V1 |
| GitHub README | `github.com/*/blob/*README*` | Fetch raw Markdown | N/A | ✅ | V1 |
| arXiv | `arxiv.org/abs/*` | Title, abstract, authors via arXiv API | N/A | N/A | Phase 2 |
| Twitter/X | `twitter.com/*/status/*` | Tweet text + metadata | N/A | N/A | Phase 2 |

---

## Media Formats

| Format | Strategy | Metadata | Transcript/Description | Phase |
|---|---|---|---|---|
| JPEG/PNG/GIF/WebP | EXIF via `exiftool` (system) | Camera, date, GPS | AI vision description (optional) | V1 |
| SVG | Inline as Markdown code block | None | N/A | V1 |
| MP3/WAV/M4A | EXIF metadata | Duration, bitrate, artist | Whisper transcription (optional) | Phase 2 |
| MP4/MOV | EXIF metadata | Duration, resolution | Audio track → Whisper (optional) | Phase 2 |
| WebM | EXIF metadata | Duration | Audio track → Whisper (optional) | Phase 2 |

---

## Output Format Details

### PDF Table Detection

`pdfplumber` detects tables in PDFs and renders them as GFM Markdown tables:
```
| Column A | Column B |
|---|---|
| Value    | Value    |
```

If table detection confidence is low (<70%), falls back to plain text with a `> ⚠️ Table structure uncertain` note.

### XLSX Sheet Handling

Each worksheet becomes a top-level Markdown section:
```markdown
## Sheet 1: Patient Data

| ID | Name | Age |
|---|---|---|
| 001 | ... | ... |

## Sheet 2: Summary

...
```

### PPTX Slide Handling

Each slide becomes a section:
```markdown
## Slide 1: Introduction

Body text here.

> Speaker notes: ...

### Table

| Col A | Col B |
|---|---|
```

### Jupyter Notebook Handling

```markdown
## Cell 1 (code)

```python
import pandas as pd
df = pd.read_csv("data.csv")
```

**Output:**
```
   col_a  col_b
0      1      2
```

## Cell 2 (markdown)

This is a markdown cell rendered as-is.
```

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*