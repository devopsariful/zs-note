import type {
  EditorSettings,
  VaultConfig,
  ProviderConfig,
  SandboxConfig,
  SyncConfig,
  PublishTarget,
  KnowledgeBaseInfo,
  PluginConfig,
  ImageHost,
} from '../types';
import { invoke } from '@tauri-apps/api/core';

type BackendAIConfig = {
  provider: string;
  model: string;
  api_key?: string | null;
  base_url?: string | null;
  max_tokens: number;
  temperature: number;
};

type BackendThemeConfig = {
  name: string;
  background: string;
  foreground: string;
  accent: string;
  font_family: string;
  font_size: number;
};

type BackendToolConfig = {
  name: string;
  wasmPath: string;
  permissions: string[];
  memoryLimit: number;
  timeout: number;
};

type BackendSandboxConfig = {
  enabled: boolean;
  default_memory_limit: number;
  default_timeout: number;
  allowed_domains: string[];
  max_module_size: number;
  tools: BackendToolConfig[];
};

type BackendRemoteConfig = {
  url: string;
  branch: string;
  sshKey?: string | null;
  autoSync: boolean;
  syncInterval: number;
};

type BackendGitConfig = {
  auto_commit: boolean;
  commit_style: string;
  remote?: BackendRemoteConfig | null;
};

type BackendSyncConfig = {
  autoCommit: boolean;
  commitStyle: string;
  remote?: BackendRemoteConfig | null;
};

type BackendKeyBindings = {
  save: string;
  search: string;
  command_palette: string;
  toggle_sidebar: string;
  toggle_preview: string;
};

type BackendPluginsConfig = {
  enabled: string[];
  settings: Record<string, unknown>;
};

type BackendVoiceConfig = {
  enabled: boolean;
  language: string;
  model: string;
};

type BackendKnowledgeBase = {
  name: string;
  path: string;
  formats: string[];
  indexOnStart: boolean;
};

type BackendPublishTarget = {
  name: string;
  type: string;
  repo?: string | null;
  endpoint?: string | null;
  branch?: string | null;
  keyId?: string | null;
};

type BackendVaultConfig = {
  name: string;
  path: string;
  vault_type: string;
  created_at: string;
  version: string;
};

type BackendEditorSettings = {
  theme: string;
  fontSize: number;
  fontFamily: string;
  lineHeight: number;
  proseWidth: number;
  vimMode: boolean;
  spellCheck: boolean;
  autoSave: boolean;
};

type BackendConfig = {
  version: string;
  vault: BackendVaultConfig;
  editor: BackendEditorSettings;
  ai: BackendAIConfig;
  theme: BackendThemeConfig;
  sandbox: BackendSandboxConfig;
  git: BackendGitConfig;
  sync: BackendSyncConfig;
  keys: BackendKeyBindings;
  plugins: BackendPluginsConfig;
  voice: BackendVoiceConfig;
  knowledge: BackendKnowledgeBase[];
  publish: BackendPublishTarget[];
  imageHost?: ImageHost | null;
  features: string[];
};

let editorSettings = $state<EditorSettings>({
  fontSize: 16,
  fontFamily: 'system-ui',
  lineHeight: 1.7,
  tabSize: 2,
  wordWrap: true,
  lineNumbers: true,
  vimMode: false,
  spellCheck: false,
  autoSave: true,
  autoSaveDelay: 3000,
  defaultMode: 'wysiwyg',
  theme: 'system',
  milkdownTheme: 'default',
});

let vaultConfig = $state<VaultConfig>({
  path: '',
  name: 'My Vault',
  autoIndex: true,
  watchForChanges: true,
  ignorePatterns: ['node_modules', '.git', '.obsidian'],
  maxFileSize: 10485760,
});

let providers = $state<ProviderConfig[]>([]);
let sandboxConfig = $state<SandboxConfig>({
  enabled: false,
  timeout_ms: 30000,
  memory_mb: 128,
  allowed_fs_paths: [],
  allowed_network: false,
  wasm_optimization: 'balanced',
});

let syncConfig = $state<SyncConfig>({
  enabled: false,
  type: 'git',
  intervalMinutes: 30,
  autoSync: false,
  conflictResolution: 'manual',
});

let publishTargets = $state<PublishTarget[]>([]);
let imageHost = $state<ImageHost | null>(null);

let knowledgeBases = $state<KnowledgeBaseInfo[]>([]);
let plugins = $state<PluginConfig[]>([]);

let currentConfig = $state<BackendConfig>(createDefaultBackendConfig());
let isLoading = $state(false);
let error = $state<string | null>(null);

function createDefaultBackendConfig(): BackendConfig {
  const now = new Date().toISOString();

  return {
    version: '1',
    vault: {
      name: 'My Vault',
      path: '.',
      vault_type: 'local',
      created_at: now,
      version: '1',
    },
    editor: {
      theme: 'zarish-light',
      fontSize: 16,
      fontFamily: 'Inter, sans-serif',
      lineHeight: 1.7,
      proseWidth: 720,
      vimMode: false,
      spellCheck: true,
      autoSave: true,
    },
    ai: {
      provider: 'ollama',
      model: 'llama3.2',
      api_key: null,
      base_url: 'http://localhost:11434',
      max_tokens: 4096,
      temperature: 0.7,
    },
    theme: {
      name: 'zarish-light',
      background: '#ffffff',
      foreground: '#1a1a2e',
      accent: '#7c3aed',
      font_family: 'Inter, sans-serif',
      font_size: 16,
    },
    sandbox: {
      enabled: true,
      default_memory_limit: 64 * 1024 * 1024,
      default_timeout: 30000,
      allowed_domains: ['*.example.com'],
      max_module_size: 10 * 1024 * 1024,
      tools: [],
    },
    git: {
      auto_commit: true,
      commit_style: 'conventional',
      remote: null,
    },
    sync: {
      autoCommit: true,
      commitStyle: 'conventional',
      remote: null,
    },
    keys: {
      save: 'ctrl+s',
      search: 'ctrl+p',
      command_palette: 'ctrl+shift+p',
      toggle_sidebar: 'ctrl+b',
      toggle_preview: 'ctrl+shift+v',
    },
    plugins: {
      enabled: [],
      settings: {},
    },
    voice: {
      enabled: false,
      language: 'en-US',
      model: 'base',
    },
    knowledge: [],
    publish: [],
    imageHost: null,
    features: ['sandbox', 'ai', 'git', 'search'],
  };
}

function toPathString(path: unknown): string {
  if (typeof path === 'string') return path;
  return path == null ? '' : String(path);
}

function toEditorTheme(theme: string): EditorSettings['theme'] {
  if (theme === 'light' || theme === 'zarish-light') return 'light';
  if (theme === 'dark' || theme === 'zarish-dark') return 'dark';
  return 'system';
}

function applyBackendConfig(config: BackendConfig): void {
  currentConfig = config;

  vaultConfig = {
    ...vaultConfig,
    name: config.vault.name,
    path: toPathString(config.vault.path),
  };

  editorSettings = {
    ...editorSettings,
    fontSize: config.editor.fontSize,
    fontFamily: config.editor.fontFamily,
    lineHeight: config.editor.lineHeight,
    vimMode: config.editor.vimMode,
    spellCheck: config.editor.spellCheck,
    autoSave: config.editor.autoSave,
    theme: toEditorTheme(config.editor.theme),
  };

  providers = [{
    id: config.ai.provider,
    name: config.ai.provider,
    provider_type: config.ai.provider as import('../types').ProviderType,
    api_key: config.ai.api_key ?? '',
    base_url: config.ai.base_url ?? '',
    models: config.ai.model ? [config.ai.model] : [],
    default_model: config.ai.model,
    enabled: true,
    temperature: config.ai.temperature,
    max_tokens: config.ai.max_tokens,
  }];

  sandboxConfig = {
    ...sandboxConfig,
    enabled: config.sandbox.enabled,
    timeout_ms: config.sandbox.default_timeout,
    memory_mb: Math.max(1, Math.round(config.sandbox.default_memory_limit / (1024 * 1024))),
    allowed_network: config.sandbox.allowed_domains.length > 0,
  };

  syncConfig = {
    ...syncConfig,
    enabled: config.sync.autoCommit,
    type: 'git',
    autoSync: config.sync.remote?.autoSync ?? false,
    intervalMinutes: Math.max(1, Math.round((config.sync.remote?.syncInterval ?? 1_800_000) / 60000)),
    remoteUrl: config.sync.remote?.url,
    branch: config.sync.remote?.branch,
  };

  publishTargets = config.publish.map((target, index) => ({
    id: `${target.name || 'target'}-${index}`,
    name: target.name,
    type: target.type as PublishTarget['type'],
    url: target.endpoint ?? undefined,
    branch: target.branch ?? undefined,
    remoteName: target.repo ?? undefined,
    endpoint: target.endpoint ?? undefined,
    keyId: target.keyId ?? undefined,
    uploadImages: true,
    convertWikilinks: true,
    stripPrivate: false,
    generateRss: false,
  }));

  imageHost = config.imageHost ?? null;
  plugins = config.plugins.enabled.map((id) => ({
    id,
    name: id,
    version: 'unknown',
    enabled: true,
    source: 'local',
    config: config.plugins.settings[id] as Record<string, unknown> | undefined,
  }));
}

function buildConfigForSave(): BackendConfig {
  const provider = providers.find((item) => item.enabled) ?? providers[0];
  const remote = syncConfig.remoteUrl || syncConfig.branch || syncConfig.autoSync
    ? {
        url: syncConfig.remoteUrl ?? currentConfig.sync.remote?.url ?? '',
        branch: syncConfig.branch ?? currentConfig.sync.remote?.branch ?? 'main',
        sshKey: currentConfig.sync.remote?.sshKey ?? null,
        autoSync: syncConfig.autoSync,
        syncInterval: syncConfig.intervalMinutes * 60_000,
      }
    : null;

  return {
    ...currentConfig,
    vault: {
      ...currentConfig.vault,
      name: vaultConfig.name,
      path: vaultConfig.path || '.',
    },
    editor: {
      ...currentConfig.editor,
      theme: editorSettings.theme,
      fontSize: editorSettings.fontSize,
      fontFamily: editorSettings.fontFamily,
      lineHeight: editorSettings.lineHeight,
      vimMode: editorSettings.vimMode,
      spellCheck: editorSettings.spellCheck,
      autoSave: editorSettings.autoSave,
    },
    ai: provider
      ? {
          ...currentConfig.ai,
          provider: provider.provider_type,
          model: provider.default_model || provider.models[0] || currentConfig.ai.model,
          api_key: provider.api_key || null,
          base_url: provider.base_url || null,
          max_tokens: provider.max_tokens ?? currentConfig.ai.max_tokens,
          temperature: provider.temperature ?? currentConfig.ai.temperature,
        }
      : currentConfig.ai,
    sandbox: {
      ...currentConfig.sandbox,
      enabled: sandboxConfig.enabled,
      default_timeout: sandboxConfig.timeout_ms,
      default_memory_limit: sandboxConfig.memory_mb * 1024 * 1024,
      allowed_domains: sandboxConfig.allowed_network ? currentConfig.sandbox.allowed_domains : [],
    },
    sync: {
      ...currentConfig.sync,
      autoCommit: syncConfig.enabled,
      remote,
    },
    publish: publishTargets.map((target) => ({
      name: target.name,
      type: target.type,
      repo: target.remoteName ?? null,
      endpoint: target.endpoint ?? target.url ?? null,
      branch: target.branch ?? null,
      keyId: target.keyId ?? null,
    })),
    imageHost,
    plugins: {
      ...currentConfig.plugins,
      enabled: plugins.filter((plugin) => plugin.enabled).map((plugin) => plugin.id),
      settings: plugins.reduce<Record<string, unknown>>((settings, plugin) => {
        if (plugin.config) settings[plugin.id] = plugin.config;
        return settings;
      }, { ...currentConfig.plugins.settings }),
    },
  };
}

async function loadAll(): Promise<void> {
  isLoading = true;
  error = null;

  try {
    const config = await invoke<BackendConfig>('get_config');
    applyBackendConfig(config);
  } catch (err) {
    error = `Failed to load settings: ${String(err)}`;
    throw err;
  } finally {
    isLoading = false;
  }
}

async function saveAll(): Promise<void> {
  try {
    const nextConfig = buildConfigForSave();
    await invoke('update_config', { newConfig: nextConfig });
    currentConfig = nextConfig;
    error = null;
  } catch (err) {
    error = `Failed to save settings: ${String(err)}`;
    throw err;
  }
}

async function saveEditorSettings(): Promise<void> {
  await saveAll();
}

async function saveVaultConfig(): Promise<void> {
  await saveAll();
}

async function saveProviders(): Promise<void> {
  await saveAll();
}

async function saveSandboxConfig(): Promise<void> {
  await saveAll();
}

async function saveSyncConfig(): Promise<void> {
  await saveAll();
}

async function savePublishTargets(): Promise<void> {
  await saveAll();
}

async function saveImageHost(): Promise<void> {
  await saveAll();
}

export function getConfigStore() {
  return {
    get editorSettings() { return editorSettings; },
    set editorSettings(v: EditorSettings) { editorSettings = v; },
    get vaultConfig() { return vaultConfig; },
    set vaultConfig(v: VaultConfig) { vaultConfig = v; },
    get providers() { return providers; },
    set providers(v: ProviderConfig[]) { providers = v; },
    get sandboxConfig() { return sandboxConfig; },
    set sandboxConfig(v: SandboxConfig) { sandboxConfig = v; },
    get syncConfig() { return syncConfig; },
    set syncConfig(v: SyncConfig) { syncConfig = v; },
    get publishTargets() { return publishTargets; },
    set publishTargets(v: PublishTarget[]) { publishTargets = v; },
    get imageHost() { return imageHost; },
    set imageHost(v: ImageHost | null) { imageHost = v; },
    get knowledgeBases() { return knowledgeBases; },
    set knowledgeBases(v: KnowledgeBaseInfo[]) { knowledgeBases = v; },
    get plugins() { return plugins; },
    set plugins(v: PluginConfig[]) { plugins = v; },
    get isLoading() { return isLoading; },
    get error() { return error; },
    loadAll,
    saveAll,
    saveEditorSettings,
    saveVaultConfig,
    saveProviders,
    saveSandboxConfig,
    saveSyncConfig,
    savePublishTargets,
    saveImageHost,
  };
}
