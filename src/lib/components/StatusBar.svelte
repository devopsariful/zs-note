<script lang="ts">
  import { getEditorStore } from '../stores/editor.svelte';
  import { getFilesStore } from '../stores/files.svelte';

  const editor = getEditorStore();
  const files = getFilesStore();

  let {
    gitBranch = 'main',
    gitAhead = 0,
    gitBehind = 0,
    hasGitConflicts = false,
    wordCount = 0,
    charCount = 0,
  }: {
    gitBranch?: string;
    gitAhead?: number;
    gitBehind?: number;
    hasGitConflicts?: boolean;
    wordCount?: number;
    charCount?: number;
  } = $props();

  let countInfo = $derived.by(() => {
    const text = editor.content;
    const words = text ? text.trim().split(/\s+/).filter(Boolean).length : 0;
    const chars = text.length;
    return { words, chars };
  });
</script>

<footer class="statusbar" role="status" aria-label="Status bar">
  <div class="status-left">
    <!-- Git Status -->
    <div class="status-item git-status" class:has-conflicts={hasGitConflicts}>
      <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
        <path d="M8 1L1 8l7 7 7-7-7-7z" stroke="currentColor" stroke-width="1.5" fill="none" />
      </svg>
      <span>{gitBranch}</span>
      {#if gitAhead > 0 || gitBehind > 0}
        <span class="git-counts">
          {#if gitAhead > 0}<span class="ahead">↑{gitAhead}</span>{/if}
          {#if gitBehind > 0}<span class="behind">↓{gitBehind}</span>{/if}
        </span>
      {/if}
      {#if hasGitConflicts}
        <span class="conflict-badge">!</span>
      {/if}
    </div>
  </div>

  <div class="status-right">
    <!-- File Save Status -->
    <div class="status-item save-status">
      <span class="save-indicator" class:dirty={editor.isDirty}>
        {editor.isDirty ? 'Unsaved' : 'Saved'}
      </span>
    </div>

    <!-- Mode -->
    <div class="status-item mode-indicator">
      {editor.editorMode}
    </div>

    <!-- Word/Char Count -->
    <div class="status-item word-count">
      {countInfo.words} words
    </div>

    <!-- Cursor Position -->
    <div class="status-item cursor-pos">
      Ln {editor.cursorPosition.line}, Col {editor.cursorPosition.col}
    </div>
  </div>
</footer>

<style>
  .statusbar {
    height: var(--statusbar-height);
    min-height: var(--statusbar-height);
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 12px;
    background: var(--color-surface);
    border-top: 1px solid var(--color-border);
    font-size: 11px;
    color: var(--color-text-muted);
    user-select: none;
  }
  .status-left,
  .status-right {
    display: flex;
    align-items: center;
    gap: 12px;
  }
  .status-item {
    display: flex;
    align-items: center;
    gap: 4px;
    padding: 2px 6px;
    border-radius: var(--radius-sm);
    transition: background var(--transition-fast);
    cursor: default;
  }
  .status-item:hover {
    background: color-mix(in srgb, var(--color-accent) 8%, transparent);
  }
  .git-status.has-conflicts {
    color: var(--color-error);
  }
  .git-counts {
    display: flex;
    gap: 3px;
  }
  .ahead { color: var(--color-success); }
  .behind { color: var(--color-warning); }
  .conflict-badge {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--color-error);
    color: white;
    font-size: 10px;
    font-weight: 700;
  }
  .save-indicator::before {
    content: '';
    display: inline-block;
    width: 6px;
    height: 6px;
    border-radius: 50%;
    margin-right: 4px;
    background: var(--color-success);
  }
  .save-indicator.dirty::before {
    background: var(--color-warning);
  }
  .mode-indicator {
    text-transform: capitalize;
  }
</style>
