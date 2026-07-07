# 003-knowledge-bases.md
## ZarishNote Knowledge Bases Specification
### Local RAG with LanceDB vector store

**Document type:** Specification — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## Table of Contents

1. [Overview](#1-overview)
2. [Architecture](#2-architecture)
3. [Indexing Pipeline](#3-indexing-pipeline)
4. [Query Pipeline](#4-query-pipeline)
5. [Chunking Strategies](#5-chunking-strategies)
6. [Embedding Model](#6-embedding-model)
7. [Knowledge Base Configuration](#7-knowledge-base-configuration)
8. [GUI: Knowledge Base Manager](#8-gui-knowledge-base-manager)

---

## 1. Overview

Knowledge Bases are indexed document collections that the AI can query via semantic search (RAG). They enable context-aware answers without requiring all documents to fit in the LLM context window.

Key characteristics:
- **Local only:** All embeddings and queries run on-device
- **No API required:** FastEmbed-rs runs ONNX models locally
- **Per-vault:** Each vault has its own vector store
- **Auto-indexing:** New files indexed automatically on vault open

---

## 2. Architecture

```
┌──────────────────────────────────────────────────────┐
│                  ZarishNote Vault                      │
│                                                        │
│  ┌──────────────┐    ┌────────────────────────────┐   │
│  │ Knowledge    │    │  .znrc-vectors/            │   │
│  │ Base Folder  │    │  ├── lancedb/              │   │
│  │ (files/*.md) │───→│  │   ├── tables/           │   │
│  └──────────────┘    │  │   └── index.idx         │   │
│                      │  └────────────────────────┘   │
│  ┌──────────────┐    │                              │
│  │ Ingestion    │───→│  Chunks → Embed → Store       │
│  │ Engine       │    │  (512 tokens, 50 overlap)     │
│  └──────────────┘    │                              │
│                      │   ↑ Query: top-5 chunks       │
│  ┌──────────────┐    │                              │
│  │ AI Panel     │────│─────────────────────────────  │
│  │ (query)      │    │                              │
│  └──────────────┘    └──────────────────────────────┘
```

---

## 3. Indexing Pipeline

### 3.1 Trigger Events

Indexing runs on these events:

| Event | Behavior |
|---|---|
| **Vault open** | Full scan: index all new/modified files |
| **File save** | Re-index the saved file |
| **File delete** | Remove deleted file's chunks from index |
| **File rename** | Re-index under new path, remove old |
| **Manual re-index** | User-initiated full re-index |

### 3.2 Pipeline Steps

```
1. Scan KB folder for changes (file path + modified timestamp)
2. For each new/modified file:
   a. If not Markdown → pass through Ingestion Engine
   b. Split into chunks (512 tokens, 50 overlap)
   c. Generate embedding via FastEmbed-rs
   d. Upsert into LanceDB table
3. For deleted files:
   a. Remove all chunks matching file path
4. Log indexing results (files indexed, chunks created, errors)
```

### 3.3 File Watch

- Uses `notify` (Rust crate) for filesystem events
- Debounced: 5s after last file change before re-index
- Ignores patterns from `.znrc` `sync.ignore_patterns`

---

## 4. Query Pipeline

### 4.1 Query Flow

```
1. AI decides it needs more context
2. ZarishNote detects relevant KB via keyword match or explicit KB tool call
3. Embed query string via FastEmbed-rs
4. LanceDB ANN search: find top-K nearest chunks (K=5 default)
5. Return chunks with similarity scores and source paths
6. Inject into AI context with citation markers
7. AI generates response citing sources
```

### 4.2 Query Parameters

```yaml
ai:
  context:
    rag:
      enabled: true
      top_k: 5
      similarity_threshold: 0.65    # ignore results below this score
      max_tokens: 2048              # max context tokens from RAG
```

### 4.3 Source Citations

Every RAG result includes source citation:

```markdown
Based on WHO guidelines (source: `knowledge/clinical/who-protocols.md`):
- Recommended treatment duration: 7-10 days
- First-line therapy: Artemether-lumefantrine
```

---

## 5. Chunking Strategies

| Strategy | Description | Best for |
|---|---|---|
| `fixed` | Fixed 512 tokens, no overlap | Simple documents |
| `semantic` (default) | Split at sentence/paragraph boundaries, respect semantic units | General prose |
| `hierarchical` | Document → section → chunk with parent references | Long structured documents |

Semantic chunking algorithm:

```
1. Parse document into sections (by heading level)
2. For each section:
   a. Split into sentences (using sentence tokenizer)
   b. Group sentences until ~512 tokens or natural break (heading)
   c. Overlap: include 50 tokens from previous chunk
3. Store chunk metadata: source file, section heading, chunk index
```

---

## 6. Embedding Model

| Field | Value |
|---|---|
| Model | `all-MiniLM-L6-v2` (ONNX, ~80MB) |
| Library | FastEmbed-rs (Rust, local ONNX runtime) |
| Dimension | 384 |
| Quantization | int8 (default) |
| Storage | LanceDB: IVF-PQ index, ~1KB per chunk |

### 6.1 Performance

| Corpus size | Index time | Query latency |
|---|---|---|
| 1,000 chunks | ~5s | <5ms |
| 10,000 chunks | ~30s | <10ms |
| 100,000 chunks | ~5min | <50ms |

Model files cached in app data directory after first download.

---

## 7. Knowledge Base Configuration

```yaml
# In .znrc
mcp:
  knowledge_bases:
    - name: "clinical-guidelines"
      path: "knowledge/clinical"
      description: "WHO and UNHCR clinical guidelines"
      formats: ["*.md", "*.pdf"]
      index_on_start: true
      auto_reindex: true

    - name: "project-context"
      path: "knowledge/project"
      formats: ["*.md", "*.txt", "*.docx"]
      index_on_start: true
      embedding_model: "all-MiniLM-L6-v2"   # per-KB override
```

Each knowledge base can have:
- `formats`: glob patterns for files to include
- `embedding_model`: override global embedding model per KB
- `reindex_interval`: schedule re-indexing (e.g., `3600s`)

---

## 8. GUI: Knowledge Base Manager

```
Settings → MCP → Knowledge Bases:

┌────────────────────────────────────────────────────────┐
│  Knowledge Bases                            [+ Add]    │
│                                                        │
│  📚 clinical-guidelines                                │
│     path: knowledge/clinical                           │
│     2,340 chunks indexed                               │
│     Last index: 2 minutes ago                          │
│     ☑ Auto-reindex     [Re-index now] [Browse] [×]    │
│                                                        │
│  📚 project-context                                    │
│     path: knowledge/project                            │
│     847 chunks indexed                                 │
│     Last index: 5 minutes ago                          │
│     ☑ Auto-reindex     [Re-index now] [Browse] [×]    │
│                                                        │
│  [Test Query...]                                       │
│                                                        │
│  Query: "Rohingya health guidelines 2026"              │
│  ┌────────────────────────────────────────────────┐    │
│  │ 1. WHO Emergency Health Response (0.89)        │    │
│  │    knowledge/clinical/who-response.md          │    │
│  │ 2. UNHCR Health Protocols (0.82)              │    │
│  │    knowledge/clinical/unhcr-protocols.md       │    │
│  └────────────────────────────────────────────────┘    │
└────────────────────────────────────────────────────────┘
```

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*
