# 001-ai-chat-spec.md
## ZarishNote AI Engine Specification
### Multi-provider AI chat, tool calling, and writing actions

**Document type:** Specification — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## Table of Contents

1. [AI Panel Layout](#1-ai-panel-layout)
2. [Supported Providers](#2-supported-providers)
3. [Streaming Chat](#3-streaming-chat)
4. [Writing Actions](#4-writing-actions)
5. [AI Templates](#5-ai-templates)
6. [Context Injection](#6-context-injection)
7. [API Key Security](#7-api-key-security)
8. [Image Generation](#8-image-generation)

---

## 1. AI Panel Layout

The AI panel is toggled with `Cmd/Ctrl + Shift + A`. It docks to the right of the editor and does not push the editor — it overlays with adjustable width.

```
┌─────────────────────────────────────┐
│  AI Panel                    [×] [↔]│
│  ┌─────────────────────────────────┐│
│  │  Provider: [Claude ▾] [Model ▾] ││
│  └─────────────────────────────────┘│
│                                     │
│  ┌─────────────────────────────────┐│
│  │  Chat history                   ││
│  │  (scrollable)                   ││
│  │                                 ││
│  │  [You] Summarize this section   ││
│  │  [AI]  Here is a summary...     ││
│  └─────────────────────────────────┘│
│                                     │
│  [Insert] [Replace] [Copy] [Retry]  │
│                                     │
│  ┌─────────────────────────────────┐│
│  │  Ask anything... (Shift+Enter   ││
│  │  = newline, Enter = send)       ││
│  └─────────────────────────────────┘│
│  [Templates ▾] [Tools ▾] [Clear]    │
└─────────────────────────────────────┘
```

---

## 2. Supported Providers

| Provider | Auth method | Local/Cloud | Streaming |
|---|---|---|---|
| OpenAI | API key (Keychain) | Cloud | ✅ |
| Anthropic Claude | API key (Keychain) | Cloud | ✅ |
| Google Gemini | API key (Keychain) | Cloud | ✅ |
| DeepSeek | API key (Keychain) | Cloud | ✅ |
| Ollama | No key (localhost) | Local | ✅ |
| LM Studio | No key (localhost) | Local | ✅ |
| Any OpenAI-compatible | API key optional | Local or Cloud | ✅ |

### 2.1 Provider Switching

- Switch provider mid-conversation from the panel dropdown
- Conversation history carries over (reformatted if needed for different API)
- Model list auto-populated from API (for Ollama: `ollama list`)
- "Test connection" button verifies provider before use

### 2.2 Multi-Provider Routing (Advanced)

Defined in `.znrc`, rules can auto-route queries:
```yaml
ai:
  routing_rules:
    - pattern: "*.py"
      provider: "openai"
      model: "gpt-4o"
    - pattern: "docs/**"
      provider: "ollama"
      model: "llama3.2"
```

---

## 3. Streaming Chat

All providers use SSE (Server-Sent Events) or equivalent streaming:
- Response tokens appear in the panel as they arrive
- "Stop" button cancels in-flight request
- Cursor jumps to bottom of response as it streams
- Code blocks are syntax-highlighted as they render

### 3.1 Input Behavior

- `Enter` sends message
- `Shift+Enter` inserts newline
- Prevents CJK IME composition conflicts (IME composition active = enter inserts newline)
- Paste image: auto-attaches image as vision input (if provider supports it)

### 3.2 Conversation History

- Stored in vault as `.znrc-history/{date}-{model}.jsonl`
- Never sent to any server other than the configured provider
- Conversation visible in history sidebar (per-file or global)
- Export conversation as Markdown

---

## 4. Writing Actions

When AI returns a response, action buttons appear:

| Button | Action |
|---|---|
| **Insert** | Insert AI response at current cursor position in editor |
| **Replace** | Replace selected text in editor with AI response |
| **Copy** | Copy AI response to clipboard |
| **Retry** | Regenerate response with same prompt |
| **Edit & Retry** | Edit the last user message and regenerate |

### 4.1 Inline Actions (on selected text in editor)

Right-click on selected text → AI submenu:

| Action | Prompt sent to AI |
|---|---|
| **Summarize** | "Summarize this: {selection}" |
| **Rewrite** | "Rewrite this more clearly: {selection}" |
| **Translate → [lang]** | "Translate to [lang]: {selection}" |
| **Fix grammar** | "Fix grammar and spelling: {selection}" |
| **Make formal** | "Rewrite this in formal language: {selection}" |
| **Make concise** | "Rewrite this more concisely: {selection}" |
| **Expand** | "Expand this with more detail: {selection}" |
| **Explain** | "Explain this: {selection}" |
| **Generate follow-up** | "Write 3 follow-up questions about: {selection}" |
| **Custom prompt...** | Opens AI panel with selection as context |

---

## 5. AI Templates

Templates are reusable prompts stored in vault as Markdown files in `templates/` folder.

### 5.1 Template File Format

```markdown
---
template_name: "Clinical Case Summary"
shortcut: "/case"
model: "claude"
system_prompt: "You are a clinical documentation assistant."
variables:
  - name: "patient_id"
    label: "Patient ID"
    type: "text"
  - name: "condition"
    label: "Condition"
    type: "text"
---

Write a clinical case summary for patient {{patient_id}} with condition {{condition}}.
Include: chief complaint, history, assessment, and plan.
Use SOAP note format.
```

### 5.2 Template Invocation

- Type `/` in editor → shows template picker with search
- Click template → variable form appears (if any variables defined)
- Fill variables → click "Generate" → AI response inserted at cursor
- Templates also available from AI Panel → Templates dropdown

### 5.3 Built-in Templates

ZarishNote ships with a default template library:

| Template | Purpose |
|---|---|
| `summarize` | Summarize current document |
| `outline` | Generate outline from document |
| `meeting-notes` | Format rough notes into structured meeting summary |
| `blog-post` | Turn notes into a blog post draft |
| `email` | Turn bullet points into email |
| `code-review` | Review selected code |
| `explain-code` | Explain selected code in plain language |
| `translate-en` | Translate to English |
| `translate-bn` | Translate to Bangla |
| `formal` | Rewrite in formal tone |
| `sop` | Turn notes into SOP format |
| `report` | Turn notes into report format |

---

## 6. Context Injection

Before every AI request, ZarishNote automatically injects context:

1. **Current file content** — active document (truncated to model token limit)
2. **Selection context** — if text is selected, included as `[Selected text]:`
3. **Core files** — from `.znrc` `ai.context.core_files`
4. **RAG retrieval** — top-5 most relevant chunks from local vector store
5. **Rules** — matching rules from `.znrc` `ai.context.rules`
6. **Tool results** — if tools are active in session

Token budget is managed automatically — higher-priority context wins when truncation needed.

### 6.1 Context Panel

Users can see exactly what context is being sent:
- `Cmd/Ctrl + Shift + C` opens "Context Inspector" panel
- Shows all injected context with token count
- Individual items can be pinned or excluded

---

## 7. API Key Security

All API keys are stored in the **OS Keychain**, not in `.znrc`:

| Platform | Storage |
|---|---|
| macOS | macOS Keychain Services |
| Windows | Windows Credential Manager |
| Linux | GNOME Keyring / KWallet |
| Android | Android Keystore |
| iOS | iOS Keychain |

Key management in GUI:
- Settings → AI → Providers → [provider] → "Set API Key"
- Key stored under name like `zarishnote.openai-api-key`
- "Test Connection" sends one token to verify key
- Key is never shown again after initial entry (masked)
- "Delete Key" removes from OS Keychain

---

## 8. Image Generation

When a provider supports image generation (OpenAI DALL-E, Stability AI):

- AI panel toolbar shows image icon
- Click → opens image generation dialog
- Type prompt → generates image
- Image saved to `assets/ai-images/` in vault
- Markdown reference inserted at cursor: `![generated image](assets/ai-images/img-001.png)`

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*