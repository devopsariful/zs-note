# 002-ai-providers.md
## ZarishNote AI Providers Specification
### All supported AI providers, configuration, and routing

**Document type:** Specification — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## Table of Contents

1. [Provider Model](#1-provider-model)
2. [OpenAI](#2-openai)
3. [Anthropic Claude](#3-anthropic-claude)
4. [Google Gemini](#4-google-gemini)
5. [DeepSeek](#5-deepseek)
6. [Ollama (Local)](#6-ollama-local)
7. [LM Studio (Local)](#7-lm-studio-local)
8. [Custom OpenAI-Compatible Endpoint](#8-custom-openai-compatible-endpoint)
9. [Provider Configuration in .znrc](#9-provider-configuration-in-znrc)
10. [Multi-Provider Routing](#10-multi-provider-routing)
11. [API Key Management](#11-api-key-management)

---

## 1. Provider Model

All providers expose the same capabilities through a unified interface:
- **Chat completion** (streaming + non-streaming)
- **Tool calling** (function calling)
- **Vision** (image inputs, where supported)
- **Token counting** (estimated before send)

Each provider is configured with: endpoint, model name, key reference, and optional parameters (temperature, max_tokens, etc.).

---

## 2. OpenAI

| Field | Value |
|---|---|
| Base endpoint | `https://api.openai.com/v1` |
| Auth | Bearer token via API key |
| Streaming | SSE (server-sent events) |
| Tool calling | Function calling API |
| Vision | Supported (GPT-4o, GPT-4o-mini) |

### Supported Models

| Model | Purpose | Max tokens | Vision |
|---|---|---|---|
| `gpt-4o` | General intelligence | 16K | ✅ |
| `gpt-4o-mini` | Lightweight, cost-effective | 16K | ✅ |
| `o3` | Reasoning, complex tasks | 100K | ❌ |
| `gpt-4.1` | Latest flagship | 32K | ✅ |

### Configuration

```yaml
ai:
  providers:
    - name: "openai"
      model: "gpt-4o"
      key_id: "openai-api-key"
      max_tokens: 4096
      temperature: 0.7
```

---

## 3. Anthropic Claude

| Field | Value |
|---|---|
| Base endpoint | `https://api.anthropic.com/v1` |
| Auth | `x-api-key` header |
| Streaming | SSE |
| Tool calling | Tool use API |
| Vision | Supported |

### Supported Models

| Model | Purpose | Max tokens | Vision |
|---|---|---|---|
| `claude-sonnet-4-6` | Balanced intelligence | 8K | ✅ |
| `claude-opus-4` | Maximum capability | 16K | ✅ |
| `claude-haiku-3-5` | Fast, lightweight | 8K | ✅ |

### Configuration

```yaml
ai:
  providers:
    - name: "claude"
      model: "claude-sonnet-4-6"
      key_id: "anthropic-api-key"
      max_tokens: 8192
      temperature: 0.7
```

---

## 4. Google Gemini

| Field | Value |
|---|---|
| Base endpoint | `https://generativelanguage.googleapis.com/v1beta` |
| Auth | API key as query param |
| Streaming | SSE |
| Tool calling | Function calling API |
| Vision | Supported |

### Supported Models

| Model | Purpose | Max tokens | Vision |
|---|---|---|---|
| `gemini-2.5-pro` | General intelligence | 8K | ✅ |
| `gemini-2.5-flash` | Fast, cost-effective | 8K | ✅ |

### Configuration

```yaml
ai:
  providers:
    - name: "gemini"
      model: "gemini-2.5-pro"
      key_id: "gemini-api-key"
      max_tokens: 8192
```

---

## 5. DeepSeek

| Field | Value |
|---|---|
| Base endpoint | `https://api.deepseek.com/v1` |
| Auth | Bearer token via API key |
| Streaming | SSE |
| Tool calling | OpenAI-compatible function calling |
| Vision | Not supported |

### Supported Models

| Model | Purpose | Max tokens |
|---|---|---|
| `deepseek-chat` | General intelligence | 8K |
| `deepseek-reasoner` | Reasoning tasks | 8K |

### Configuration

```yaml
ai:
  providers:
    - name: "deepseek"
      model: "deepseek-chat"
      key_id: "deepseek-api-key"
      max_tokens: 4096
```

---

## 6. Ollama (Local)

| Field | Value |
|---|---|
| Base endpoint | `http://localhost:11434/v1` |
| Auth | None |
| Streaming | SSE |
| Tool calling | OpenAI-compatible |
| Vision | Depends on model |

### Configuration

```yaml
ai:
  providers:
    - name: "ollama"
      model: "llama3.2:3b"
      endpoint: "http://localhost:11434/v1"
      key_id: null
      max_tokens: 4096
```

Model list is fetched dynamically via `ollama list` on connection test. Users pick from installed models.

---

## 7. LM Studio (Local)

| Field | Value |
|---|---|
| Base endpoint | `http://localhost:1234/v1` |
| Auth | None |
| Streaming | SSE |
| Tool calling | OpenAI-compatible (model dependent) |

### Configuration

```yaml
ai:
  providers:
    - name: "lm-studio"
      model: "local-model"
      endpoint: "http://localhost:1234/v1"
      key_id: null
      max_tokens: 4096
```

---

## 8. Custom OpenAI-Compatible Endpoint

ZarishNote supports any server that implements the OpenAI chat completions API format.

| Field | Required |
|---|---|
| Endpoint URL | ✅ |
| Model name | ✅ |
| API key | Optional |
| Extra headers | Optional |

### Configuration

```yaml
ai:
  providers:
    - name: "custom"
      model: "my-model"
      endpoint: "http://localhost:8080/v1"
      key_id: null
      max_tokens: 2048
      extra_headers:
        X-Organization: "my-org"
```

---

## 9. Provider Configuration in .znrc

```yaml
ai:
  default_provider: "ollama"

  providers:
    - name: "openai"
      model: "gpt-4o"
      endpoint: "https://api.openai.com/v1"     # optional, has default
      key_id: "openai-api-key"
      max_tokens: 4096
      temperature: 0.7

    - name: "claude"
      model: "claude-sonnet-4-6"
      key_id: "anthropic-api-key"                # uses default endpoint
      max_tokens: 8192
      temperature: 0.7

    - name: "ollama"
      model: "llama3.2:3b"
      key_id: null                               # no key needed
      max_tokens: 4096
```

Fields are nullable where sensible: `endpoint` defaults to provider standard, `key_id` null means no auth, `temperature` defaults to 0.7.

---

## 10. Multi-Provider Routing

Routing rules in `.znrc` automatically select a provider based on file path or query pattern:

```yaml
ai:
  routing_rules:
    - pattern: "*.py"
      provider: "openai"
      model: "gpt-4o"
    - pattern: "docs/clinical/**"
      provider: "claude"
      model: "claude-sonnet-4-6"
    - pattern: "journals/**"
      provider: "ollama"
      model: "llama3.2:3b"
```

First matching rule wins. Rules evaluated in order. Default: `ai.default_provider`.

---

## 11. API Key Management

All API keys stored in OS Keychain. Access pattern:

```rust
// Rust keychain access via Tauri plugin
#[tauri::command]
async fn get_api_key(key_id: String) -> Result<String, String> {
    let key = keyring::Entry::new("zarishnote", &key_id)
        .map_err(|e| format!("Keychain error: {}", e))?;

    key.get_password()
        .map_err(|_| format!("Key '{}' not found in keychain", key_id))
}
```

Keys are:
- Stored under namespaced identifier: `zarishnote.{key_id}`
- Never written to disk in config files
- Never displayed in UI after entry
- Deleted via "Remove Key" button in Settings

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*
