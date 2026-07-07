<script lang="ts">
  import Modal from './Modal.svelte';
  import MCPManager from './MCPManager.svelte';
  import LocaleSwitcher from '../i18n/LocaleSwitcher.svelte';
  import { getConfigStore } from '../stores/config.svelte';
  import { getAIStore } from '../stores/ai.svelte';
  import { invoke } from '@tauri-apps/api/core';

  const config = getConfigStore();
  const ai = getAIStore();

  let {
    show = false,
    onClose = () => {},
  }: {
    show?: boolean;
    onClose?: () => void;
  } = $props();

  let activeTab = $state('editor');
  let saving = $state(false);
  let error = $state<string | null>(null);
  let displayedError = $derived(error ?? config.error);

  const tabs = [
    { id: 'editor', label: 'Editor' },
    { id: 'ai', label: 'AI' },
    { id: 'sandbox', label: 'Sandbox' },
    { id: 'mcp', label: 'MCP' },
    { id: 'publish', label: 'Publish' },
    { id: 'sync', label: 'Sync' },
    { id: 'plugins', label: 'Plugins' },
    { id: 'security', label: 'Security' },
  ];

  function handleSave() {
    saving = true;
    error = null;

    config.saveAll()
      .then(() => {
        ai.setProviders(config.providers);
      })
      .catch((err) => { error = String(err); })
      .finally(() => { saving = false; });
  }

  function handleNewProvider() {
    config.providers = [
      ...config.providers,
      {
        id: crypto.randomUUID(),
        name: 'New Provider',
        provider_type: 'openai',
        api_key: '',
        base_url: '',
        models: [],
        default_model: '',
        enabled: true,
      },
    ];
  }

  function handleRemoveProvider(id: string) {
    config.providers = config.providers.filter(p => p.id !== id);
  }

  // --- Image Hosting derived from config ---
  let imageHostType = $state<'github' | 'cloudflare'>(config.imageHost?.type ?? 'github');
  let imageRepo = $state(config.imageHost?.repo ?? '');
  let imageBranch = $state(config.imageHost?.branch ?? 'main');
  let imageToken = $state('');
  let imageAccountId = $state(config.imageHost?.accountId ?? '');
  let imageApiToken = $state('');

  // --- RSS Settings State ---
  let rssTitle = $state('ZarishNote Vault');
  let rssDescription = $state('Published notes from ZarishNote');
  let rssMaxItems = $state(50);
  let rssIncludeFullContent = $state(true);

  function handleNewPublishTarget() {
    config.publishTargets = [
      ...config.publishTargets,
      {
        id: crypto.randomUUID(),
        name: 'New Target',
        type: 'github',
        branch: 'main',
        remoteName: 'origin',
        uploadImages: true,
        convertWikilinks: true,
        stripPrivate: false,
        generateRss: false,
      },
    ];
  }

  function handleRemovePublishTarget(id: string) {
    config.publishTargets = config.publishTargets.filter(t => t.id !== id);
  }

  function handleSaveImageHost() {
    config.imageHost = {
      type: imageHostType,
      repo: imageRepo || undefined,
      branch: imageBranch || 'main',
      token: imageToken || undefined,
      accountId: imageAccountId || undefined,
      apiToken: imageApiToken || undefined,
    };
    config.saveImageHost().catch((err) => {
      error = String(err);
    });
  }
</script>

<Modal title="Settings" bind:show size="lg" {onClose}>
  <div class="settings">
    <div class="tabs-sidebar">
      {#each tabs as tab}
        <button
          class="tab-btn"
          class:active={activeTab === tab.id}
          onclick={() => { activeTab = tab.id; }}
        >
          {tab.label}
        </button>
      {/each}
    </div>

    <div class="tab-content">
      {#if displayedError}
        <div class="error-banner text-sm" style="color: var(--color-error); padding: 8px 12px; background: var(--color-error-bg); border-radius: var(--radius-md); margin-bottom: 12px;">
          {displayedError}
        </div>
      {/if}

      <!-- Editor Settings -->
      {#if activeTab === 'editor'}
        <div class="settings-section">
          <h3 class="section-title">Editor Settings</h3>
          <div class="settings-grid">
            <div class="input-group">
              <label>Font Size</label>
              <input type="number" bind:value={config.editorSettings.fontSize} min="10" max="32" />
            </div>
            <div class="input-group">
              <label>Font Family</label>
              <select bind:value={config.editorSettings.fontFamily}>
                <option value="system-ui">System UI</option>
                <option value="'JetBrains Mono', monospace">JetBrains Mono</option>
                <option value="'Fira Code', monospace">Fira Code</option>
                <option value="Georgia, serif">Georgia</option>
              </select>
            </div>
            <div class="input-group">
              <label>Line Height</label>
              <input type="number" bind:value={config.editorSettings.lineHeight} step="0.1" min="1" max="2.5" />
            </div>
            <div class="input-group">
              <label>Tab Size</label>
              <input type="number" bind:value={config.editorSettings.tabSize} min="1" max="8" />
            </div>
            <div class="input-group">
              <label>Default Mode</label>
              <select bind:value={config.editorSettings.defaultMode}>
                <option value="wysiwyg">WYSIWYG</option>
                <option value="source">Source</option>
                <option value="split">Split</option>
              </select>
            </div>
            <div class="input-group">
              <label>Theme</label>
              <select bind:value={config.editorSettings.theme}>
                <option value="system">System</option>
                <option value="light">Light</option>
                <option value="dark">Dark</option>
              </select>
            </div>
            <div class="input-group">
              <label>Language</label>
              <LocaleSwitcher />
            </div>
          </div>
          <div class="settings-checks">
            <label class="check-row">
              <input type="checkbox" bind:checked={config.editorSettings.wordWrap} />
              <span>Word Wrap</span>
            </label>
            <label class="check-row">
              <input type="checkbox" bind:checked={config.editorSettings.lineNumbers} />
              <span>Line Numbers</span>
            </label>
            <label class="check-row">
              <input type="checkbox" bind:checked={config.editorSettings.vimMode} />
              <span>Vim Mode</span>
            </label>
            <label class="check-row">
              <input type="checkbox" bind:checked={config.editorSettings.spellCheck} />
              <span>Spell Check</span>
            </label>
            <label class="check-row">
              <input type="checkbox" bind:checked={config.editorSettings.autoSave} />
              <span>Auto Save</span>
            </label>
          </div>
          {#if config.editorSettings.autoSave}
            <div class="input-group">
              <label>Auto Save Delay (ms)</label>
              <input type="number" bind:value={config.editorSettings.autoSaveDelay} min="1000" max="60000" step="500" />
            </div>
          {/if}
        </div>

      <!-- AI Settings -->
      {:else if activeTab === 'ai'}
        <div class="settings-section">
          <h3 class="section-title">AI Providers</h3>
          {#each config.providers as provider, idx}
            <div class="provider-card">
              <div class="provider-header">
                <input
                  type="text"
                  bind:value={provider.name}
                  class="provider-name-input"
                  placeholder="Provider name"
                />
                <button class="btn btn-ghost btn-sm danger" onclick={() => handleRemoveProvider(provider.id)}>
                  <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
                    <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
                  </svg>
                </button>
              </div>
              <div class="provider-fields">
                <div class="input-group">
                  <label>Type</label>
                  <select bind:value={provider.provider_type}>
                    <option value="openai">OpenAI</option>
                    <option value="anthropic">Anthropic</option>
                    <option value="google">Google</option>
                    <option value="ollama">Ollama</option>
                    <option value="custom">Custom</option>
                  </select>
                </div>
                <div class="input-group">
                  <label>API Key</label>
                  <input type="password" value={provider.api_key ?? ''} oninput={(e) => { provider.api_key = (e.target as HTMLInputElement).value; }} placeholder="sk-..." />
                </div>
                <div class="input-group">
                  <label>Base URL</label>
                  <input type="text" value={provider.base_url ?? ''} oninput={(e) => { provider.base_url = (e.target as HTMLInputElement).value; }} placeholder="https://api.openai.com/v1" />
                </div>
                <div class="input-group">
                  <label>Models (comma-separated)</label>
                  <input
                    type="text"
                    value={provider.models.join(', ')}
                    oninput={(e) => { provider.models = (e.target as HTMLInputElement).value.split(',').map(s => s.trim()).filter(Boolean); }}
                    placeholder="gpt-4, gpt-3.5-turbo"
                  />
                </div>
                <div class="input-group">
                  <label>Default Model</label>
                  <select bind:value={provider.default_model}>
                    <option value="">Select...</option>
                    {#each provider.models as m}
                      <option value={m}>{m}</option>
                    {/each}
                  </select>
                </div>
                <div class="settings-checks">
                  <label class="check-row">
                    <input type="checkbox" checked={provider.enabled} onchange={(e) => { provider.enabled = (e.target as HTMLInputElement).checked; }} />
                    <span>Enabled</span>
                  </label>
                </div>
              </div>
            </div>
          {/each}
          <button class="btn btn-secondary btn-sm" onclick={handleNewProvider}>
            <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
              <path d="M8 3v10M3 8h10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
            </svg>
            Add Provider
          </button>
        </div>

      <!-- Sandbox Settings -->
      {:else if activeTab === 'sandbox'}
        <div class="settings-section">
          <h3 class="section-title">Sandbox Configuration</h3>
          <div class="settings-checks">
            <label class="check-row">
              <input type="checkbox" bind:checked={config.sandboxConfig.enabled} />
              <span>Enable Sandbox</span>
            </label>
          </div>
          <div class="settings-grid">
            <div class="input-group">
              <label>Timeout (ms)</label>
              <input type="number" bind:value={config.sandboxConfig.timeout_ms} min="1000" max="300000" />
            </div>
            <div class="input-group">
              <label>Memory Limit (MB)</label>
              <input type="number" bind:value={config.sandboxConfig.memory_mb} min="16" max="1024" />
            </div>
            <div class="input-group">
              <label>WASM Optimization</label>
              <select bind:value={config.sandboxConfig.wasm_optimization}>
                <option value="size">Size</option>
                <option value="speed">Speed</option>
                <option value="balanced">Balanced</option>
              </select>
            </div>
          </div>
          <div class="settings-checks">
            <label class="check-row">
              <input type="checkbox" bind:checked={config.sandboxConfig.allowed_network} />
              <span>Allow network access</span>
            </label>
          </div>
          <div class="input-group">
            <label>Allowed Filesystem Paths (one per line)</label>
            <textarea
              value={config.sandboxConfig.allowed_fs_paths.join('\n')}
              oninput={(e) => { config.sandboxConfig.allowed_fs_paths = (e.target as HTMLTextAreaElement).value.split('\n').filter(Boolean); }}
              rows={3}
            />
          </div>
        </div>

      <!-- MCP -->
      {:else if activeTab === 'mcp'}
        <div class="settings-section">
          <MCPManager
            servers={config.knowledgeBases.map(kb => ({
              id: kb.id,
              name: kb.name,
              transport: 'stdio' as const,
              enabled: kb.enabled,
              status: (kb.indexStatus === 'ready' ? 'connected' : kb.indexStatus === 'error' ? 'error' : 'disconnected') as 'connected' | 'disconnected' | 'error',
              errorMessage: kb.errorMessage,
            }))}
            onUpdate={() => {}}
          />
        </div>

      <!-- Publish Settings -->
      {:else if activeTab === 'publish'}
        <div class="settings-section">
          <h3 class="section-title">Publish Targets</h3>
          {#each config.publishTargets as target, idx}
            <div class="provider-card">
              <div class="provider-header">
                <input type="text" bind:value={target.name} class="provider-name-input" placeholder="Target name" />
                <button class="btn btn-ghost btn-sm danger" onclick={() => handleRemovePublishTarget(target.id)}>×</button>
              </div>
              <div class="provider-fields">
                <div class="input-group">
                  <label>Type</label>
                  <select bind:value={target.type}>
                    <option value="github">GitHub</option>
                    <option value="gitlab">GitLab</option>
                    <option value="s3">S3</option>
                    <option value="r2">Cloudflare R2</option>
                    <option value="custom_api">Custom API</option>
                    <option value="rss">RSS Feed</option>
                  </select>
                </div>
                {#if target.type === 'github' || target.type === 'gitlab'}
                  <div class="input-group">
                    <label>Repository URL</label>
                    <input type="text" value={target.url ?? ''} oninput={(e) => { target.url = (e.target as HTMLInputElement).value; }} placeholder="https://github.com/user/repo" />
                  </div>
                  <div class="input-group">
                    <label>Branch</label>
                    <input type="text" value={target.branch ?? ''} oninput={(e) => { target.branch = (e.target as HTMLInputElement).value; }} placeholder="main" />
                  </div>
                {:else if target.type === 'custom_api'}
                  <div class="input-group">
                    <label>API Endpoint</label>
                    <input type="text" value={target.endpoint ?? ''} oninput={(e) => { target.endpoint = (e.target as HTMLInputElement).value; }} placeholder="https://api.example.com/publish" />
                  </div>
                {:else if target.type === 'rss'}
                  <div class="input-group">
                    <label>Output Path (optional)</label>
                    <input type="text" value={target.url ?? ''} oninput={(e) => { target.url = (e.target as HTMLInputElement).value; }} placeholder="rss.xml" />
                  </div>
                {:else}
                  <div class="input-group">
                    <label>URL / Endpoint</label>
                    <input type="text" value={target.url ?? ''} oninput={(e) => { target.url = (e.target as HTMLInputElement).value; }} />
                  </div>
                  <div class="input-group">
                    <label>Branch</label>
                    <input type="text" value={target.branch ?? ''} oninput={(e) => { target.branch = (e.target as HTMLInputElement).value; }} />
                  </div>
                {/if}
                <!-- Default publish options per target -->
                <div class="settings-checks" style="margin-top: 6px;">
                  <label class="check-row">
                    <input type="checkbox" checked={target.uploadImages} onchange={(e) => { target.uploadImages = (e.target as HTMLInputElement).checked; }} />
                    <span>Upload images</span>
                  </label>
                  <label class="check-row">
                    <input type="checkbox" checked={target.convertWikilinks} onchange={(e) => { target.convertWikilinks = (e.target as HTMLInputElement).checked; }} />
                    <span>Convert [[wikilinks]]</span>
                  </label>
                  <label class="check-row">
                    <input type="checkbox" checked={target.stripPrivate} onchange={(e) => { target.stripPrivate = (e.target as HTMLInputElement).checked; }} />
                    <span>Strip private tags</span>
                  </label>
                  <label class="check-row">
                    <input type="checkbox" checked={target.generateRss} onchange={(e) => { target.generateRss = (e.target as HTMLInputElement).checked; }} />
                    <span>Generate RSS feed</span>
                  </label>
                </div>
              </div>
            </div>
          {/each}
          <button class="btn btn-secondary btn-sm" onclick={handleNewPublishTarget}>
            <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
              <path d="M8 3v10M3 8h10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
            </svg>
            Add Target
          </button>
        </div>

        <!-- Image Hosting Configuration -->
        <div class="settings-section" style="margin-top: 16px;">
          <h3 class="section-title">Image Hosting</h3>
          <p class="text-muted text-sm">Configure where images are uploaded when publishing.</p>
          <div class="input-group">
            <label>Host Type</label>
            <select bind:value={imageHostType}>
              <option value="github">GitHub Repository</option>
              <option value="cloudflare">Cloudflare Images</option>
            </select>
          </div>
          {#if imageHostType === 'github'}
            <div class="input-group">
              <label>Repository (owner/repo)</label>
              <input type="text" bind:value={imageRepo} placeholder="username/repository" />
            </div>
            <div class="input-group">
              <label>Branch</label>
              <input type="text" bind:value={imageBranch} placeholder="main" />
            </div>
            <div class="input-group">
              <label>Access Token</label>
              <input type="password" bind:value={imageToken} placeholder="GitHub Personal Access Token" />
            </div>
          {:else}
            <div class="input-group">
              <label>Cloudflare Account ID</label>
              <input type="text" bind:value={imageAccountId} placeholder="Account ID" />
            </div>
            <div class="input-group">
              <label>API Token</label>
              <input type="password" bind:value={imageApiToken} placeholder="Cloudflare API Token" />
            </div>
          {/if}
          <button class="btn btn-secondary btn-sm" onclick={handleSaveImageHost}>
            Save Image Host Config
          </button>
        </div>

        <!-- RSS Feed Settings -->
        <div class="settings-section" style="margin-top: 16px;">
          <h3 class="section-title">RSS Feed Settings</h3>
          <div class="input-group">
            <label>Feed Title</label>
            <input type="text" bind:value={rssTitle} placeholder="ZarishNote Vault" />
          </div>
          <div class="input-group">
            <label>Feed Description</label>
            <input type="text" bind:value={rssDescription} placeholder="Published notes from ZarishNote" />
          </div>
          <div class="input-group">
            <label>Max Items</label>
            <input type="number" bind:value={rssMaxItems} min="10" max="500" />
          </div>
          <div class="settings-checks">
            <label class="check-row">
              <input type="checkbox" bind:checked={rssIncludeFullContent} />
              <span>Include full content in RSS</span>
            </label>
          </div>
        </div>

      <!-- Sync Settings -->
      {:else if activeTab === 'sync'}
        <div class="settings-section">
          <h3 class="section-title">Sync Configuration</h3>
          <div class="settings-checks">
            <label class="check-row">
              <input type="checkbox" bind:checked={config.syncConfig.enabled} />
              <span>Enable Sync</span>
            </label>
            <label class="check-row">
              <input type="checkbox" bind:checked={config.syncConfig.autoSync} />
              <span>Auto Sync</span>
            </label>
          </div>
          <div class="settings-grid">
            <div class="input-group">
              <label>Type</label>
              <select bind:value={config.syncConfig.type}>
                <option value="git">Git</option>
                <option value="rsync">rsync</option>
                <option value="rclone">rclone</option>
                <option value="custom">Custom</option>
              </select>
            </div>
            <div class="input-group">
              <label>Interval (minutes)</label>
              <input type="number" bind:value={config.syncConfig.intervalMinutes} min="1" max="1440" />
            </div>
            <div class="input-group">
              <label>Remote URL</label>
              <input type="text" value={config.syncConfig.remoteUrl ?? ''} oninput={(e) => { config.syncConfig.remoteUrl = (e.target as HTMLInputElement).value; }} />
            </div>
            <div class="input-group">
              <label>Branch</label>
              <input type="text" value={config.syncConfig.branch ?? ''} oninput={(e) => { config.syncConfig.branch = (e.target as HTMLInputElement).value; }} />
            </div>
            <div class="input-group">
              <label>Conflict Resolution</label>
              <select bind:value={config.syncConfig.conflictResolution}>
                <option value="ours">Keep Ours</option>
                <option value="theirs">Keep Theirs</option>
                <option value="manual">Manual</option>
              </select>
            </div>
          </div>
        </div>

      <!-- Plugins -->
      {:else if activeTab === 'plugins'}
        <div class="settings-section">
          <h3 class="section-title">Plugins</h3>
          <p class="text-muted text-sm">Plugin management is not yet available. Check back in a future release.</p>
          <div class="empty-state">
            <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" style="opacity: 0.3;">
              <path d="M12 22s8-4 8-10V5l-8-3-8 3v7c0 6 8 10 8 10z" />
            </svg>
          </div>
        </div>

      <!-- Security -->
      {:else if activeTab === 'security'}
        <div class="settings-section">
          <h3 class="section-title">Security Settings</h3>
          <p class="text-muted text-sm">Security and privacy settings will be available in a future update.</p>
          {#each config.providers.filter(p => p.api_key) as provider}
            <div class="provider-card">
              <div class="text-sm">
                <strong>{provider.name}</strong> — API key stored
              </div>
            </div>
          {/each}
        </div>
      {/if}

      <!-- Save Button -->
      <div class="save-bar">
        <button class="btn btn-primary" onclick={handleSave} disabled={saving}>
          {saving ? 'Saving...' : 'Save Settings'}
        </button>
      </div>
    </div>
  </div>
</Modal>

<style>
  .settings {
    display: flex;
    gap: 16px;
    min-height: 400px;
  }
  .tabs-sidebar {
    width: 140px;
    min-width: 120px;
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .tab-btn {
    display: block;
    width: 100%;
    padding: 8px 12px;
    text-align: left;
    font-size: 13px;
    font-weight: 500;
    border-radius: var(--radius-sm);
    transition: all var(--transition-fast);
  }
  .tab-btn:hover {
    background: var(--color-surface);
  }
  .tab-btn.active {
    background: color-mix(in srgb, var(--color-accent) 12%, transparent);
    color: var(--color-accent);
  }
  .tab-content {
    flex: 1;
    overflow-y: auto;
    min-width: 0;
  }
  .settings-section {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .section-title {
    font-size: 15px;
    font-weight: 600;
    margin-bottom: 4px;
  }
  .settings-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
  }
  .settings-checks {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .check-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    cursor: pointer;
  }
  .check-row input[type="checkbox"] {
    width: 14px;
    height: 14px;
  }
  .provider-card {
    padding: 10px 12px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
  }
  .provider-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 8px;
  }
  .provider-name-input {
    font-size: 14px;
    font-weight: 600;
    flex: 1;
    padding: 4px 8px;
  }
  .provider-fields {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .save-bar {
    display: flex;
    justify-content: flex-end;
    padding-top: 12px;
    border-top: 1px solid var(--color-border);
    margin-top: 12px;
  }
  .empty-state {
    display: flex;
    justify-content: center;
    padding: 24px;
  }
  .danger {
    color: var(--color-error);
  }
</style>
