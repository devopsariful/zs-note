<script lang="ts">
  import { onMount } from 'svelte';
  import Sidebar from './Sidebar.svelte';
  import Editor from './Editor.svelte';
  import AIPanel from './AIPanel.svelte';
  import StatusBar from './StatusBar.svelte';
  import Settings from './Settings.svelte';
  import APIKeyManager from './APIKeyManager.svelte';
  import ContextInspector from './ContextInspector.svelte';
  import HistoryBrowser from './HistoryBrowser.svelte';
  import TabBar from './TabBar.svelte';
  import ImageGenerationDialog from './ImageGenerationDialog.svelte';
  import SkipLink from '../a11y/SkipLink.svelte';
  import Announcements from '../a11y/Announcements.svelte';
  import { announce } from '../a11y/announce';
  import { getI18n } from '../i18n';
  import { getEditorStore } from '../stores/editor.svelte';
  import { getFilesStore } from '../stores/files.svelte';
  import { getAIStore } from '../stores/ai.svelte';
  import { getConfigStore } from '../stores/config.svelte';

  const editor = getEditorStore();
  const files = getFilesStore();
  const ai = getAIStore();
  const config = getConfigStore();
  const i18n = getI18n();

  /* Sync document direction and lang with locale */
  $effect(() => {
    document.documentElement.dir = i18n.direction;
    document.documentElement.lang = i18n.locale;
  });

  let showSidebar = $state(true);
  let showAIPanel = $state(false);
  let showSettings = $state(false);
  let showAPIKeys = $state(false);
  let showContextInspector = $state(false);
  let showHistory = $state(false);
  let showImageGen = $state(false);
  let sidebarWidth = $state(280);

  onMount(() => {
    config.loadAll();
    files.loadFileTree();
    files.loadRecentFiles();

    document.addEventListener('keydown', handleGlobalKeydown);

    return () => {
      document.removeEventListener('keydown', handleGlobalKeydown);
    };
  });

  function handleGlobalKeydown(e: KeyboardEvent) {
    const mod = e.metaKey || e.ctrlKey;

    if (mod && e.key === '\\') {
      e.preventDefault();
      showSidebar = !showSidebar;
    }

    if (mod && e.shiftKey && (e.key === 'A' || e.key === 'a')) {
      e.preventDefault();
      showAIPanel = !showAIPanel;
    }

    if (mod && e.shiftKey && (e.key === 'C' || e.key === 'c')) {
      e.preventDefault();
      showContextInspector = !showContextInspector;
    }

    if (mod && e.key === ',') {
      e.preventDefault();
      showSettings = !showSettings;
    }

    if (mod && e.shiftKey && (e.key === 'H' || e.key === 'h')) {
      e.preventDefault();
      showHistory = !showHistory;
    }

    if (mod && e.shiftKey && (e.key === 'I' || e.key === 'i')) {
      e.preventDefault();
      showImageGen = !showImageGen;
    }
  }

  function toggleSettings() {
    showSettings = !showSettings;
  }

  function handleSidebarResize(e: MouseEvent) {
    e.preventDefault();
    const startX = e.clientX;
    const startWidth = sidebarWidth;

    function onMove(ev: MouseEvent) {
      const delta = ev.clientX - startX;
      sidebarWidth = Math.max(200, Math.min(500, startWidth + delta));
    }

    function onUp() {
      document.removeEventListener('mousemove', onMove);
      document.removeEventListener('mouseup', onUp);
    }

    document.addEventListener('mousemove', onMove);
    document.addEventListener('mouseup', onUp);
  }

  function handleFileEvents(e: CustomEvent) {
    const { detail } = e;

    if (e.type === 'file:create') {
      files.createFile(detail.path);
      announce(`Created ${detail.path}`, 'polite');
    } else if (e.type === 'file:create-folder') {
      files.createFolder(detail.path);
      announce(`Created folder ${detail.path}`, 'polite');
    } else if (e.type === 'file:delete') {
      files.deleteFile(detail.path);
      announce(`Deleted ${detail.path}`, 'assertive');
    } else if (e.type === 'file:rename') {
      files.renameFile(detail.oldPath, detail.newPath);
      announce(`Renamed to ${detail.newPath}`, 'polite');
    } else if (e.type === 'file:move') {
      files.moveFile(detail.oldPath, detail.newPath);
      announce(`Moved to ${detail.newPath}`, 'polite');
    } else if (e.type === 'file:duplicate') {
      files.duplicateFile(detail.path);
      announce(`Duplicated ${detail.path}`, 'polite');
    }
  }

  onMount(() => {
    const handlers = ['file:create', 'file:create-folder', 'file:delete', 'file:rename', 'file:move', 'file:duplicate'];
    for (const evt of handlers) {
      window.addEventListener(evt, handleFileEvents as EventListener);
    }

    return () => {
      for (const evt of handlers) {
        window.removeEventListener(evt, handleFileEvents as EventListener);
      }
    };
  });
</script>

<div class="app-container">
  <!-- Accessibility: Skip link & announcements -->
  <SkipLink />
  <Announcements />

  <!-- Title Bar -->
  <header class="titlebar">
    <div class="titlebar-left">
      <button
        class="btn btn-ghost btn-icon"
        onclick={() => { showSidebar = !showSidebar; }}
        aria-label="Toggle sidebar"
        title="Toggle Sidebar (⌘\)"
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
          <rect x="1" y="1" width="14" height="14" rx="1" stroke="currentColor" stroke-width="1.5" />
          <line x1="5" y1="1" x2="5" y2="15" stroke="currentColor" stroke-width="1.5" />
        </svg>
      </button>
      <span class="app-name">{i18n.t('app.name')}</span>
      {#if editor.activeFilePath}
        <span class="file-breadcrumb text-muted text-sm truncate">
          {editor.activeFilePath}
        </span>
      {/if}
    </div>
    <div class="titlebar-center">
      <div class="mode-switcher">
        {#each ['wysiwyg', 'source', 'split'] as mode}
          <button
            class="mode-btn"
            class:active={editor.editorMode === mode}
            onclick={() => editor.setMode(mode as 'wysiwyg' | 'source' | 'split')}
          >
            {mode.charAt(0).toUpperCase() + mode.slice(1)}
          </button>
        {/each}
      </div>
    </div>
    <div class="titlebar-right">
      <button
        class="btn btn-ghost btn-icon"
        onclick={() => { showHistory = !showHistory; }}
        title="History (⌘⇧H)"
        aria-label="History"
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
          <circle cx="8" cy="8" r="6" stroke="currentColor" stroke-width="1.5" />
          <polyline points="8,4 8,8 11,10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
      </button>
      <button
        class="btn btn-ghost btn-icon"
        onclick={() => { showAIPanel = !showAIPanel; }}
        title="Toggle AI Panel (⌘⇧A)"
        aria-label="AI Assistant"
        class:active={showAIPanel}
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M12 3v4M12 17v4M5 12H3M21 12h-2M6.3 6.3l-1.4-1.4M19 19l-1.4-1.4M17.7 6.3l1.4-1.4M5 19l1.4-1.4" stroke-linecap="round" />
          <circle cx="12" cy="12" r="3" />
        </svg>
      </button>
      <button
        class="btn btn-ghost btn-icon"
        onclick={() => { showImageGen = !showImageGen; }}
        title="Generate Image (⌘⇧I)"
        aria-label="Generate Image"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <rect x="3" y="3" width="18" height="18" rx="2" />
          <circle cx="8.5" cy="8.5" r="1.5" />
          <polyline points="21 15 16 10 5 21" />
        </svg>
      </button>
      <button
        class="btn btn-ghost btn-icon"
        onclick={() => { showAPIKeys = !showAPIKeys; }}
        title="API Keys"
        aria-label="API Keys"
      >
        <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
          <path d="M21 2l-2 2m-7.61 7.61a5.5 5.5 0 11-7.778 7.778 5.5 5.5 0 017.777-7.777zm0 0L15.5 7.5m0 0l3 3L22 7l-3-3m-3.5 3.5L19 4" />
        </svg>
      </button>
      <button
        class="btn btn-ghost btn-icon"
        onclick={toggleSettings}
        title="Settings (⌘,)"
        aria-label="Settings"
      >
        <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
          <circle cx="8" cy="8" r="2.5" stroke="currentColor" stroke-width="1.5" />
          <path d="M8 1v2M8 13v2M1 8h2M13 8h2M3.5 3.5l1.4 1.4M11.1 11.1l1.4 1.4M3.5 12.5l1.4-1.4M11.1 4.9l1.4-1.4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
      </button>
    </div>
  </header>

  <!-- Main Layout -->
  <div class="main-layout">
    {#if showSidebar}
      <div class="sidebar-wrapper" style="width: {sidebarWidth}px;">
        <Sidebar />
        <div class="sidebar-resizer" onmousedown={handleSidebarResize} />
      </div>
    {/if}

    <div class="editor-area" id="main-content" tabindex="-1">
      {#if editor.openTabs.length > 0}
        <TabBar
          onSelect={(path) => editor.selectTab(path)}
          onClose={(path) => editor.closeTab(path)}
          onReorder={(from, to) => editor.reorderTabs(from, to)}
        />
      {/if}
      <Editor
        content={editor.content}
        filePath={editor.activeFilePath}
        mode={editor.editorMode}
        onContentChange={(c) => editor.updateContent(c)}
        onSave={() => editor.saveFile()}
      />
    </div>

    {#if showAIPanel}
      <AIPanel
        visible={showAIPanel}
        onToggle={() => { showAIPanel = !showAIPanel; }}
      />
    {/if}
  </div>

  <!-- Status Bar -->
  <StatusBar />

  <!-- Modals -->
  {#if showSettings}
    <Settings
      show={showSettings}
      onClose={() => { showSettings = false; }}
    />
  {/if}

  {#if showContextInspector}
    <ContextInspector
      show={showContextInspector}
      onClose={() => { showContextInspector = false; }}
    />
  {/if}

  {#if showHistory}
    <HistoryBrowser
      show={showHistory}
      filePath={editor.activeFilePath}
      onClose={() => { showHistory = false; }}
    />
  {/if}

  {#if showAPIKeys}
    <APIKeyManager
      show={showAPIKeys}
      onClose={() => { showAPIKeys = false; }}
    />
  {/if}

  {#if showImageGen}
    <ImageGenerationDialog
      show={showImageGen}
      onClose={() => { showImageGen = false; }}
      onInsert={(dataUrl: string) => {
        const markdown = `![generated-image](${dataUrl})`;
        const event = new CustomEvent('ai:insert', { detail: { content: markdown } });
        window.dispatchEvent(event);
      }}
    />
  {/if}
</div>

<style>
  .app-container {
    height: 100%;
    display: flex;
    flex-direction: column;
    overflow: hidden;
  }
  .titlebar {
    height: var(--titlebar-height);
    min-height: var(--titlebar-height);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    user-select: none;
    -webkit-app-region: drag;
  }
  .titlebar-left,
  .titlebar-right {
    display: flex;
    align-items: center;
    gap: 8px;
    -webkit-app-region: no-drag;
  }
  .titlebar-center {
    -webkit-app-region: no-drag;
  }
  .app-name {
    font-size: 14px;
    font-weight: 700;
    letter-spacing: -0.01em;
  }
  .file-breadcrumb {
    max-width: 300px;
  }
  .mode-switcher {
    display: flex;
    background: var(--color-bg);
    border-radius: var(--radius-md);
    border: 1px solid var(--color-border);
    overflow: hidden;
  }
  .mode-btn {
    padding: 4px 12px;
    font-size: 11px;
    font-weight: 500;
    transition: all var(--transition-fast);
  }
  .mode-btn:hover {
    background: color-mix(in srgb, var(--color-accent) 8%, transparent);
  }
  .mode-btn.active {
    background: var(--color-accent);
    color: white;
  }
  .titlebar-right .btn-icon.active {
    color: var(--color-accent);
  }
  .main-layout {
    flex: 1;
    display: flex;
    overflow: hidden;
    position: relative;
  }
  .sidebar-wrapper {
    position: relative;
    flex-shrink: 0;
  }
  .sidebar-resizer {
    position: absolute;
    right: -3px;
    top: 0;
    bottom: 0;
    width: 6px;
    cursor: col-resize;
    z-index: 5;
  }
  .sidebar-resizer:hover {
    background: color-mix(in srgb, var(--color-accent) 20%, transparent);
  }
  .editor-area {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    min-width: 0;
  }
</style>
