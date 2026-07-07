# 004-data-model.md
## ZarishNote Data Model
### Local DB, vector store, Git history, and config

**Document type:** Architecture — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## Table of Contents

1. [Data Storage Overview](#1-data-storage-overview)
2. [Vault Storage on Disk](#2-vault-storage-on-disk)
3. [Vector Store (LanceDB)](#3-vector-store-lancedb)
4. [Full-Text Search Index](#4-full-text-search-index)
5. [Git History](#5-git-history)
6. [Config (.znrc)](#6-config-znrc)
7. [AI Conversation History](#7-ai-conversation-history)

---

## 1. Data Storage Overview

ZarishNote uses **no cloud database**. All data is stored locally, on the user's filesystem:

| Data type | Storage | Location |
|---|---|---|
| Documents | Plain Markdown files | Vault root |
| Config | YAML file | `.znrc` at vault root |
| Vector embeddings | LanceDB table | `.znrc-vectors/` |
| Full-text search | SQLite FTS5 | `.znrc-search/` |
| Git history | Git objects | `.git/` |
| AI conversation history | JSONL files | `.znrc-history/` |
| Plugin state | JSON files | `.znrc-state/{tool}/` |
| Audit log | Plain text | `.znrc-audit.log` |
| API keys | OS Keychain | Platform-specific secure storage |

---

## 2. Vault Storage on Disk

### 2.1 Document Storage

Documents are plain Markdown files (`.md`) organized in folders. No database, no proprietary format.

```markdown
# Title

**YAML front matter** (optional):
---
title: "My Document"
date: 2026-06-08
tags: [health, research]
---

Body content here.
```

### 2.2 Metadata Extraction

On file open, ZarishNote parses:
- YAML front matter → structured metadata (title, date, tags, custom fields)
- First H1 → document title (fallback if no `title` in front matter)
- Internal links → `[[wikilinks]]` resolved to vault paths

### 2.3 Binary File Handling

Binary files (images, PDFs, audio) stored in vault folders:
- Images in `assets/`
- Recordings in `recordings/`
- Imported documents in `inbox/` (converted to Markdown)

Binary files are not edited inside ZarishNote — they are opened with the system default application.

---

## 3. Vector Store (LanceDB)

### 3.1 Schema

```rust
struct Chunk {
    id: String,                    // UUID
    file_path: String,             // Relative to vault root
    chunk_index: u32,              // Position within document
    section_heading: Option<String>, // Heading context
    token_count: u32,
    text: String,                  // Original chunk text
    embedding: Vec<f32>,           // 384-dim float32 vector
    metadata: HashMap<String, String>, // File metadata
    created_at: i64,               // Unix timestamp
    updated_at: i64,
}
```

### 3.2 Index Configuration

- Index type: IVF-PQ (Inverted File with Product Quantization)
- Number of partitions: auto (sqrt of dataset size)
- Metric: cosine similarity
- Quantization: PQ4 (4-bit product quantization) for memory efficiency

### 3.3 Storage

```
.znrc-vectors/
└── lancedb/
    ├── _version/
    ├── chunks.lance/          # Main table
    └── chunks.idx/            # ANN index
```

Typical storage: ~1KB per chunk (after quantization).

---

## 4. Full-Text Search Index

### 4.1 Implementation

- SQLite FTS5 (Full-Text Search v5)
- Content table + FTS virtual table
- Updated on file save (real-time)

### 4.2 Schema

```sql
CREATE VIRTUAL TABLE files_fts USING fts5(
    path,
    title,
    content,
    tags,
    content=files,
    tokenize='unicode61 remove_diacritics 2'
);

CREATE TABLE files (
    id INTEGER PRIMARY KEY,
    path TEXT UNIQUE,
    title TEXT,
    content TEXT,
    tags TEXT,
    modified_at INTEGER
);
```

### 4.3 Features

- Unicode tokenizer with diacritic removal
- Prefix queries (type "hea" → matches "health")
- Ranking by BM25
- Highlighted match excerpts in results

---

## 5. Git History

### 5.1 Commit Structure

```
docs: update report.md

Summary: Modified section 2 (+120 characters)
```

ZarishNote does not alter standard Git commit objects. The Git repository is fully compatible with standard Git tooling.

### 5.2 Gitignore (.znrc)

The auto-generated `.gitignore` includes:

```
.znrc-vectors/
.znrc-plugins/
.znrc-state/
.znrc-history/
.znrc-search/
.znrc-audit.log
*.lock
.DS_Store
```

### 5.3 History Queries

ZarishNote queries Git history via git2-rs:

| Query | Method |
|---|---|
| File history | `git log --follow <path>` |
| File at revision | `git show <rev>:<path>` |
| Diff between versions | `git diff <rev1> <rev2> -- <path>` |
| Vault-wide changes | `git log --since=1d` |

---

## 6. Config (.znrc)

### 6.1 Format

- YAML (`.znrc` at vault root)
- Full schema defined in `003-znrc-schema.md`
- Hot-reloaded on file change (file watcher)

### 6.2 Validation

On load, the Rust backend validates:
- Schema version
- Field types (string, bool, number, arrays)
- Referential integrity (file paths exist)
- No API key strings stored in config

### 6.3 Backup

`.znrc` is part of the vault and tracked in Git. A backup of the previous valid config is kept at `.znrc.bak`.

---

## 7. AI Conversation History

### 7.1 Storage Format

Each AI conversation session is stored as JSONL (one JSON object per line):

```jsonl
{"role": "user", "content": "Summarize this document", "timestamp": "2026-06-08T14:32:05Z"}
{"role": "assistant", "content": "Here is a summary...", "model": "claude-sonnet-4-6", "timestamp": "2026-06-08T14:32:12Z"}
```

### 7.2 File Naming

`.znrc-history/{date}-{provider}-{session-id}.jsonl`

- `date`: ISO date of first message
- `provider`: provider name (e.g., `claude`)
- `session-id`: UUID

### 7.3 Retention

| Policy | Default | Configurable |
|---|---|---|
| Max age | 30 days | Yes (in `.znrc`) |
| Max sessions | 100 | Yes |
| Auto-cleanup | Weekly | Yes |

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*
