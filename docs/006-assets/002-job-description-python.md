# 002-job-description-python.md
## Job Description: Python/Ingestion Engine Developer
### ZarishNote — ZarishSphere Foundation

**Document type:** Asset — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0

---

## Position

**Title:** Python/Ingestion Engine Developer
**Type:** Contract (2–4 months, extendable)
**Location:** Remote
**Time commitment:** Part-time (15+ hrs/week)

---

## About ZarishNote

ZarishNote is a free, open-source, ultra-lightweight (~12MB) WYSIWYG Markdown editor with a sandboxed private AI assistant. The Ingestion Engine is its document conversion system — turning 20+ file formats and web sources into clean Markdown.

---

## Responsibilities

- Build and maintain the `zarishnote-ingest` Python CLI/library
- Extend Microsoft MarkItDown with ZarishNote-specific converters
- Implement web converters: YouTube transcript, Wikipedia, RSS, SERP
- Handle edge cases: corrupt files, charset detection, XXE prevention
- Write tests for each converter (unit + integration)
- Package and distribute via PyPI

---

## Requirements

### Required

- 3+ years Python experience
- Experience with document processing libraries (pdfminer, mammoth, python-pptx)
- Understanding of XML security (XXE prevention, defusedxml)
- Familiarity with CLI packaging (pyproject.toml, setuptools, pip)
- Cross-platform testing (Windows, macOS, Linux)

### Nice to Have

- Experience with MarkItDown library
- Knowledge of Whisper or other transcription tools
- YouTube transcript API experience
- Experience with Rust/Python interop (PyO3)

---

## Deliverables

| Item | Timeline |
|---|---|
| CLI scaffold + MarkItDown integration | Week 1 |
| PDF, DOCX, PPTX, XLSX converters | Week 1–2 |
| EPUB, CSV, Jupyter, ZIP, MSG converters | Week 2–3 |
| YouTube, Wikipedia, RSS, SERP converters | Week 3–4 |
| Edge case handling (charset, corrupt files, XXE) | Week 4–5 |
| Test suite + PyPI packaging | Week 5–6 |

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*
