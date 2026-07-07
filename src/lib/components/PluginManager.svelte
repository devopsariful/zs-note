<script lang="ts">
  import { onMount } from 'svelte';
  import { getPluginsStore } from '../stores/plugins.svelte';
  import MarketplaceBrowser from './MarketplaceBrowser.svelte';
  import type { PluginInfo } from '../types';

  const store = getPluginsStore();

  let selectedPlugin = $state<PluginInfo | null>(null);
  let showMarketplace = $state(false);
  let showInstallPicker = $state(false);
  let installError = $state<string | null>(null);
  let installing = $state(false);

  onMount(() => {
    store.loadPlugins();
  });

  function selectPlugin(plugin: PluginInfo) {
    selectedPlugin = plugin;
  }

  function handleToggle(plugin: PluginInfo) {
    store.togglePlugin(plugin.id, !plugin.enabled).catch((err) => {
      installError = String(err);
    });
  }

  function handleUninstall(plugin: PluginInfo) {
    if (!confirm(`Uninstall plugin "${plugin.name}"?`)) return;
    store.uninstallPlugin(plugin.id).catch((err) => {
      installError = String(err);
    });
    if (selectedPlugin?.id === plugin.id) {
      selectedPlugin = null;
    }
  }

  async function handleFilePick() {
    // Use Tauri dialog to pick a .wasm file
    try {
      const { open } = await import('@tauri-apps/plugin-dialog');
      const selected = await open({
        multiple: false,
        filters: [{ name: 'WASM Plugins', extensions: ['wasm'] }],
      });
      if (selected) {
        installing = true;
        installError = null;
        try {
          await store.installPlugin(selected);
        } catch (err) {
          installError = String(err);
        } finally {
          installing = false;
        }
      }
    } catch (err) {
      installError = String(err);
    }
  }

  function handleCloseMarketplace() {
    showMarketplace = false;
    store.loadPlugins();
  }

  function permissionLabel(perm: string): string {
    switch (perm) {
      case 'fs:read': return 'Read files';
      case 'fs:write': return 'Write files';
      case 'net:http': return 'HTTP requests';
      case 'net:all': return 'Full network';
      case 'ai:chat': return 'AI chat access';
      case 'clipboard:read': return 'Read clipboard';
      case 'clipboard:write': return 'Write clipboard';
      case '*': return 'All permissions';
      default: return perm;
    }
  }
</script>

<div class="plugin-manager">
  <div class="manager-header">
    <h3 class="text-sm font-bold">Plugins</h3>
    <div class="header-actions">
      <button class="btn btn-secondary btn-sm" onclick={() => { showInstallPicker = true; handleFilePick(); }} disabled={installing}>
        {installing ? 'Installing...' : 'Install .wasm'}
      </button>
      <button class="btn btn-primary btn-sm" onclick={() => { showMarketplace = true; }}>
        Browse Marketplace
      </button>
    </div>
  </div>

  {#if installError}
    <div class="error-banner">
      <span class="text-sm" style="color: var(--color-error)">{installError}</span>
      <button class="btn btn-ghost btn-icon" onclick={() => { installError = null; }}>×</button>
    </div>
  {/if}

  <div class="plugin-layout">
    <!-- Plugin list -->
    <div class="plugin-list">
      {#if store.isLoading && store.plugins.length === 0}
        <div class="empty-state text-muted text-sm">Loading plugins...</div>
      {:else if store.plugins.length === 0}
        <div class="empty-state text-muted text-sm">
          No plugins installed.
          <br />
          Install a .wasm file or browse the marketplace.
        </div>
      {:else}
        {#each store.plugins as plugin}
          <div
            class="plugin-card"
            class:selected={selectedPlugin?.id === plugin.id}
            class:disabled={!plugin.enabled}
            onclick={() => selectPlugin(plugin)}
            role="button"
            tabindex="0"
            onkeydown={(e) => { if (e.key === 'Enter') selectPlugin(plugin); }}
          >
            <div class="plugin-info">
              <div class="plugin-name">{plugin.name}</div>
              <span class="badge" class:badge-success={plugin.enabled} class:badge-muted={!plugin.enabled}>
                {plugin.enabled ? 'Enabled' : 'Disabled'}
              </span>
            </div>
            <div class="plugin-meta text-xs text-muted">
              v{plugin.version} · {plugin.author || 'Unknown author'}
            </div>
            <div class="plugin-description text-xs">{plugin.description || 'No description'}</div>
            <div class="plugin-actions">
              <!-- svelte-ignore a11y_click_events_have_key_events a11y_no_noninteractive_element_interactions -->
              <label class="toggle-label" onclick={(e) => e.stopPropagation()}>
                <input
                  type="checkbox"
                  checked={plugin.enabled}
                  onchange={() => handleToggle(plugin)}
                ></input>
                <span class="toggle-slider"></span>
              </label>
              <button
                class="btn btn-ghost btn-sm danger"
                onclick={(e) => { e.stopPropagation(); handleUninstall(plugin); }}
              >
                Uninstall
              </button>
            </div>
          </div>
        {/each}
      {/if}
    </div>

    <!-- Detail panel -->
    {#if selectedPlugin}
      <div class="plugin-detail">
        <h4 class="text-sm font-bold">{selectedPlugin.name}</h4>
        <div class="detail-meta text-xs text-muted">
          <span>Version: {selectedPlugin.version}</span>
          {#if selectedPlugin.author}
            <span>Author: {selectedPlugin.author}</span>
          {/if}
        </div>

        {#if selectedPlugin.description}
          <p class="detail-description text-sm">{selectedPlugin.description}</p>
        {/if}

        <div class="detail-section">
          <h5 class="text-xs font-bold">Permissions</h5>
          {#if selectedPlugin.permissions.length === 0}
            <span class="text-xs text-muted">No permissions requested</span>
          {:else}
            <ul class="permission-list">
              {#each selectedPlugin.permissions as perm}
                <li class="permission-item text-xs">{permissionLabel(perm)}</li>
              {/each}
            </ul>
          {/if}
        </div>

        <div class="detail-section">
          <h5 class="text-xs font-bold">WASM Path</h5>
          <code class="wasm-path text-xs">{selectedPlugin.wasm_path}</code>
        </div>

        {#if selectedPlugin.homepage}
          <div class="detail-section">
            <h5 class="text-xs font-bold">Homepage</h5>
            <a href={selectedPlugin.homepage} target="_blank" rel="noopener" class="text-xs">
              {selectedPlugin.homepage}
            </a>
          </div>
        {/if}
      </div>
    {/if}
  </div>
</div>

{#if showMarketplace}
  <MarketplaceBrowser onclose={handleCloseMarketplace} />
{/if}

<style>
  .plugin-manager {
    display: flex;
    flex-direction: column;
    gap: 12px;
    height: 100%;
  }
  .manager-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .header-actions {
    display: flex;
    gap: 8px;
  }
  .error-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 10px;
    background: var(--color-error-bg);
    border-radius: var(--radius-md);
  }
  .plugin-layout {
    display: flex;
    gap: 12px;
    flex: 1;
    min-height: 0;
  }
  .plugin-list {
    display: flex;
    flex-direction: column;
    gap: 8px;
    flex: 1;
    overflow-y: auto;
  }
  .empty-state {
    text-align: center;
    padding: 24px;
    line-height: 1.6;
  }
  .plugin-card {
    padding: 10px 12px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    cursor: pointer;
    display: flex;
    flex-direction: column;
    gap: 4px;
    transition: border-color var(--transition-fast);
  }
  .plugin-card:hover {
    border-color: var(--color-accent);
  }
  .plugin-card.selected {
    border-color: var(--color-accent);
    background: var(--color-surface);
  }
  .plugin-card.disabled {
    opacity: 0.6;
  }
  .plugin-info {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .plugin-name {
    font-weight: 500;
    font-size: 13px;
  }
  .plugin-meta {
    font-family: var(--font-code);
  }
  .plugin-description {
    color: var(--color-text-muted);
    line-height: 1.4;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    line-clamp: 2;
    overflow: hidden;
  }
  .plugin-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    padding-top: 6px;
    border-top: 1px solid var(--color-border);
  }
  .plugin-actions .danger {
    color: var(--color-error);
    margin-left: auto;
  }
  .plugin-detail {
    flex: 1;
    max-width: 320px;
    padding: 12px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-surface);
    display: flex;
    flex-direction: column;
    gap: 10px;
    overflow-y: auto;
  }
  .detail-meta {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .detail-description {
    line-height: 1.5;
    color: var(--color-text);
  }
  .detail-section {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }
  .permission-list {
    list-style: none;
    display: flex;
    flex-wrap: wrap;
    gap: 4px;
  }
  .permission-item {
    padding: 2px 8px;
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    font-family: var(--font-code);
  }
  .wasm-path {
    padding: 4px 8px;
    background: var(--color-bg);
    border-radius: var(--radius-sm);
    overflow: hidden;
    text-overflow: ellipsis;
    font-family: var(--font-code);
    word-break: break-all;
  }
  .toggle-label {
    position: relative;
    display: inline-block;
    width: 32px;
    height: 18px;
    cursor: pointer;
  }
  .toggle-label input {
    opacity: 0;
    width: 0;
    height: 0;
  }
  .toggle-slider {
    position: absolute;
    inset: 0;
    background: var(--color-border);
    border-radius: 18px;
    transition: background var(--transition-fast);
  }
  .toggle-slider::before {
    content: '';
    position: absolute;
    width: 14px;
    height: 14px;
    left: 2px;
    bottom: 2px;
    background: white;
    border-radius: 50%;
    transition: transform var(--transition-fast);
  }
  .toggle-label input:checked + .toggle-slider {
    background: var(--color-accent);
  }
  .toggle-label input:checked + .toggle-slider::before {
    transform: translateX(14px);
  }
  .badge {
    font-size: 10px;
    font-weight: 600;
    padding: 2px 8px;
    border-radius: 10px;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }
  .badge-success {
    background: var(--color-success-bg);
    color: var(--color-success);
  }
  .badge-muted {
    background: var(--color-surface);
    color: var(--color-text-muted);
  }
</style>
