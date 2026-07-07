export interface Document {
  path: string;
  name: string;
  extension: string;
  content: string;
  frontmatter: Record<string, unknown>;
  tags: string[];
  created: string;
  modified: string;
  size: number;
}

export interface FileEntry {
  name: string;
  path: string;
  is_dir: boolean;
  children?: FileEntry[];
  extension?: string;
  size?: number;
  modified?: string;
}

export interface RecentFile {
  name: string;
  path: string;
  modified: string;
}

export interface ChatMessage {
  role: 'user' | 'assistant' | 'system';
  content: string;
  id: string;
  timestamp: number;
  provider?: string;
  model?: string;
  tokens?: number;
  done: boolean;
}

export interface Provider {
  id: string;
  name: string;
  type: ProviderType;
  apiKey?: string;
  baseUrl?: string;
  models: string[];
  defaultModel?: string;
  enabled: boolean;
}

export type ProviderType = 'openai' | 'anthropic' | 'google' | 'deepseek' | 'ollama' | 'custom';

export interface ProviderConfig {
  id: string;
  name: string;
  provider_type: ProviderType;
  api_key?: string | null;
  base_url?: string | null;
  models: string[];
  default_model: string;
  enabled: boolean;
  temperature?: number | null;
  max_tokens?: number | null;
}

export interface ToolConfig {
  id: string;
  name: string;
  description: string;
  enabled: boolean;
  command?: string;
  args?: string[];
}

export interface SandboxConfig {
  enabled: boolean;
  timeout_ms: number;
  memory_mb: number;
  allowed_fs_paths: string[];
  allowed_network: boolean;
  wasm_optimization: 'size' | 'speed' | 'balanced';
}

export interface EditorSettings {
  fontSize: number;
  fontFamily: string;
  lineHeight: number;
  tabSize: number;
  wordWrap: boolean;
  lineNumbers: boolean;
  vimMode: boolean;
  spellCheck: boolean;
  autoSave: boolean;
  autoSaveDelay: number;
  defaultMode: 'wysiwyg' | 'source' | 'split';
  theme: 'light' | 'dark' | 'system';
  milkdownTheme: string;
}

export interface VaultConfig {
  path: string;
  name: string;
  autoIndex: boolean;
  watchForChanges: boolean;
  ignorePatterns: string[];
  maxFileSize: number;
}

export interface PublishTarget {
  id: string;
  name: string;
  type: 'github' | 'gitlab' | 's3' | 'r2' | 'custom' | 'custom_api' | 'rss';
  url?: string;
  branch?: string;
  remoteName?: string;
  endpoint?: string;
  keyId?: string;
  uploadImages: boolean;
  convertWikilinks: boolean;
  stripPrivate: boolean;
  generateRss: boolean;
}

export interface HeaderPair {
  key: string;
  value: string;
}

export interface PublishOptions {
  uploadImages: boolean;
  convertWikilinks: boolean;
  stripPrivate: boolean;
  generateRss: boolean;
  customEndpoint?: string;
  customHeaders?: HeaderPair[];
}

export interface PublicationRecord {
  id: string;
  targetName: string;
  filePath: string;
  publishedAt: string;
  status: string;
  url?: string;
}

export type ImageHostType = 'github' | 'cloudflare';

export interface ImageHost {
  type: ImageHostType;
  repo?: string;
  branch?: string;
  token?: string;
  accountId?: string;
  apiToken?: string;
}

export interface SyncConfig {
  enabled: boolean;
  type: 'git' | 'rsync' | 'rclone' | 'custom';
  intervalMinutes: number;
  autoSync: boolean;
  conflictResolution: 'ours' | 'theirs' | 'manual';
  remoteUrl?: string;
  branch?: string;
}

export interface KnowledgeBase {
  id: string;
  name: string;
  path: string;
  enabled: boolean;
  lastIndexed?: string;
  documentCount?: number;
  indexStatus: 'idle' | 'indexing' | 'ready' | 'error';
}

export interface PluginConfig {
  id: string;
  name: string;
  version: string;
  enabled: boolean;
  source: 'npm' | 'local' | 'url';
  config?: Record<string, unknown>;
}

export interface SearchResult {
  path: string;
  name: string;
  snippet: string;
  score: number;
  matches: { line: number; column: number; length: number }[];
  tags: string[];
}

export interface CommitEntry {
  hash: string;
  message: string;
  author: string;
  email: string;
  timestamp: string;
  filePath?: string;
}

export interface DiffResult {
  hunks: DiffHunk[];
  oldPath: string;
  newPath: string;
  status: 'added' | 'modified' | 'deleted' | 'renamed';
}

export interface DiffHunk {
  oldStart: number;
  oldLines: number;
  newStart: number;
  newLines: number;
  lines: DiffLine[];
}

export interface DiffLine {
  type: 'context' | 'added' | 'removed';
  content: string;
  oldLineNumber?: number;
  newLineNumber?: number;
}

export interface GitStatus {
  branch: string;
  ahead: number;
  behind: number;
  staged: number;
  unstaged: number;
  untracked: number;
  hasConflicts: boolean;
  lastCommit?: CommitEntry;
}

export interface Template {
  id: string;
  name: string;
  description: string;
  prompt: string;
  variables: TemplateVariable[];
  category: string;
  isBuiltin: boolean;
}

export interface TemplateVariable {
  name: string;
  label: string;
  type: 'text' | 'select' | 'textarea' | 'boolean';
  required: boolean;
  defaultValue?: string;
  options?: string[];
  description?: string;
}

export interface McpServerInfo {
  id: string;
  name: string;
  transport: 'stdio' | 'sse';
  command?: string;
  args?: string[];
  url?: string;
  env?: Record<string, string>;
  enabled: boolean;
  status: 'connected' | 'disconnected' | 'error';
  errorMessage?: string;
  tools?: McpTool[];
}

export interface McpTool {
  name: string;
  description: string;
  inputSchema: Record<string, unknown>;
}

export interface KnowledgeBaseInfo {
  id: string;
  name: string;
  path: string;
  enabled: boolean;
  lastIndexed?: string;
  documentCount: number;
  indexStatus: 'idle' | 'indexing' | 'ready' | 'error';
  errorMessage?: string;
}

export interface IndexStats {
  totalDocuments: number;
  indexedDocuments: number;
  lastIndexed?: string;
  indexSizeBytes: number;
  status: 'idle' | 'indexing' | 'ready' | 'error';
}

export interface PluginInfo {
  id: string;
  name: string;
  version: string;
  description: string;
  author: string;
  enabled: boolean;
  wasm_path: string;
  permissions: string[];
  homepage: string | null;
  source: string | null;
}

export interface MarketplaceServerInfo {
  id: string;
  name: string;
  description: string;
  version: string;
  author: string;
  downloads: number;
  rating: number | null;
  transport: string;
  homepage: string | null;
  license: string | null;
}

export interface RagConfig {
  embedding_model: string;
  chunk_strategy: 'fixed' | 'paragraph' | 'semantic';
  chunk_size: number;
  chunk_overlap: number;
  min_score: number;
}

export interface IngestProgress {
  fileName: string;
  percent: number;
  status: 'pending' | 'ingesting' | 'indexing' | 'done' | 'error';
  error?: string;
}

export interface McpConfirmationRequest {
  server: string;
  tool: string;
  args: Record<string, unknown>;
  id: string;
  sensitiveOps: string[];
}
