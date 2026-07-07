# 001-editor-spec.md
## ZarishNote Core Editor Specification
### WYSIWYG Markdown editor — full feature requirements

**Document type:** Specification — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## Table of Contents

1. [Editor Modes](#1-editor-modes)
2. [Markdown Feature Support](#2-markdown-feature-support)
3. [Formatting Toolbar](#3-formatting-toolbar)
4. [Search and Replace](#4-search-and-replace)
5. [YAML Front Matter](#5-yaml-front-matter)
6. [Keyboard Shortcuts](#6-keyboard-shortcuts)
7. [Editor Settings](#7-editor-settings)
8. [Accessibility](#8-accessibility)

---

## 1. Editor Modes

ZarishNote supports three view modes, switchable at any time without losing content or cursor position:

| Mode | Description | Trigger |
|---|---|---|
| **WYSIWYG (default)** | Type `#` → heading renders instantly. Type `**` → bold. Live Markdown rendering via Milkdown/ProseMirror | Default on open |
| **Source** | Raw Markdown text. Syntax highlighting via Shiki. Full monospace. | `Cmd/Ctrl + E` or View menu |
| **Split** | Side-by-side: source (left) + rendered preview (right). Live sync. | `Cmd/Ctrl + Shift + E` or View menu |
| **Preview only** | Read-only rendered view. Useful for reviewing before publishing. | `Cmd/Ctrl + Shift + P` |

### 1.1 WYSIWYG Behavior Rules

- Heading rendering: type `# ` → H1 renders inline. No separate preview pane needed.
- Bold/italic: type `**text**` → renders as bold, stored as Markdown.
- Links: type `[text](url)` → renders as clickable link, shows URL on hover.
- Tables: rendered as HTML table. Editable in place.
- Code blocks: fenced with language-specific syntax highlighting.
- Math: `$inline$` and `$$block$$` render via KaTeX.
- Mermaid diagrams: fenced code blocks with `mermaid` tag render as live diagrams.
- Escape key in a heading/block → reverts to raw Markdown for that block.

---

## 2. Markdown Feature Support

### 2.1 Standard CommonMark

All CommonMark 0.31+ elements are supported: headings, paragraphs, blockquotes, lists (ordered/unordered/task), code spans, code fences, images, links, horizontal rules, hard line breaks.

### 2.2 Extended Syntax (GFM + extras)

| Feature | Syntax | Notes |
|---|---|---|
| **Tables** | `\| col \| col \|` | Editable in WYSIWYG mode. Sortable header. |
| **Task lists** | `- [ ] text` | Checkboxes clickable in WYSIWYG mode |
| **Strikethrough** | `~~text~~` | GFM |
| **Footnotes** | `[^1]` / `[^1]:` | |
| **Definition lists** | `term\n: definition` | |
| **Inline math** | `$expr$` | KaTeX renderer |
| **Block math** | `$$\nexpr\n$$` | KaTeX renderer |
| **Mermaid diagrams** | ` ```mermaid ` | Live render in all modes |
| **PlantUML** | ` ```plantuml ` | Requires local PlantUML server or pre-compiled |
| **D2 diagrams** | ` ```d2 ` | Via D2 WASM binary |
| **Frontmatter** | `---\nkey: val\n---` | YAML, parsed and displayed as structured metadata panel |
| **Highlighted text** | `==text==` | |
| **Subscript/Superscript** | `~sub~` / `^sup^` | |
| **Emoji** | `:emoji-name:` | Resolved via local emoji map |
| **Wikilinks** | `[[note-title]]` | Resolved to vault files |

### 2.3 Code Blocks

All fenced code blocks support:
- Language identifier for syntax highlighting (Shiki: 200+ languages)
- Copy button in WYSIWYG mode
- Line numbers (optional, toggle in settings)
- Diff highlighting (`diff` language identifier)
- `run` button for executable code blocks (sandboxed via Wasmtime — see sandbox spec)

### 2.4 Image Handling

- Paste from clipboard → auto-saved to `assets/` folder in vault
- Drag-and-drop from file system → same
- Remote URLs → optional: download + embed locally, or keep URL reference
- Resize handle in WYSIWYG mode
- Alt text editable inline

---

## 3. Formatting Toolbar

Shown below the document title. Collapsible. Keyboard-first design.

```
[H1] [H2] [H3] | [B] [I] [S] | [Link] [Image] | [Table] [Code] [Math] | [List] [OList] [Task] | [Diagram] | [Quote] | [HR]
```

Each button shows its keyboard shortcut on hover.

### 3.1 Table Editor

When cursor is inside a table:
- Context toolbar appears: Add row, Delete row, Add column, Delete column, Align (left/center/right)
- Tab key moves between cells
- Enter key adds a new row at end

### 3.2 Diagram Selector

Clicking `[Diagram]` opens a panel:
- Select type: Mermaid, PlantUML, D2
- Choose subtype: flowchart, sequence, class, gantt, entity-relation, pie, state, C4
- Insert template for selected type
- Preview updates as you type

---

## 4. Search and Replace

Triggered with `Cmd/Ctrl + F` (search) or `Cmd/Ctrl + H` (replace).

| Feature | Detail |
|---|---|
| **Search scope** | Current file, or all files in vault |
| **Case sensitivity** | Toggle |
| **Regex support** | Toggle (standard JS/PCRE regex) |
| **Replace** | Single occurrence or All |
| **Highlight** | All matches highlighted in editor |
| **Navigate** | Next / Previous match (`F3` / `Shift+F3`) |

---

## 5. YAML Front Matter

Front matter is automatically detected and:
- Displayed as a structured metadata panel at top of document (not raw YAML in WYSIWYG mode)
- Fields editable as form inputs (string, date, boolean, list)
- Toggle to show raw YAML
- Predefined fields: `title`, `date`, `tags`, `author`, `status`, `published`, `cover`
- Custom fields: any key/value pair

---

## 6. Keyboard Shortcuts

### 6.1 Global

| Action | macOS | Windows/Linux |
|---|---|---|
| New file | `Cmd+N` | `Ctrl+N` |
| Open vault | `Cmd+O` | `Ctrl+O` |
| Save | Auto (every keystroke) | Auto |
| Find | `Cmd+F` | `Ctrl+F` |
| Replace | `Cmd+H` | `Ctrl+H` |
| Toggle sidebar | `Cmd+\` | `Ctrl+\` |
| Toggle AI panel | `Cmd+Shift+A` | `Ctrl+Shift+A` |
| Command palette | `Cmd+P` | `Ctrl+P` |

### 6.2 Editor

| Action | macOS | Windows/Linux |
|---|---|---|
| Bold | `Cmd+B` | `Ctrl+B` |
| Italic | `Cmd+I` | `Ctrl+I` |
| Inline code | `Cmd+\`` | `Ctrl+\`` |
| Link | `Cmd+K` | `Ctrl+K` |
| Toggle WYSIWYG/Source | `Cmd+E` | `Ctrl+E` |
| Toggle split view | `Cmd+Shift+E` | `Ctrl+Shift+E` |
| Insert table | `Cmd+Shift+T` | `Ctrl+Shift+T` |
| Insert code block | `Cmd+Shift+C` | `Ctrl+Shift+C` |
| Indent list | `Tab` | `Tab` |
| Outdent list | `Shift+Tab` | `Shift+Tab` |

---

## 7. Editor Settings

All settings available in GUI (Settings → Editor tab):

| Setting | Type | Default | Description |
|---|---|---|---|
| `font-family` | string | `"JetBrains Mono, monospace"` | Source/code font |
| `font-size` | number | 16 | Base font size in px |
| `line-height` | number | 1.7 | Line height multiplier |
| `prose-width` | string | `"720px"` | Max content width |
| `theme` | enum | `"system"` | `"light"`, `"dark"`, `"system"` |
| `auto-save` | boolean | true | Save on every keystroke |
| `vim-mode` | boolean | false | Vim keybindings |
| `spell-check` | boolean | true | System spell check |
| `line-numbers` | boolean | false | Show line numbers in source mode |
| `word-count` | boolean | true | Show word/char count in status bar |
| `focus-mode` | boolean | false | Hide sidebar + toolbar for distraction-free writing |

---

## 8. Accessibility

- ARIA labels on all controls
- Full keyboard navigation without mouse
- High-contrast theme option
- Respects system font scaling
- Screen reader compatibility (semantic HTML output)

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*