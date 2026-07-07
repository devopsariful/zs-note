<script lang="ts">
  import { getEditorStore } from '../stores/editor.svelte';

  const editor = getEditorStore();

  let {
    onSelect = (_path: string) => {},
    onClose = (_path: string) => {},
    onReorder = (_from: number, _to: number) => {},
  }: {
    onSelect?: (path: string) => void;
    onClose?: (path: string) => void;
    onReorder?: (from: number, to: number) => void;
  } = $props();

  let tabsContainer: HTMLDivElement;
  let dragIndex: number | null = $state(null);
  let dragOverIndex: number | null = $state(null);

  function handleDragStart(e: DragEvent, index: number) {
    dragIndex = index;
    if (e.dataTransfer) {
      e.dataTransfer.effectAllowed = 'move';
      e.dataTransfer.setData('text/plain', String(index));
    }
  }

  function handleDragOver(e: DragEvent, index: number) {
    e.preventDefault();
    if (e.dataTransfer) {
      e.dataTransfer.dropEffect = 'move';
    }
    dragOverIndex = index;
  }

  function handleDragLeave() {
    dragOverIndex = null;
  }

  function handleDrop(e: DragEvent, toIndex: number) {
    e.preventDefault();
    if (dragIndex !== null && dragIndex !== toIndex) {
      onReorder(dragIndex, toIndex);
    }
    dragIndex = null;
    dragOverIndex = null;
  }

  function handleDragEnd() {
    dragIndex = null;
    dragOverIndex = null;
  }

  function handleClose(e: MouseEvent, path: string) {
    e.stopPropagation();
    onClose(path);
  }

  function handleNewFile() {
    // Dispatch custom event for creating a new file
    const event = new CustomEvent('tab:new', {});
    window.dispatchEvent(event);
  }

  function handleSelect(path: string) {
    onSelect(path);
  }
</script>

<div class="tab-bar-container">
  <div
    class="tab-bar"
    bind:this={tabsContainer}
    role="tablist"
    aria-label="Open files"
  >
    {#each editor.openTabs as tab, idx (tab.path)}
      <button
        class="tab"
        class:active={tab.path === editor.activeFilePath}
        class:drag-over={dragOverIndex === idx}
        draggable="true"
        ondragstart={(e) => handleDragStart(e, idx)}
        ondragover={(e) => handleDragOver(e, idx)}
        ondragleave={handleDragLeave}
        ondrop={(e) => handleDrop(e, idx)}
        ondragend={handleDragEnd}
        onclick={() => handleSelect(tab.path)}
        role="tab"
        aria-selected={tab.path === editor.activeFilePath}
        title={tab.path}
      >
        <span class="tab-name truncate">{tab.name}</span>
        {#if editor.getTabModifiedState(tab.path)}
          <span class="tab-modified-dot" aria-label="Unsaved changes" />
        {/if}
        <span
          class="tab-close"
          onclick={(e) => handleClose(e, tab.path)}
          role="button"
          tabindex="-1"
          aria-label="Close {tab.name}"
        >
          <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
            <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
          </svg>
        </span>
      </button>
    {/each}
    <button
      class="tab new-tab-btn"
      onclick={handleNewFile}
      title="New file"
      aria-label="New file"
    >
      <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
        <path d="M8 3v10M3 8h10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
      </svg>
    </button>
  </div>
  <div class="tab-bar-fade tab-bar-fade-left" />
  <div class="tab-bar-fade tab-bar-fade-right" />
</div>

<style>
  .tab-bar-container {
    position: relative;
    display: flex;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    min-height: 36px;
    overflow: hidden;
  }
  .tab-bar {
    display: flex;
    align-items: center;
    overflow-x: auto;
    overflow-y: hidden;
    scrollbar-width: none;
    padding: 0 4px;
    gap: 2px;
    flex: 1;
    min-width: 0;
  }
  .tab-bar::-webkit-scrollbar {
    display: none;
  }
  .tab {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 6px 10px;
    font-size: 12px;
    font-weight: 500;
    color: var(--color-text-muted);
    background: transparent;
    border: none;
    border-right: 1px solid var(--color-border);
    cursor: pointer;
    white-space: nowrap;
    flex-shrink: 0;
    position: relative;
    transition: background var(--transition-fast), color var(--transition-fast);
    user-select: none;
    min-width: 0;
  }
  .tab:hover {
    background: color-mix(in srgb, var(--color-accent) 8%, transparent);
    color: var(--color-text);
  }
  .tab.active {
    color: var(--color-text);
    background: var(--color-bg);
    border-bottom: 2px solid var(--color-accent);
  }
  .tab.drag-over {
    border-left: 2px solid var(--color-accent);
  }
  .tab-name {
    max-width: 140px;
  }
  .tab-modified-dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--color-accent);
    flex-shrink: 0;
  }
  .tab-close {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 18px;
    height: 18px;
    border-radius: var(--radius-sm);
    opacity: 0;
    transition: opacity var(--transition-fast), background var(--transition-fast);
    flex-shrink: 0;
  }
  .tab:hover .tab-close {
    opacity: 0.6;
  }
  .tab-close:hover {
    opacity: 1 !important;
    background: color-mix(in srgb, var(--color-text) 12%, transparent);
  }
  .new-tab-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    padding: 6px 8px;
    font-size: 12px;
    color: var(--color-text-muted);
    background: transparent;
    border: none;
    cursor: pointer;
    flex-shrink: 0;
    transition: color var(--transition-fast), background var(--transition-fast);
  }
  .new-tab-btn:hover {
    color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 8%, transparent);
  }
  .tab-bar-fade {
    position: absolute;
    top: 0;
    bottom: 0;
    width: 24px;
    pointer-events: none;
    z-index: 1;
  }
  .tab-bar-fade-left {
    left: 0;
    background: linear-gradient(to right, var(--color-surface), transparent);
  }
  .tab-bar-fade-right {
    right: 0;
    background: linear-gradient(to left, var(--color-surface), transparent);
  }
</style>
