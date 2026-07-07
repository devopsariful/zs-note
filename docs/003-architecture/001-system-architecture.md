# 001-system-architecture.md
## ZarishNote System Architecture
### Tauri layers, IPC, data flow, and process model

**Document type:** Architecture — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## Table of Contents

1. [High-Level Architecture](#1-high-level-architecture)
2. [Rust Backend Layers](#2-rust-backend-layers)
3. [Frontend Layers](#3-frontend-layers)
4. [IPC Communication](#4-ipc-communication)
5. [Data Flow](#5-data-flow)
6. [Process Model](#6-process-model)
7. [Startup Sequence](#7-startup-sequence)

---

## 1. High-Level Architecture

```
┌────────────────────────────────────────────────────────────────┐
│                    ZarishNote Application                        │
│                                                                  │
│  ┌──────────────────────┐     ┌──────────────────────────────┐  │
│  │   Frontend (WebView)  │     │      Rust Backend (Tauri)     │  │
│  │                       │     │                              │  │
│  │  ┌─────────────────┐  │ IPC │  ┌────────────────────────┐  │  │
│  │  │ Svelte 5 UI     │──┼─────┼─→│ Plugin: Editor         │  │  │
│  │  │ Milkdown Editor │  │     │  │ (Milkdown protocol)    │  │  │
│  │  │ AI Panel UI     │  │     │  ├────────────────────────┤  │  │
│  │  │ File Manager UI │  │     │  │ Plugin: Sandbox        │  │  │
│  │  └─────────────────┘  │     │  │ (Wasmtime manager)     │  │  │
│  │                       │     │  ├────────────────────────┤  │  │
│  │  ┌─────────────────┐  │     │  │ Plugin: Ingestion      │  │  │
│  │  │ Vite Dev Server │  │     │  │ (Python subprocess)    │  │  │
│  │  │ (dev only)      │  │     │  ├────────────────────────┤  │  │
│  │  └─────────────────┘  │     │  │ Plugin: Git Engine     │  │  │
│  └──────────────────────┘     │  │ (libgit2)               │  │  │
│                               │  ├────────────────────────┤  │  │
│                               │  │ Plugin: Voice           │  │  │
│                               │  │ (Whisper.cpp FFI)       │  │  │
│                               │  ├────────────────────────┤  │  │
│                               │  │ Plugin: MCP Client      │  │  │
│                               │  │ (JSON-RPC 2.0)          │  │  │
│                               │  └────────────────────────┘  │  │
│                               │                              │  │
│                               │  ┌────────────────────────┐  │  │
│                               │  │ Service: Config (.znrc) │  │  │
│                               │  │ Service: Vector Store   │  │  │
│                               │  │ (LanceDB)               │  │  │
│                               │  │ Service: Keychain       │  │  │
│                               │  │ (Tauri plugin)          │  │  │
│                               │  └────────────────────────┘  │  │
│                               └──────────────────────────────┘  │
└────────────────────────────────────────────────────────────────┘
```

---

## 2. Rust Backend Layers

### 2.1 Tauri Core

- Window management, menu, tray
- File system access (with capability checks)
- Shell/process spawning (for MCP stdio, ingestion)
- Global shortcuts

### 2.2 Editor Plugin

- Manages Milkdown/ProseMirror state bridge
- File save/load coordination
- Document format conversion (Markdown → ProseMirror JSON → DOM)

### 2.3 Sandbox Plugin

- Wasmtime engine singleton
- Tool registry (which tools are available, their capabilities)
- Execution queue (serial per-tool, parallel across different tools)
- Audit logging

### 2.4 Ingestion Plugin

- Spawns/manages Python `zarishnote-ingest` subprocess
- Pipes file → conversion → Markdown output
- Handles missing dependency errors gracefully

### 2.5 Git Engine Plugin

- `git2-rs` wrapper for auto-commit, history, diff
- Remote push/pull management
- SSH agent session management

### 2.6 Voice Plugin

- Whisper.cpp FFI bindings
- Audio capture via Tauri audio plugin
- Streaming transcription with sentence buffering

### 2.7 MCP Client Plugin

- stdio subprocess management for MCP servers
- HTTP transport for remote MCP servers
- JSON-RPC 2.0 message framing
- Tool discovery and routing

### 2.8 Shared Services

| Service | Responsibility |
|---|---|
| **Config** | Load, validate, watch `.znrc` (hot-reload on file change) |
| **Vector Store** | LanceDB connection pool, embedding, ANN search |
| **Keychain** | Unified interface to OS keychain via Tauri plugin |
| **Logger** | Structured logging to file + console |

---

## 3. Frontend Layers

### 3.1 Svelte 5 Application

- Component tree managed by Svelte 5 runes
- State management via Svelte stores (no Redux)
- Routing: simple view state (editor, settings, about) — no URL router needed

### 3.2 Component Hierarchy

```
App.svelte
├── Sidebar.svelte          (file tree, search, tags, history)
│   ├── FileTree.svelte
│   ├── Search.svelte
│   └── TagList.svelte
├── Editor.svelte           (editor viewport)
│   ├── MilkdownEditor.svelte  (WYSIWYG instance)
│   ├── SourceEditor.svelte   (CodeMirror/Monaco alternative for source mode)
│   └── SplitEditor.svelte    (side-by-side)
├── AIPanel.svelte          (right-side AI panel)
│   ├── ChatHistory.svelte
│   ├── TemplatePicker.svelte
│   └── ContextInspector.svelte
├── StatusBar.svelte        (bottom status bar)
└── Modal.svelte            (settings, dialogs)
```

### 3.3 Milkdown Integration

- Milkdown v7 (ProseMirror-based) creates editor instance
- Configured with: CommonMark nodes, GFM extension, math, diagrams
- Custom ZarishNote node: executable code blocks, AI action buttons
- Communication with Rust backend via Tauri commands

---

## 4. IPC Communication

### 4.1 Tauri Commands (Invoke)

Frontend → Backend calls:
```
editor.save()           → cmd: save_file(path, content)
file.list()             → cmd: list_files(path)
ai.chat()               → cmd: ai_chat(messages, provider)
sandbox.execute()       → cmd: sandbox_execute(tool, args)
ingest.file()           → cmd: ingest_file(source, opts)
sync.commit()           → cmd: git_commit()
voice.transcribe()      → cmd: transcribe_audio(path)
```

### 4.2 Tauri Events (Backend → Frontend)

```
event: file-changed     → { path, type: "modified" }
event: ai-token         → { token: "Hello" }
event: ai-done          → { full_response }
event: sync-status      → { status: "pushing" }
event: indexing-progress → { kb: "clinical", progress: 0.45 }
event: error            → { code, message }
```

### 4.3 Security Boundary

All IPC is typed. The frontend cannot:
- Access the filesystem directly (must go through Tauri commands)
- Access the OS keychain
- Spawn subprocesses
- Access network (must use sandbox/ingestion commands)

---

## 5. Data Flow

### 5.1 File Open

```
1. User clicks file in sidebar
2. Svelte component invokes cmd: read_file(path)
3. Rust backend reads file from disk
4. If .md: returns as string
5. If binary: returns error with "cannot edit binary" message
6. Frontend sets Milkdown content
7. File path stored as "active document" in Svelte store
```

### 5.2 AI Chat

```
1. User types message in AI panel
2. Frontend gathers context: current doc, selection, RAG results
3. Frontend invokes cmd: ai_chat(messages, context, provider)
4. Rust backend builds request, sends to provider API
5. Tokens streamed back via event: ai-token
6. Frontend appends tokens to chat display in real-time
7. On completion: action buttons (Insert, Replace, Copy) shown
```

### 5.3 Auto-Save

```
1. Milkdown emits "updated" event
2. Debounce (2s)
3. Frontend invokes cmd: save_file(path, content)
4. Rust backend writes to disk
5. Rust backend triggers git add + commit
6. Rust backend emits event: file-changed
7. Frontend updates status bar: "Saved"
```

---

## 6. Process Model

| Process | Count | Description |
|---|---|---|
| ZarishNote main | 1 | Tauri window process (Rust + WebView) |
| Ingestion engine | 0-1 | Python subprocess, spawned on demand |
| MCP servers | 0-N | One subprocess per enabled stdio MCP server |
| Voice engine | 0-1 | Whisper.cpp loaded as native library (no subprocess) |
| Git operations | 0-1 | git2-rs in-thread (non-blocking) |

All non-Rust code runs in subprocesses or the Wasmtime sandbox.

---

## 7. Startup Sequence

```
1. Tauri creates main window
2. Rust backend loads:
   a. Config (read + validate .znrc)
   b. Vector store (open LanceDB connection)
   c. Git engine (check .git exists, init if needed)
   d. Sandbox engine (instantiate Wasmtime)
3. Frontend loads:
   a. Svelte app mounts
   b. File tree populated via cmd: list_files()
   c. Editor initialized with last active file
   d. AI panel ready (no default provider connection)
4. Post-startup:
   a. MCP servers started (async, per configuration)
   b. Knowledge base indexing begins (async)
   c. Git status check (pending push, conflict detection)
5. App ready — status bar shows "Ready"
```

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*
