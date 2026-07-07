# 003-ai-templates.md
## ZarishNote AI Templates Specification
### Reusable prompt templates — format, library, and invocation

**Document type:** Specification — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## Table of Contents

1. [Template Format](#1-template-format)
2. [Built-in Template Library](#2-built-in-template-library)
3. [Variable System](#3-variable-system)
4. [Template Invocation](#4-template-invocation)
5. [Template Management](#5-template-management)
6. [Template Execution Flow](#6-template-execution-flow)

---

## 1. Template Format

Templates are Markdown files stored in a `templates/` subfolder in the vault root. Each template file has YAML front matter and a Markdown body that serves as the AI prompt.

### 1.1 File Example

```markdown
---
template_name: "Clinical Case Summary"
description: "Generate a structured SOAP note from bullet points"
shortcut: "/case"
model: "claude"
system_prompt: "You are a clinical documentation assistant. Use SOAP format."
variables:
  - name: "patient_info"
    label: "Patient information"
    type: "textarea"
    required: true
  - name: "tone"
    label: "Tone"
    type: "select"
    options: ["formal", "standard"]
    default: "standard"
tags: ["clinical", "medical", "soap"]
---

Write a clinical case summary using the following patient information:

{{patient_info}}

Use a {{tone}} tone. Include:
- Chief complaint
- History of present illness
- Assessment
- Plan

Format as a SOAP note with clear section headings.
```

### 1.2 Front Matter Fields

| Field | Type | Required | Description |
|---|---|---|---|
| `template_name` | string | ✅ | Display name in UI |
| `description` | string | ❌ | Shown in template picker |
| `shortcut` | string | ❌ | `/shortcut` for quick invocation |
| `model` | string | ❌ | Preferred model hint (not enforced) |
| `system_prompt` | string | ❌ | Injected as system message |
| `variables` | array | ❌ | User-fillable variables |
| `tags` | array | ❌ | For categorization in picker |

### 1.3 Variable Definitions

Each variable object supports:

| Field | Type | Required | Description |
|---|---|---|---|
| `name` | string | ✅ | Variable key (used in template as `{{name}}`) |
| `label` | string | ✅ | UI label shown in form |
| `type` | enum | ✅ | `text`, `textarea`, `select`, `file`, `date` |
| `required` | bool | ❌ | Default: false |
| `default` | any | ❌ | Default value |
| `options` | array | For `select` type | List of options |
| `placeholder` | string | ❌ | Placeholder text |

---

## 2. Built-in Template Library

ZarishNote ships with these built-in templates:

| Template | Shortcut | Description |
|---|---|---|
| `summarize` | `/sum` | Summarize current document or selection |
| `outline` | `/outline` | Generate document outline |
| `meeting-notes` | `/mtg` | Structure rough notes into meeting summary |
| `blog-post` | `/blog` | Turn notes into blog post draft |
| `email` | `/email` | Convert bullet points into email |
| `code-review` | `/review` | Review selected code |
| `explain-code` | `/explain` | Explain code in plain language |
| `translate-en` | `/en` | Translate to English |
| `translate-bn` | `/bn` | Translate to Bangla |
| `translate-ar` | `/ar` | Translate to Arabic |
| `formal` | `/formal` | Rewrite in formal tone |
| `sop` | `/sop` | Format as standard operating procedure |
| `report` | `/report` | Structure as formal report |

Built-in templates are stored in the application data directory, not in the vault. They are read-only. Users can copy and customize them.

---

## 3. Variable System

### 3.1 Variable Substitution

Variables in the template body use `{{variable_name}}` syntax. Before the template is sent to the AI:

1. ZarishNote parses the front matter for variable declarations
2. If variables exist, a form is shown to the user
3. User fills in values → template body is rendered with substitutions
4. System prompt (if any) is prepended
5. Current document context is injected after template body

### 3.2 Context Variables

Built-in variables that require no user input:

| Variable | Value |
|---|---|
| `{{selection}}` | Currently selected text in editor |
| `{{document}}` | Full content of active document |
| `{{title}}` | Document title (first H1 or front matter title) |
| `{{date}}` | Current date |
| `{{vault_name}}` | Vault name from `.znrc` |

These are available in any template without declaration.

---

## 4. Template Invocation

### 4.1 From Editor (Slash Command)

Type `/` in the editor → template picker dropdown appears:
- Search by name, shortcut, or description
- Results filtered as user types
- Recently used templates shown first
- Select template → variable form (if needed) → AI response inserted at cursor

### 4.2 From AI Panel

- AI Panel → Templates dropdown → select template
- Same flow as slash command
- Response appears in AI panel (not auto-inserted)

### 4.3 From Right-Click

Select text → right-click → AI → "Template..." → pick template → response replaces selection.

---

## 5. Template Management

### 5.1 User-Created Templates

- Created via Templates sidebar → "+ New Template"
- GUI editor: name, description, shortcut, variables, prompt body
- Stored as `.md` files in `templates/` folder in vault
- Synced via Git like any vault file

### 5.2 Template Folder

```
vault/
├── templates/
│   ├── my-template.md
│   ├── weekly-report.md
│   └── clinical-case.md
└── (other vault files)
```

Templates are Git-tracked by default (part of the vault).

### 5.3 Sharing

Users can share templates by copying the `.md` file. There is no built-in template marketplace in V1.

---

## 6. Template Execution Flow

```
1. User invokes template (slash command, panel, right-click)
2. ZarishNote parses template file front matter + body
3. If variables exist → render variable form UI
4. User fills variables → submit
5. ZarishNote substitutes {{variables}} in body
6. System prompt (from front matter) set as system message
7. Current document context injected (truncated to fit token budget)
8. Request sent to configured AI provider
9. Response streamed back to user
10. "Insert" / "Replace" / "Copy" / "Retry" actions shown
```

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*
