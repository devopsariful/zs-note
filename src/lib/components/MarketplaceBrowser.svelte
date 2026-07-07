<script lang="ts">
  import { onMount } from 'svelte';
  import { getPluginsStore } from '../stores/plugins.svelte';
  import type { MarketplaceServerInfo } from '../types';

  let {
    onclose = () => {},
  }: {
    onclose?: () => void;
  } = $props();

  const store = getPluginsStore();

  let searchQuery = $state('');
  let selectedCategory = $state<string>('');
  let installingId = $state<string | null>(null);
  let installError = $state<string | null>(null);

  const categories = [
    { id: '', label: 'All' },
    { id: 'ai', label: 'AI' },
    { id: 'storage', label: 'Storage' },
    { id: 'productivity', label: 'Productivity' },
    { id: 'dev-tools', label: 'Dev Tools' },
  ];

  const filteredServers = $derived.by(() => {
    let result = store.marketplaceServers;

    if (searchQuery.trim()) {
      const q = searchQuery.toLowerCase();
      result = result.filter(
        (s) =>
          s.name.toLowerCase().includes(q) ||
          s.description.toLowerCase().includes(q) ||
          s.author.toLowerCase().includes(q),
      );
    }

    return result;
  });

  onMount(() => {
    store.fetchMarketplace();
  });

  function handleRefresh() {
    store.fetchMarketplace();
  }

  async function handleInstall(server: MarketplaceServerInfo) {
    installingId = server.id;
    installError = null;
    try {
      await store.installFromMarketplace(server.id);
    } catch (err) {
      installError = String(err);
    } finally {
      installingId = null;
    }
  }

  function categoryIcon(cat: string): string {
    switch (cat) {
      case 'ai': return '🤖';
      case 'storage': return '💾';
      case 'productivity': return '⚡';
      case 'dev-tools': return '🔧';
      default: return '📦';
    }
  }
</script>

<div class="marketplace-overlay" role="dialog" aria-label="Marketplace Browser">
  <div class="marketplace-panel">
    <!-- Header -->
    <div class="panel-header">
      <div class="panel-title">
        <h3 class="text-sm font-bold">MCP Server Marketplace</h3>
        <span class="text-xs text-muted">{store.marketplaceServers.length} servers available</span>
      </div>
      <div class="panel-actions">
        <button class="btn btn-secondary btn-sm" onclick={handleRefresh} disabled={store.marketplaceLoading}>
          {store.marketplaceLoading ? 'Loading...' : 'Refresh'}
        </button>
        <button class="btn btn-ghost btn-sm" onclick={onclose}>✕</button>
      </div>
    </div>

    {#if installError}
      <div class="error-banner">
        <span class="text-sm" style="color: var(--color-error)">{installError}</span>
        <button class="btn btn-ghost btn-icon" onclick={() => { installError = null; }}>×</button>
      </div>
    {/if}

    <!-- Search and filters -->
    <div class="search-filters">
      <div class="search-wrapper">
        <svg class="search-icon" width="14" height="14" viewBox="0 0 16 16" fill="none">
          <circle cx="7" cy="7" r="5.5" stroke="currentColor" stroke-width="1.5" />
          <path d="M11 11l3.5 3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
        <input
          type="text"
          class="search-input"
          placeholder="Search servers..."
          bind:value={searchQuery}
        />
      </div>
      <div class="category-filters">
        {#each categories as cat}
          <button
            class="category-chip"
            class:active={selectedCategory === cat.id}
            onclick={() => { selectedCategory = cat.id; }}
          >
            {#if cat.id}
              <span class="category-icon">{categoryIcon(cat.id)}</span>
            {/if}
            {cat.label}
          </button>
        {/each}
      </div>
    </div>

    <!-- Server list -->
    <div class="server-grid">
      {#if store.marketplaceLoading && store.marketplaceServers.length === 0}
        <div class="empty-state text-muted text-sm">Fetching marketplace listing...</div>
      {:else if filteredServers.length === 0}
        <div class="empty-state text-muted text-sm">
          {#if searchQuery}
            No servers match "{searchQuery}"
          {:else}
            No servers available yet. Check back later.
          {/if}
        </div>
      {:else}
        {#each filteredServers as server}
          <div class="server-card">
            <div class="server-card-header">
              <div class="server-name-group">
                <span class="server-name">{server.name}</span>
                <span class="badge badge-transport">{server.transport}</span>
              </div>
              {#if server.rating}
                <span class="rating" title="Rating">
                  <svg width="12" height="12" viewBox="0 0 16 16" fill="currentColor">
                    <path d="M8 1.5l1.76 3.57 3.94.57-2.85 2.78.67 3.93L8 10.9l-3.52 1.85.67-3.93L2.3 5.64l3.94-.57L8 1.5z" />
                  </svg>
                  {server.rating.toFixed(1)}
                </span>
              {/if}
            </div>
            <p class="server-description text-xs">{server.description}</p>
            <div class="server-meta text-xs text-muted">
              <span>v{server.version}</span>
              {#if server.author}
                <span>by {server.author}</span>
              {/if}
              <span>{server.downloads.toLocaleString()} downloads</span>
            </div>
            {#if server.license}
              <span class="license-badge text-xs">{server.license}</span>
            {/if}
            <div class="server-card-actions">
              {#if server.homepage}
                <a
                  href={server.homepage}
                  target="_blank"
                  rel="noopener"
                  class="btn btn-ghost btn-sm"
                >
                  Homepage
                </a>
              {/if}
              <button
                class="btn btn-primary btn-sm"
                onclick={() => handleInstall(server)}
                disabled={installingId === server.id}
              >
                {installingId === server.id ? 'Installing...' : 'Install'}
              </button>
            </div>
          </div>
        {/each}
      {/if}
    </div>
  </div>
</div>

<style>
  .marketplace-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.4);
    z-index: var(--z-modal, 50);
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .marketplace-panel {
    width: 90%;
    max-width: 720px;
    max-height: 85vh;
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    display: flex;
    flex-direction: column;
    overflow: hidden;
    box-shadow: var(--shadow-lg);
  }
  .panel-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 14px 16px;
    border-bottom: 1px solid var(--color-border);
  }
  .panel-title {
    display: flex;
    align-items: baseline;
    gap: 8px;
  }
  .panel-actions {
    display: flex;
    gap: 8px;
  }
  .error-banner {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 6px 10px;
    margin: 0 16px;
    background: var(--color-error-bg);
    border-radius: var(--radius-md);
  }
  .search-filters {
    display: flex;
    flex-direction: column;
    gap: 10px;
    padding: 12px 16px;
    border-bottom: 1px solid var(--color-border);
  }
  .search-wrapper {
    position: relative;
    display: flex;
    align-items: center;
  }
  .search-icon {
    position: absolute;
    left: 10px;
    color: var(--color-text-muted);
    pointer-events: none;
  }
  .search-input {
    width: 100%;
    padding: 8px 10px 8px 30px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    background: var(--color-surface);
    color: var(--color-text);
    font-size: 13px;
    outline: none;
  }
  .search-input:focus {
    border-color: var(--color-accent);
  }
  .category-filters {
    display: flex;
    gap: 6px;
    flex-wrap: wrap;
  }
  .category-chip {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 4px 12px;
    border: 1px solid var(--color-border);
    border-radius: 16px;
    background: var(--color-surface);
    color: var(--color-text);
    font-size: 12px;
    cursor: pointer;
    transition: all var(--transition-fast);
  }
  .category-chip:hover {
    border-color: var(--color-accent);
  }
  .category-chip.active {
    background: var(--color-accent);
    color: white;
    border-color: var(--color-accent);
  }
  .category-icon {
    font-size: 12px;
  }
  .server-grid {
    flex: 1;
    overflow-y: auto;
    padding: 12px 16px;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 10px;
  }
  .empty-state {
    grid-column: 1 / -1;
    text-align: center;
    padding: 40px 24px;
  }
  .btn-xs {
    font-size: 11px;
    padding: 2px 6px;
  }
  .server-card {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 12px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    transition: border-color var(--transition-fast);
  }
  .server-card:hover {
    border-color: var(--color-accent);
  }
  .server-card-header {
    display: flex;
    align-items: flex-start;
    justify-content: space-between;
  }
  .server-name-group {
    display: flex;
    align-items: center;
    gap: 6px;
  }
  .server-name {
    font-weight: 600;
    font-size: 13px;
  }
  .badge-transport {
    font-size: 9px;
    padding: 1px 6px;
    border-radius: 8px;
    background: var(--color-surface);
    color: var(--color-text-muted);
    border: 1px solid var(--color-border);
    text-transform: uppercase;
    font-weight: 600;
  }
  .rating {
    display: flex;
    align-items: center;
    gap: 2px;
    font-size: 11px;
    color: #f59e0b;
  }
  .server-description {
    color: var(--color-text-muted);
    line-height: 1.4;
    display: -webkit-box;
    -webkit-line-clamp: 2;
    -webkit-box-orient: vertical;
    line-clamp: 2;
    overflow: hidden;
  }
  .server-meta {
    display: flex;
    gap: 8px;
    font-family: var(--font-code);
  }
  .license-badge {
    align-self: flex-start;
    padding: 1px 6px;
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
    font-family: var(--font-code);
  }
  .server-card-actions {
    display: flex;
    align-items: center;
    justify-content: flex-end;
    gap: 6px;
    padding-top: 8px;
    border-top: 1px solid var(--color-border);
    margin-top: auto;
  }
</style>
