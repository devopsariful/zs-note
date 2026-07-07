# 002-user-personas.md
## ZarishNote User Personas
### Who ZarishNote is for

**Document type:** Concept — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## 1. The Field Researcher (Amina)

**Age:** 32
**Role:** Public health researcher, Cox's Bazar field office
**Device:** 8GB RAM laptop, Ubuntu, intermittent internet

**Needs:**
- Take structured field notes that sync when connectivity allows
- Convert PDF/Word reports from WHO and NGO partners into clean Markdown
- Query notes via AI without sending data to cloud
- Publish findings to GitHub Pages for team visibility

**ZarishNote fit:**
- Offline-first: all features work without internet
- Ingestion engine: drops WHO PDFs into Markdown instantly
- Local RAG: AI queries her field notes without data leaving device
- Git sync: auto-commits every save, pushes when online

---

## 2. The Developer (Rafi)

**Age:** 28
**Role:** Full-stack developer, remote
**Device:** macOS, 16GB RAM

**Needs:**
- Write project documentation with live diagram rendering
- Execute code blocks inline (sandboxed) for quick tests
- Keep docs in Git alongside code
- Access AI for code review and explanation without leaving editor

**ZarishNote fit:**
- Mermaid/PlantUML/D2 diagrams render live from code blocks
- Sandboxed code execution via Wasmtime
- Git auto-commit: doc as code
- AI panel with multi-provider support for code tasks

---

## 3. The Clinician (Dr. Rahman)

**Age:** 45
**Role:** Senior medical officer, district hospital
**Device:** Windows laptop, 8GB RAM

**Needs:**
- Dictate clinical notes via voice
- Maintain patient records as Markdown (local, no cloud)
- Access clinical guidelines via AI with RAG
- Publish anonymized case reports to hospital knowledge base

**ZarishNote fit:**
- Voice engine: whisper.cpp for offline dictation
- Template system: SOAP note, case summary templates
- Knowledge bases: index clinical guidelines for AI retrieval
- Publishing: custom API push to hospital CMS

---

## 4. The Journalist (Nadia)

**Age:** 35
**Role:** Investigative journalist, covers humanitarian crises
**Device:** iPad + Bluetooth keyboard, mobile hotspot

**Needs:**
- Mobile Markdown editing with touch toolbar
- AI-assisted research: summarize web pages, YouTube transcripts
- Secure: no data sent to intermediary servers
- Publish stories directly to personal blog

**ZarishNote fit:**
- Tauri v2 mobile: iOS/Android native
- Ingestion engine: YouTube, Wikipedia, RSS → Markdown
- API keys in OS Keychain, no cloud intermediary
- GitHub publishing with RSS feed generation

---

## 5. The Student (Priya)

**Age:** 21
**Role:** Graduate student, public health
**Device:** Chromebook (Linux via Crostini), 4GB RAM

**Needs:**
- Free, lightweight Markdown editor for note-taking
- Math notation for statistics coursework
- AI summarization of papers
- Cross-device sync between laptop and phone

**ZarishNote fit:**
- 12MB installer, ~15MB RAM idle
- KaTeX math rendering
- Ollama integration for free local AI
- Git-based sync across devices

---

## 6. The Team Lead (Arif)

**Age:** 40
**Role:** Program manager, humanitarian NGO
**Device:** Linux desktop, 16GB RAM

**Needs:**
- Manage team documentation across projects
- AI-powered translation between Bangla, English, Arabic
- Sandboxed plugins for custom workflows
- MCP integration with team tools (GitHub, Slack, Google Drive)

**ZarishNote fit:**
- Multi-vault workspace switching
- AI translation via any configured LLM provider
- WASM plugin system with capability-based sandboxing
- MCP client connecting to GitHub, filesystem, and custom APIs

---

## 7. Non-Target User: Notion Power User

**Why ZarishNote is wrong for them:**
- No database/spreadsheet views
- No real-time collaboration
- No block-based editor
- No cloud sync by default (Git is the sync layer)

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*
