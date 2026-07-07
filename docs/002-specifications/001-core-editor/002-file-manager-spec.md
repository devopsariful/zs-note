# 002-file-manager-spec.md
## ZarishNote File Manager Specification
### Sidebar, file tree, search, tags, and vault management

**Document type:** Specification — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## Table of Contents

1. [Sidebar Layout](#1-sidebar-layout)
2. [File Tree](#2-file-tree)
3. [Tags](#3-tags)
4. [Full-Text Search](#4-full-text-search)
5. [File Operations](#5-file-operations)
6. [Vault Management](#6-vault-management)
7. [Drag and Drop](#7-drag-and-drop)

---

## 1. Sidebar Layout

The sidebar is toggled with `Cmd/Ctrl + \`. It docks to the left of the editor.

```
┌──────────────────────────────┐
│  My Vault           [v] [+]  │
│                              │
│  🔍 Search files...          │
│                              │
│  📁 docs/                    │
│  │  ├── report.md            │
│  │  └── notes/               │
│  │      ├── meeting.md       │
│  │      └── ideas.md         │
│  ├── inbox/                  │
│  │   └── imported.pdf.md     │
│  └── assets/                 │
│      └── diagram.png         │
│                              │
│  ── Tags ──────────────────  │
│  🏷️ health (12)             │
│  🏷️ research (8)            │
│  🏷️ meeting (5)             │
│                              │
│  ── Recent ────────────────  │
│  meeting.md                  │
│  ideas.md                    │
│  report.md                   │
└──────────────────────────────┘
```

### 1.1 Resizable

- Sidebar width adjustable via drag handle (min 180px, max 400px)
- Width persisted per vault in `.znrc`

---

## 2. File Tree

### 2.1 Tree Features

- Shows all files and folders in vault root (respects `.znrc` `ignore_patterns`)
- Files sorted: folders first (alphabetical), then files (alphabetical)
- Icons: folder icons, file icons by extension (`.md`, `.pdf`, `.png`, etc.)
- Right-click context menu on any node
- Multi-select with `Cmd/Ctrl+click` or `Shift+click`

### 2.2 Context Menu

| Action | Behavior |
|---|---|
| New File | Creates `.md` file in selected folder |
| New Folder | Creates subfolder in selected folder |
| Rename | Inline rename |
| Delete | Confirmation dialog → trash (configurable permanent delete) |
| Duplicate | Creates copy with `-copy` suffix |
| Move To | Opens folder picker |
| Copy Path | Copies full vault-relative path to clipboard |
| Reveal in File Manager | Opens system file manager at file location |
| Import... | Opens file import dialog |

### 2.3 File Indicators

| Indicator | Meaning |
|---|---|
| Dot badge | File has unsaved changes |
| Lock icon | File is binary (read-only preview) |
| Link icon | File is a symlink |
| Muted text | File matches `.gitignore` pattern |

### 2.4 Folder Operations

- Auto-collapse/expand on navigation
- "Expand all" / "Collapse all" from context menu
- Empty folders shown dimmed

---

## 3. Tags

### 3.1 Tag Sources

Tags come from two sources:
1. **Front matter `tags` field** in Markdown files: `tags: [health, research]`
2. **Manual assignment** via right-click → "Add Tag"

### 3.2 Tag Sidebar Section

- Tags listed alphabetically with file count
- Click tag → filters file tree to show only tagged files
- Multiple tags: `Ctrl+click` for AND filter
- Tag colors: defined in `.znrc` or auto-assigned

### 3.3 Tag Management

```yaml
# .znrc tag configuration
tags:
  colors:
    health: "#4CAF50"
    research: "#2196F3"
    meeting: "#FF9800"
    urgent: "#F44336"
  auto_tag_rules:
    - pattern: "docs/reports/**"
      tags: ["health", "report"]
```

- GUI: Settings → Tags → Manage tag colors and auto-tagging rules
- "Remove Tag" removes from all files or selected file

---

## 4. Full-Text Search

### 4.1 Search Scope

- **Current vault** (default): searches all files in vault
- **Current folder**: scope to folder and subfolders
- **Current file**: search within active document
- **Global**: across all vaults (Phase 2)

### 4.2 Search Features

| Feature | Detail |
|---|---|
| **Fuzzy matching** | Tolerates typos and partial matches |
| **Case sensitivity** | Toggle |
| **Regex** | Toggle |
| **File type filter** | `*.md`, `*.txt`, `*.pdf` (via ingestion engine) |
| **Path filter** | Limit search to specific folders |
| **Date filter** | Modified after/before date |
| **Results** | Show filename, path, line preview with highlights |
| **Replace in files** | Bulk find-and-replace across vault |

### 4.3 Search Index

- Maintained live: updates on file save
- Index stored in `.znrc-search/` (SQLite FTS5)
- Binary files skip indexing
- Vaults over 10,000 files show performance warning

---

## 5. File Operations

### 5.1 New File

- `Cmd/Ctrl + N` creates new `.md` file in active folder
- Name dialog appears: type name, press Enter
- Template picker: optionally create from template (see AI Templates)

### 5.2 File Import

| Import source | Behavior |
|---|---|
| Local file | Copies into vault (optionally converts via Ingestion Engine) |
| URL | Fetches content, optionally converts to Markdown |
| Clipboard | Paste image → saves to `assets/` |
| Bulk | Multi-file import dialog with progress bar |

### 5.3 File Export

| Format | Behavior |
|---|---|
| Markdown | Preserve original |
| HTML | Rendered document with CSS |
| PDF | Via browser print-to-PDF |
| Plain text | Strip Markdown formatting |

---

## 6. Vault Management

### 6.1 Vault Operations

| Operation | Behavior |
|---|---|
| **Open vault** | Select folder → ZarishNote creates `.znrc` if missing |
| **Create vault** | Select or create folder → ZarishNote initializes Git + `.znrc` |
| **Recent vaults** | List of last 10 vaults with quick-open |
| **Vault settings** | Opens `.znrc` in Settings GUI |
| **Close vault** | Returns to vault picker |

### 6.2 Multi-Vault (Phase 2)

- Tabbed vaults: switch between open vaults
- Cross-vault search
- Cross-vault drag-and-drop

---

## 7. Drag and Drop

### 7.1 Internal (within ZarishNote)

| Drag | Drop target | Result |
|---|---|---|
| File | Another folder | Move file |
| File (Ctrl+drag) | Another folder | Copy file |
| File | Editor | Insert file content (Markdown) |
| File | Tag in sidebar | Apply tag to file |
| File | Publish target | Queue file for publish |
| Selection of text | Sidebar | Creates new file with selection |

### 7.2 External (into ZarishNote)

| Drag source | Result |
|---|---|
| File from OS file manager | Copy/import file into vault |
| Image from browser | Download → save to `assets/` → insert Markdown |
| URL from browser | Fetch URL → insert as link or convert to Markdown |
| Text from browser | Insert as Markdown block |

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*
