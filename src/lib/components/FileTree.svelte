<script lang="ts">
  import type { FileEntry } from '../types';
  import { getI18n } from '../i18n';
  import { toVaultRelativePath } from '../utils/vaultPath';

  const i18n = getI18n();

  let {
    entries = [] as FileEntry[],
    selectedPath = null as string | null,
    depth = 0,
    onSelect = (_path: string) => {},
  }: {
    entries: FileEntry[];
    selectedPath?: string | null;
    depth?: number;
    onSelect?: (path: string) => void;
  } = $props();

  /* ── Large vault performance: pagination & virtual scroll ── */
  const PAGE_SIZE = 500;
  const MAX_VISIBLE_NODES = 5000;

  let collapsedDirs = $state<Set<string>>(new Set());
  let selectedPaths = $state<Set<string>>(new Set());
  let contextMenuTarget = $state<{ x: number; y: number; entry: FileEntry } | null>(null);
  let visibleOffset = $state(0);
  let isLoadingMore = $state(false);
  let containerEl: HTMLDivElement | undefined = $state(undefined);

  /* Cache expanded directory children so re-collapsing is instant */
  let expandedCache = $state(new Map<string, FileEntry[]>());

  function toggleDir(path: string, entry: FileEntry) {
    path = toVaultRelativePath(path);
    const next = new Set(collapsedDirs);
    if (next.has(path)) {
      next.delete(path);
    } else {
      next.add(path);
      /* Cache children for instant re-open */
      if (entry.children && entry.children.length > 0) {
        expandedCache.set(path, entry.children);
      }
    }
    collapsedDirs = next;
  }

  function isDirExpanded(path: string): boolean {
    return !collapsedDirs.has(toVaultRelativePath(path));
  }

  function handleSelect(path: string, e: MouseEvent) {
    path = toVaultRelativePath(path);
    if (e.metaKey || e.ctrlKey) {
      const next = new Set(selectedPaths);
      if (next.has(path)) {
        next.delete(path);
      } else {
        next.add(path);
      }
      selectedPaths = next;
    } else {
      selectedPaths = new Set([path]);
    }
    onSelect(path);
  }

  function handleContextMenu(e: MouseEvent, entry: FileEntry) {
    e.preventDefault();
    contextMenuTarget = { x: e.clientX, y: e.clientY, entry };
  }

  function closeContextMenu() {
    contextMenuTarget = null;
  }

  /* ── Scroll-based lazy loading ── */
  let treeFlat = $derived(flattenTree(entries, 0));

  function flattenTree(items: FileEntry[], currentDepth: number): Array<{ entry: FileEntry; depth: number; isVisible: boolean }> {
    const result: Array<{ entry: FileEntry; depth: number; isVisible: boolean }> = [];
    let nodeCount = 0;

    function walk(items: FileEntry[], d: number, parentExpanded: boolean) {
      for (const item of items) {
        nodeCount++;
        if (nodeCount > MAX_VISIBLE_NODES) {
          /* Safety valve – stop processing past 5000 nodes */
          return;
        }
        result.push({ entry: item, depth: d, isVisible: parentExpanded });

        if (item.is_dir && isDirExpanded(item.path) && item.children) {
          walk(item.children, d + 1, true);
        }
      }
    }

    walk(items, currentDepth, true);
    return result;
  }

  let paginatedNodes = $derived(
    treeFlat.slice(0, visibleOffset + PAGE_SIZE)
  );

  let hasMore = $derived(treeFlat.length > paginatedNodes.length);

  function handleScroll() {
    if (!containerEl) return;
    const { scrollTop, scrollHeight, clientHeight } = containerEl;
    if (scrollTop + clientHeight >= scrollHeight - 200 && hasMore && !isLoadingMore) {
      isLoadingMore = true;
      requestAnimationFrame(() => {
        visibleOffset = paginatedNodes.length;
        isLoadingMore = false;
      });
    }
  }

  /* ── SVG icons (unchanged) ── */
  const SVG_FOLDER_CLOSED = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>';
  const SVG_FOLDER_OPEN = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/><path d="M6 9l-2 8h16l2-8z"/></svg>';
  const SVG_FILE = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/></svg>';
  const SVG_FILE_TEXT = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><line x1="8" y1="13" x2="16" y2="13"/><line x1="8" y1="17" x2="16" y2="17"/></svg>';
  const SVG_FILE_CODE = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><polyline points="14 2 14 8 20 8"/><path d="m10 13-2 2 2 2"/><path d="m14 17 2-2-2-2"/></svg>';
  const SVG_FILE_IMAGE = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>';
  const SVG_FILE_GLOBE = '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="2" y1="12" x2="22" y2="12"/><path d="M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>';

  const fileIconMap: Record<string, string> = {
    md: SVG_FILE_TEXT, txt: SVG_FILE_TEXT,
    js: SVG_FILE_CODE, ts: SVG_FILE_CODE, jsx: SVG_FILE_CODE, tsx: SVG_FILE_CODE,
    json: SVG_FILE_CODE, yaml: SVG_FILE_CODE, yml: SVG_FILE_CODE,
    css: SVG_FILE_CODE, scss: SVG_FILE_CODE,
    html: SVG_FILE_GLOBE, htm: SVG_FILE_GLOBE,
    svg: SVG_FILE_IMAGE, png: SVG_FILE_IMAGE, jpg: SVG_FILE_IMAGE,
    jpeg: SVG_FILE_IMAGE, gif: SVG_FILE_IMAGE, webp: SVG_FILE_IMAGE, ico: SVG_FILE_IMAGE,
    wasm: SVG_FILE_CODE, toml: SVG_FILE_CODE, rs: SVG_FILE_CODE,
    py: SVG_FILE_CODE, go: SVG_FILE_CODE, svelte: SVG_FILE_CODE,
  };

  function getIcon(entry: FileEntry): string {
    if (entry.is_dir) {
      return collapsedDirs.has(entry.path) ? SVG_FOLDER_CLOSED : SVG_FOLDER_OPEN;
    }
    const ext = entry.extension || entry.name.split('.').pop()?.toLowerCase() || '';
    return fileIconMap[ext] || SVG_FILE;
  }

  /* ── Context menu actions ── */
  function handleRename(entry: FileEntry) {
    contextMenuTarget = null;
    const newName = prompt('Rename to:', entry.name);
    if (newName && newName !== entry.name) {
      const parts = entry.path.split('/');
      parts[parts.length - 1] = newName;
      const newPath = toVaultRelativePath(parts.join('/'));
      window.dispatchEvent(new CustomEvent('file:rename', {
        detail: { oldPath: toVaultRelativePath(entry.path), newPath },
      }));
    }
  }

  function handleDeleteEntry(entry: FileEntry) {
    contextMenuTarget = null;
    if (confirm(`Delete "${entry.name}"?`)) {
      window.dispatchEvent(new CustomEvent('file:delete', {
        detail: { path: toVaultRelativePath(entry.path) },
      }));
    }
  }

  function handleDuplicate(entry: FileEntry) {
    contextMenuTarget = null;
    window.dispatchEvent(new CustomEvent('file:duplicate', {
      detail: { path: toVaultRelativePath(entry.path) },
    }));
  }

  function handleCopyPath(entry: FileEntry) {
    contextMenuTarget = null;
    navigator.clipboard.writeText(toVaultRelativePath(entry.path));
  }

  function handleDragStart(e: DragEvent, entry: FileEntry) {
    e.dataTransfer?.setData('text/plain', toVaultRelativePath(entry.path));
    e.dataTransfer!.effectAllowed = 'move';
  }

  function handleDrop(e: DragEvent, targetDir: FileEntry) {
    e.preventDefault();
    if (!targetDir.is_dir) return;
    const sourcePath = e.dataTransfer?.getData('text/plain');
    if (sourcePath) {
      const name = sourcePath.split('/').pop();
      window.dispatchEvent(new CustomEvent('file:move', {
        detail: {
          oldPath: toVaultRelativePath(sourcePath),
          newPath: toVaultRelativePath(`${targetDir.path}/${name}`),
        },
      }));
    }
  }

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    if (e.dataTransfer) e.dataTransfer.dropEffect = 'move';
  }
</script>

<div
  class="file-tree-container"
  bind:this={containerEl}
  onscroll={handleScroll}
  role="tree"
  aria-label={i18n.t('sidebar.files')}
>
  {#each paginatedNodes as { entry, depth: nodeDepth } (entry.path)}
    {#if entry.is_dir}
      <div class="tree-item" role="treeitem" aria-expanded={isDirExpanded(entry.path)}>
        <button
          class="tree-row dir-row"
          class:selected={selectedPath === entry.path || selectedPaths.has(entry.path)}
          style="--depth: {nodeDepth}"
          onclick={(e) => { toggleDir(entry.path, entry); handleSelect(entry.path, e); }}
          oncontextmenu={(e) => handleContextMenu(e, entry)}
          ondragover={handleDragOver}
          ondrop={(e) => handleDrop(e, entry)}
          draggable="true"
          ondragstart={(e) => handleDragStart(e, entry)}
        >
          <span class="chevron" class:collapsed={collapsedDirs.has(entry.path)}>
            <svg width="10" height="10" viewBox="0 0 16 16" fill="none" aria-hidden="true">
              <path d="M6 4l4 4-4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
          </span>
          <span class="file-icon" aria-hidden="true">{@html getIcon(entry)}</span>
          <span class="file-name truncate">{entry.name}</span>
        </button>
      </div>
    {:else}
      <div class="tree-item" role="treeitem">
        <button
          class="tree-row file-row"
          class:selected={selectedPath === entry.path || selectedPaths.has(entry.path)}
          style="--depth: {nodeDepth}"
          onclick={(e) => handleSelect(entry.path, e)}
          oncontextmenu={(e) => handleContextMenu(e, entry)}
          draggable="true"
          ondragstart={(e) => handleDragStart(e, entry)}
        >
          <span class="file-icon" aria-hidden="true">{@html getIcon(entry)}</span>
          <span class="file-name truncate">{entry.name}</span>
        </button>
      </div>
    {/if}
  {/each}

  {#if hasMore}
    <div class="load-more-indicator">
      <span class="text-muted text-sm">{i18n.t('common.loading')}</span>
    </div>
  {/if}
</div>

<!-- Context Menu -->
{#if contextMenuTarget}
  <!-- svelte-ignore a11y_click_events_have_key_events -->
  <div class="context-backdrop" onclick={closeContextMenu} oncontextmenu={(e) => e.preventDefault()}>
    <div
      class="context-menu"
      style="left: {contextMenuTarget.x}px; top: {contextMenuTarget.y}px;"
      role="menu"
    >
      {#if contextMenuTarget.entry.is_dir}
        <button class="context-item" onclick={() => {
          const name = prompt('New file name:');
          if (name) {
            window.dispatchEvent(new CustomEvent('file:create', {
              detail: { path: toVaultRelativePath(`${contextMenuTarget!.entry.path}/${name}`) },
            }));
          }
          contextMenuTarget = null;
        }}>{i18n.t('file.new')}</button>
        <button class="context-item" onclick={() => {
          const name = prompt('New folder name:');
          if (name) {
            window.dispatchEvent(new CustomEvent('file:create-folder', {
              detail: { path: toVaultRelativePath(`${contextMenuTarget!.entry.path}/${name}`) },
            }));
          }
          contextMenuTarget = null;
        }}>New Folder</button>
        <div class="context-separator" />
      {/if}
      <button class="context-item" onclick={() => handleRename(contextMenuTarget!.entry)}>{i18n.t('file.rename')}</button>
      <button class="context-item" onclick={() => handleDuplicate(contextMenuTarget!.entry)}>{i18n.t('file.duplicate')}</button>
      <button class="context-item" onclick={() => handleCopyPath(contextMenuTarget!.entry)}>Copy Path</button>
      <div class="context-separator" />
      <button class="context-item danger" onclick={() => handleDeleteEntry(contextMenuTarget!.entry)}>{i18n.t('common.delete')}</button>
    </div>
  </div>
{/if}

<style>
  .file-tree-container {
    overflow-y: auto;
    overflow-x: hidden;
    max-height: 100%;
  }
  .tree-item {
    user-select: none;
  }
  .tree-row {
    display: flex;
    align-items: center;
    gap: 3px;
    width: 100%;
    padding: 3px 6px;
    padding-left: calc(6px + var(--depth) * 16px);
    font-size: 13px;
    text-align: left;
    border-radius: var(--radius-sm);
    cursor: pointer;
    transition: background var(--transition-fast);
  }
  .tree-row:hover {
    background: color-mix(in srgb, var(--color-accent) 8%, transparent);
  }
  .tree-row.selected {
    background: color-mix(in srgb, var(--color-accent) 16%, transparent);
    color: var(--color-accent);
  }
  .chevron {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    flex-shrink: 0;
    transition: transform var(--transition-fast);
  }
  .chevron.collapsed {
    transform: rotate(-90deg);
  }
  .file-icon {
    flex-shrink: 0;
    font-size: 14px;
    line-height: 1;
  }
  .file-name {
    flex: 1;
    min-width: 0;
  }
  .load-more-indicator {
    display: flex;
    justify-content: center;
    padding: 8px;
  }
  .context-backdrop {
    position: fixed;
    inset: 0;
    z-index: var(--z-modal, 50);
  }
  .context-menu {
    position: fixed;
    min-width: 180px;
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    box-shadow: var(--shadow-lg);
    padding: 4px;
    z-index: var(--z-tooltip, 60);
  }
  .context-item {
    display: block;
    width: 100%;
    padding: 6px 12px;
    font-size: 13px;
    text-align: left;
    border-radius: var(--radius-sm);
    transition: background var(--transition-fast);
  }
  .context-item:hover {
    background: var(--color-surface);
  }
  .context-item.danger {
    color: var(--color-error);
  }
  .context-item.danger:hover {
    background: var(--color-error-bg);
  }
  .context-separator {
    height: 1px;
    background: var(--color-border);
    margin: 4px 8px;
  }
</style>
