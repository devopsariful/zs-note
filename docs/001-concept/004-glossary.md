# 004-glossary.md
## ZarishNote Glossary
### Definitions of all ZarishNote-specific terms

**Document type:** Reference — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## Core Terms

| Term | Definition |
|---|---|
| **ZarishNote** | The desktop/mobile Markdown editor application described in this blueprint |
| **zs-note** | The GitHub repository codename: `zarishsphere/zs-note` |
| **ZUSS** | ZarishSphere Universal Serialization Standard — naming and formatting rules for all files and repos |
| **`.znrc`** | ZarishNote Runtime Config — the workspace configuration file (YAML), analogous to `.zrc` in the broader ZarishSphere platform |
| **Vault** | A folder on disk that ZarishNote treats as a workspace. All notes, config, and assets live inside a vault |
| **Sandbox** | The Wasmtime WASM isolation layer in which all AI tools, MCP servers, and plugins execute |
| **Full-stack sandbox** | A sandbox that supports HTTP calls, scoped filesystem access, state, and tool calling — not just code isolation |
| **G2A Engine** | Guideline-to-Action Engine — the ZarishSphere system that converts standards into digital assets. ZarishNote uses a simplified G2A context layer |
| **Ingestion Engine** | The document converter (built on MarkItDown + extensions) that turns any file or URL into clean Markdown |
| **Converter Registry** | The priority-ordered list of converters inside the Ingestion Engine |
| **MCP** | Model Context Protocol — open standard (Anthropic → Linux Foundation) for AI tool connectivity |
| **MCP Server** | A process that exposes tools, resources, and prompts to an MCP client via JSON-RPC 2.0 |
| **MCP Client** | ZarishNote acts as an MCP client, connecting to servers to give the AI access to tools |
| **Knowledge Base** | A local RAG (Retrieval-Augmented Generation) corpus stored in LanceDB, queryable by the AI |
| **RAG** | Retrieval-Augmented Generation — the pattern of searching a local vector store for relevant context before querying the LLM |
| **Vector Store** | LanceDB embedded database that stores text embeddings for semantic search |
| **WASM** | WebAssembly — the compilation target used by Wasmtime for sandbox execution |
| **Wasmtime** | The Rust-native WebAssembly runtime used by ZarishNote for sandboxed tool execution |
| **Capability** | A declared permission a tool or plugin requests (e.g., `read:workspace`, `network:github.com`) |
| **AI Panel** | The right-side panel in ZarishNote where AI chat, tools, and templates live |
| **Template** | A reusable prompt stored in the vault and injectable into the AI panel |
| **Provider** | An AI service: OpenAI, Anthropic, Google Gemini, DeepSeek, Ollama, or custom endpoint |
| **Voice Engine** | The Whisper.cpp-backed transcription module inside ZarishNote |
| **Publish** | The act of pushing a Markdown document (or whole vault) to GitHub, a custom API, or RSS |
| **Auto-commit** | ZarishNote's Git engine feature that creates a commit on every save |
| **Phase 1** | The V1 MVP target: core editor + AI + sandbox + ingestion + MCP + voice + publish |
| **Phase 2** | Extended target: mobile optimization + image generation + collaboration + marketplace |

---

## ZUSS Naming Rules (applied to zs-note)

| Rule | Applied Example |
|---|---|
| Repository prefix | `zs-note` |
| All filenames | lowercase, hyphen-separated, 3-digit prefix |
| Workflow files | `[id]--[trigger]--[process].yml` |
| No `latest` Docker tags | Always pin: `zarishsphere/zs-note:v1.0.0` |
| Version during development | Everything is V1 until first production launch |
| After launch | Semantic versioning: `v1.0.0`, `v1.1.0`, etc. |

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*