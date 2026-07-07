<script lang="ts">
  import { getFilesStore } from '../stores/files.svelte';
  import { getEditorStore } from '../stores/editor.svelte';
  import FileTree from './FileTree.svelte';
  import ImportDialog from './ImportDialog.svelte';
  import { toVaultRelativePath } from '../utils/vaultPath';

  const files = getFilesStore();
  const editor = getEditorStore();

  let searchInput = $state('');
  let showImportDialog = $state(false);

  function handleSearchInput() {
    files.searchFiles(searchInput);
  }

  function handleNewFile() {
    const name = prompt('File name:');
    if (name) {
      files.createFile(toVaultRelativePath(name));
    }
  }

  function handleImport() {
    showImportDialog = true;
  }

  function handleImported(_paths: string[]) {
    showImportDialog = false;
    files.loadFileTree();
  }

  const sections = $state([
    { id: 'files', label: 'Files', collapsed: false },
    { id: 'tags', label: 'Tags', collapsed: true },
    { id: 'recent', label: 'Recent', collapsed: true },
  ]);

  function toggleSection(id: string) {
    const section = sections.find(s => s.id === id);
    if (section) section.collapsed = !section.collapsed;
  }
</script>

<aside class="sidebar">
  <div class="sidebar-header">
    <div class="search-wrapper">
      <svg class="search-icon" width="14" height="14" viewBox="0 0 16 16" fill="none">
        <circle cx="7" cy="7" r="5.5" stroke="currentColor" stroke-width="1.5" />
        <path d="M11 11l3.5 3.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
      </svg>
      <input
        type="text"
        class="search-input"
        placeholder="Search files..."
        bind:value={searchInput}
        oninput={handleSearchInput}
      />
    </div>
  </div>

  <div class="sidebar-actions">
    <button class="btn btn-secondary btn-sm" onclick={handleNewFile}>
      <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
        <path d="M8 3v10M3 8h10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
      </svg>
      New Note
    </button>
    <button class="btn btn-secondary btn-sm" onclick={handleImport}>
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
        <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4" />
        <polyline points="17 8 12 3 7 8" />
        <line x1="12" y1="3" x2="12" y2="15" />
      </svg>
      Import
    </button>
  </div>

  <div class="sidebar-content">

    <!-- Files Section -->
    <div class="section">
      <button class="section-header" onclick={() => toggleSection('files')}>
        <svg
          width="12"
          height="12"
          viewBox="0 0 16 16"
          fill="none"
          class:collapsed={sections[0].collapsed}
        >
          <path d="M6 4l4 4-4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
        <span class="section-label">Files</span>
      </button>
      {#if !sections[0].collapsed}
        <div class="section-body">
          {#if files.isLoading}
            <div class="empty-state">
              <div class="spinner" />
              <span class="text-muted text-sm">Loading...</span>
            </div>
          {:else if files.error}
            <div class="empty-state">
              <span class="text-sm" style="color: var(--color-error)">{files.error}</span>
            </div>
          {:else if files.fileTree.length === 0}
            <div class="empty-state">
              <span class="text-muted text-sm">No files yet</span>
            </div>
          {:else}
            <FileTree
              entries={files.fileTree}
              selectedPath={files.selectedFilePath}
              onSelect={(path) => {
                const relativePath = toVaultRelativePath(path);
                files.selectFile(relativePath);
                editor.openFile(relativePath);
              }}
            />
          {/if}
        </div>
      {/if}
    </div>

    <!-- Tags Section -->
    <div class="section">
      <button class="section-header" onclick={() => toggleSection('tags')}>
        <svg
          width="12"
          height="12"
          viewBox="0 0 16 16"
          fill="none"
          class:collapsed={sections[1].collapsed}
        >
          <path d="M6 4l4 4-4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
        <span class="section-label">Tags</span>
      </button>
      {#if !sections[1].collapsed}
        <div class="section-body">
          {#if files.tags.length === 0}
            <span class="text-muted text-sm">No tags</span>
          {:else}
            {#each files.tags as tagName}
              <button class="tag-chip" onclick={() => files.searchFiles(tagName)}>
                <span>#{tagName}</span>
              </button>
            {/each}
          {/if}
        </div>
      {/if}
    </div>

    <!-- Recent Files Section -->
    <div class="section">
      <button class="section-header" onclick={() => toggleSection('recent')}>
        <svg
          width="12"
          height="12"
          viewBox="0 0 16 16"
          fill="none"
          class:collapsed={sections[2].collapsed}
        >
          <path d="M6 4l4 4-4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
        <span class="section-label">Recent</span>
      </button>
      {#if !sections[2].collapsed}
        <div class="section-body">
          {#if files.recentFiles.length === 0}
            <span class="text-muted text-sm">No recent files</span>
          {:else}
            {#each files.recentFiles as file}
              <button
                class="recent-item truncate"
                title={`${file.path} • ${new Date(file.modified).toLocaleString()}`}
                onclick={() => {
                  files.selectFile(file.path);
                  editor.openFile(file.path);
                }}
              >
                {file.name}
              </button>
            {/each}
          {/if}
        </div>
      {/if}
    </div>

  </div>
<ImportDialog
  show={showImportDialog}
  onClose={() => { showImportDialog = false; }}
  onImported={handleImported}
/>
</aside>

<style>
  .sidebar {
    width: var(--sidebar-width);
    min-width: 200px;
    max-width: 400px;
    height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--color-surface);
    border-right: 1px solid var(--color-border);
    user-select: none;
  }
  .sidebar-header {
    padding: 8px;
    border-bottom: 1px solid var(--color-border);
  }
  .search-wrapper {
    position: relative;
  }
  .search-icon {
    position: absolute;
    left: 10px;
    top: 50%;
    transform: translateY(-50%);
    color: var(--color-text-muted);
    pointer-events: none;
  }
  .search-input {
    width: 100%;
    padding: 6px 10px 6px 30px;
    border-radius: var(--radius-md);
    font-size: 13px;
  }
  .sidebar-actions {
    padding: 8px;
    border-bottom: 1px solid var(--color-border);
  }
  .sidebar-actions .btn-sm {
    width: 100%;
    justify-content: center;
    font-size: 12px;
    padding: 5px 10px;
  }
  .sidebar-content {
    flex: 1;
    overflow-y: auto;
  }
  .section-header {
    display: flex;
    align-items: center;
    gap: 4px;
    width: 100%;
    padding: 6px 8px;
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-muted);
    cursor: pointer;
    transition: background var(--transition-fast);
  }
  .section-header:hover {
    background: color-mix(in srgb, var(--color-accent) 8%, transparent);
  }
  .section-header svg {
    transition: transform var(--transition-fast);
  }
  .section-header svg.collapsed {
    transform: rotate(-90deg);
  }
  .section-label {
    flex: 1;
    text-align: left;
  }
  .section-body {
    padding: 2px 8px 8px;
  }
  .empty-state {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px;
  }
  .spinner {
    width: 14px;
    height: 14px;
    border: 2px solid var(--color-border);
    border-top-color: var(--color-accent);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  .tag-chip {
    display: inline-flex;
    align-items: center;
    padding: 2px 8px;
    margin: 2px;
    border-radius: 999px;
    font-size: 11px;
    background: color-mix(in srgb, var(--color-accent) 12%, transparent);
    color: var(--color-accent);
    cursor: pointer;
    transition: background var(--transition-fast);
  }
  .tag-chip:hover {
    background: color-mix(in srgb, var(--color-accent) 20%, transparent);
  }
  .recent-item {
    display: block;
    width: 100%;
    padding: 4px 8px;
    font-size: 13px;
    text-align: left;
    border-radius: var(--radius-sm);
    transition: background var(--transition-fast);
  }
  .recent-item:hover {
    background: color-mix(in srgb, var(--color-accent) 8%, transparent);
  }
</style>
