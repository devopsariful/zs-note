# 003-znrc-schema.md
## ZarishNote Runtime Config (`.znrc`) Schema
### Complete workspace configuration specification

**Document type:** Specification — V1
**Date:** June 08, 2026
**Author:** Mohammad Ariful Islam / ZarishSphere Foundation
**License:** CC BY 4.0
**Status:** V1 — Authoritative

---

## 1. What is `.znrc`

The `.znrc` file is the "brain" of a ZarishNote vault. It is a YAML file placed at the vault root.

It defines:
- AI provider routing and behavior rules
- Sandbox permissions and tool capabilities
- Context injection rules for RAG
- MCP server connections
- Publishing targets
- Sync strategy
- Plugin list

Every setting in `.znrc` has an equivalent GUI panel in Settings. The file and the GUI are always in sync. You never need to edit `.znrc` manually — but you can if you prefer.

---

## 2. Full Schema

```yaml
# .znrc — ZarishNote Runtime Config
# Version: v1
# Location: vault root (.znrc)
# All settings available via Settings GUI

znrc_version: "v1"

# ==============================================================================
# 1. VAULT METADATA
# ==============================================================================
vault:
  name: "My Vault"
  description: "Personal knowledge base"
  language: "en"               # ISO 639-1: en, bn, ar, fr, etc.
  timezone: "Asia/Dhaka"       # IANA timezone
  default_view: "wysiwyg"      # wysiwyg | source | split

# ==============================================================================
# 2. EDITOR
# ==============================================================================
editor:
  font_family: "JetBrains Mono, monospace"
  font_size: 16
  line_height: 1.7
  prose_width: "720px"
  theme: "system"              # light | dark | system
  auto_save: true
  vim_mode: false
  spell_check: true
  line_numbers: false
  word_count: true
  focus_mode: false

# ==============================================================================
# 3. AI ENGINE
# ==============================================================================
ai:
  # Default provider used for new conversations
  default_provider: "ollama"

  # Per-provider configuration
  # API keys are NEVER stored here — they are in OS Keychain
  # Reference them by key_id (the name you gave when setting up in GUI)
  providers:
    - name: "openai"
      model: "gpt-4o"
      endpoint: "https://api.openai.com/v1"
      key_id: "openai-api-key"     # OS Keychain reference
      max_tokens: 4096
      temperature: 0.7

    - name: "claude"
      model: "claude-sonnet-4-6"
      endpoint: "https://api.anthropic.com/v1"
      key_id: "anthropic-api-key"
      max_tokens: 8192
      temperature: 0.7

    - name: "gemini"
      model: "gemini-2.5-pro"
      endpoint: "https://generativelanguage.googleapis.com/v1beta"
      key_id: "gemini-api-key"
      max_tokens: 8192

    - name: "deepseek"
      model: "deepseek-chat"
      endpoint: "https://api.deepseek.com/v1"
      key_id: "deepseek-api-key"
      max_tokens: 4096

    - name: "ollama"
      model: "llama3.2:3b"
      endpoint: "http://localhost:11434/v1"
      key_id: null               # Ollama needs no key
      max_tokens: 4096

    - name: "custom"
      model: "my-model"
      endpoint: "http://localhost:8080/v1"
      key_id: null
      max_tokens: 2048

  # Context injection rules — what automatically goes into every AI prompt
  context:
    # Files always included (up to token budget)
    core_files:
      - "README.md"
      - "context/project-overview.md"

    # RAG configuration — local semantic search over vault
    rag:
      enabled: true
      chunking:
        size: 512                # tokens per chunk
        overlap: 50
        strategy: "semantic"     # semantic | fixed | hierarchical
      store:
        type: "lancedb"          # only option in V1
        path: ".znrc-vectors"    # relative to vault root
        embedding_model: "all-MiniLM-L6-v2"  # local ONNX
      index_rules:
        include: ["*.md", "*.txt"]
        exclude: [".znrc-vectors/**", ".git/**", "node_modules/**"]

    # Per-pattern rules injected into system prompt
    rules:
      - name: "formal-writing"
        pattern: "docs/**"
        instruction: "Use formal, professional language. Avoid contractions."
        priority: 100

      - name: "code-security"
        pattern: "src/**"
        instruction: "Prioritize security, input validation, and error handling."
        priority: 90

  # Image generation (optional)
  image_generation:
    provider: "openai"           # openai | stability | disabled
    model: "dall-e-3"
    key_id: "openai-api-key"
    output_folder: "assets/ai-images"

# ==============================================================================
# 4. SANDBOX ENGINE
# ==============================================================================
sandbox:
  # Global sandbox policy
  engine: "wasmtime"
  default_memory_limit: "256MB"
  default_timeout: "30s"
  default_network: false           # No network unless explicitly granted

  # Per-tool overrides (see tools section below)
  # Network allow-list (used when a tool requests network: true)
  network:
    default_policy: "deny"         # deny | allow
    allowed_outbound:
      - "api.openai.com"
      - "api.anthropic.com"
      - "*.github.com"
      - "localhost:*"              # local services always allowed

# ==============================================================================
# 5. TOOLS (sandboxed)
# ==============================================================================
tools:
  - name: "run-code"
    description: "Execute a code block in the current document (sandboxed)"
    type: "wasm"
    wasm_path: "tools/run-code.wasm"
    sandbox:
      memory_limit: "128MB"
      timeout: "15s"
      network: false
      permissions:
        - "read:workspace"
        - "write:stdout"

  - name: "web-fetch"
    description: "Fetch a URL and return the content as Markdown"
    type: "wasm"
    wasm_path: "tools/web-fetch.wasm"
    sandbox:
      memory_limit: "64MB"
      timeout: "10s"
      network: true
      network_domains:
        - "*.wikipedia.org"
        - "*.github.com"
      permissions:
        - "read:public-url"

  - name: "ingest-file"
    description: "Convert a local file to Markdown using the ingestion engine"
    type: "command"
    command: "zarishnote-ingest"
    sandbox:
      memory_limit: "512MB"
      timeout: "60s"
      network: false
      permissions:
        - "read:workspace"
        - "write:workspace"

# ==============================================================================
# 6. MCP SERVERS
# ==============================================================================
mcp:
  servers:
    - name: "github"
      transport: "stdio"
      command: "npx @modelcontextprotocol/server-github"
      env:
        GITHUB_TOKEN: "${keychain:github-token}"   # key from OS Keychain
      sandbox: true              # always sandboxed
      enabled: true

    - name: "filesystem"
      transport: "stdio"
      command: "npx @modelcontextprotocol/server-filesystem"
      args: ["--root", "${vault_path}"]
      sandbox: true
      enabled: true

    - name: "custom-api"
      transport: "http"
      url: "http://localhost:3000/mcp"
      sandbox: true
      enabled: false

  # Knowledge bases (local RAG sources visible to AI via MCP)
  knowledge_bases:
    - name: "clinical-guidelines"
      path: "knowledge/clinical"
      index_on_start: true
      formats: ["*.md", "*.pdf"]

    - name: "project-context"
      path: "knowledge/project"
      index_on_start: true

# ==============================================================================
# 7. VOICE ENGINE
# ==============================================================================
voice:
  enabled: true
  model: "whisper-base"          # whisper-tiny | whisper-base | whisper-small
  language: "en"                 # ISO 639-1
  speaker_labels: false          # Phase 2 feature
  output_format: "markdown"      # markdown | plain | srt
  timestamp_granularity: "sentence"  # word | sentence | paragraph
  auto_punctuate: true
  output_folder: "transcripts"

# ==============================================================================
# 8. INGESTION ENGINE
# ==============================================================================
ingestion:
  enabled: true
  default_output_folder: "inbox"
  # Vision description for images (requires AI provider configured)
  vision:
    enabled: false
    provider: "openai"
    model: "gpt-4o"
    prompt: "Describe this image in detail, including any text, charts, or diagrams visible."
  # Audio transcription
  audio:
    enabled: false
    use_voice_engine: true       # reuse voice engine config

# ==============================================================================
# 9. PUBLISHING
# ==============================================================================
publish:
  targets:
    - name: "github-pages"
      type: "github"
      repo: "username/my-blog"
      branch: "gh-pages"
      path: "/"
      key_id: "github-token"
      build_command: null        # optional: run a build before pushing
      rss:
        enabled: true
        title: "My Blog"
        description: "Notes and writings"
        author: "Mohammad Ariful Islam"
        base_url: "https://username.github.io/my-blog"

    - name: "custom-api"
      type: "api"
      endpoint: "https://my-cms.example.com/api/publish"
      method: "POST"
      key_id: "cms-api-key"
      format: "json"             # json | multipart | markdown

  # Image hosting
  image_hosting:
    provider: "github"           # github | cloudflare | local | none
    repo: "username/assets"
    branch: "main"
    path: "images/"
    key_id: "github-token"
    cdn_base_url: "https://raw.githubusercontent.com/username/assets/main/images"

# ==============================================================================
# 10. SYNC / GIT
# ==============================================================================
sync:
  auto_commit: true
  commit_message_style: "conventional"  # conventional | timestamp | diff-summary
  commit_author: "ZarishNote <zarishnote@local>"
  ignore_patterns:
    - ".znrc-vectors/**"
    - ".git/**"
    - "*.lock"
  remote:
    enabled: false               # Set true to push to remote
    url: ""                      # e.g., git@github.com:user/vault.git
    branch: "main"
    key_id: null                 # SSH key reference from keychain

# ==============================================================================
# 11. PLUGINS
# ==============================================================================
plugins:
  enabled: true
  install_dir: ".znrc-plugins"
  installed:
    - name: "mermaid-extra"
      version: "1.2.0"
      source: "https://registry.zarishsphere.com/plugins/mermaid-extra"  # hosted via Cloudflare Pages on zarishsphere.com
      checksum_sha256: "abc123..."
      sandbox: true
      enabled: true

# ==============================================================================
# 12. SECURITY
# ==============================================================================
security:
  # Prevent plugins from accessing files outside vault root
  vault_isolation: true
  # Block any tool from accessing system environment variables
  block_env_access: true
  # Log all tool executions to audit.log
  audit_log: true
  audit_log_path: ".znrc-audit.log"
```

---

## 3. Validation Rules

The Rust backend validates `.znrc` on load. Any violation causes a clear error with fix instructions:

| Rule | Violation Behavior |
|---|---|
| `znrc_version` must be `"v1"` | Error: "Unknown version. Expected `v1`." |
| `ai.providers[].key_id` must not contain actual key strings | Error: "API key stored in config. Use OS Keychain." |
| `sandbox.engine` must be `"wasmtime"` | Error: "Unsupported engine." |
| `sandbox.default_network` must be boolean | Error: "Invalid type." |
| `mcp.servers[].transport` must be `"stdio"` or `"http"` | Error: "Unsupported transport." |
| File paths in `core_files` must exist in vault | Warning: "File not found. Skipped." |
| `publish.targets[].type` must be `"github"` or `"api"` | Error: "Unknown publish type." |

---

## 4. GUI Mapping

Every field in `.znrc` has a corresponding UI control:

| `.znrc` path | Settings GUI location |
|---|---|
| `editor.*` | Settings → Editor |
| `ai.providers[]` | Settings → AI → Providers |
| `ai.context.rag` | Settings → AI → Context |
| `ai.context.rules[]` | Settings → AI → Rules |
| `sandbox.*` | Settings → Sandbox |
| `tools[]` | Settings → Tools |
| `mcp.servers[]` | Settings → MCP → Servers |
| `mcp.knowledge_bases[]` | Settings → MCP → Knowledge Bases |
| `voice.*` | Settings → Voice |
| `publish.targets[]` | Settings → Publish → Targets |
| `sync.*` | Settings → Sync |
| `plugins.installed[]` | Settings → Plugins |
| `security.*` | Settings → Security |

---

*ZarishSphere Foundation · V1 · June 2026*
*License: CC BY 4.0*