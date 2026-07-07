<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { getEditorStore } from '../stores/editor.svelte';
  import type { CommitEntry, DiffResult } from '../types';
  import Modal from './Modal.svelte';

  const editor = getEditorStore();

  let {
    show = false,
    filePath = '',
    onClose = () => {},
  }: {
    show?: boolean;
    filePath?: string;
    onClose?: () => void;
  } = $props();

  let commits = $state<CommitEntry[]>([]);
  let selectedCommit = $state<string | null>(null);
  let diffResult = $state<DiffResult | null>(null);
  let isLoading = $state(false);
  let showAllVault = $state(false);
  let error = $state<string | null>(null);

  $effect(() => {
    if (show) {
      loadHistory();
    }
  });

  function loadHistory() {
    isLoading = true;
    error = null;
    diffResult = null;

    invoke<CommitEntry[]>('git_log', showAllVault ? {} : { filePath: filePath || editor.activeFilePath || '' })
      .then((c) => { commits = c; })
      .catch((err) => { error = String(err); })
      .finally(() => { isLoading = false; });
  }

  function viewDiff(commitHash: string) {
    if (selectedCommit === commitHash) {
      selectedCommit = null;
      diffResult = null;
      return;
    }

    selectedCommit = commitHash;
    isLoading = true;

    invoke<DiffResult>('git_diff', {
      path: filePath || editor.activeFilePath || '',
      rev1: `${commitHash}^`,
      rev2: commitHash,
    })
      .then((d) => { diffResult = d; })
      .catch((err) => { error = String(err); })
      .finally(() => { isLoading = false; });
  }

  function handleRestore(commitHash: string) {
    if (!confirm('Restore file to this version? This will create a new commit.')) return;

    invoke('git_restore', {
      filePath: filePath || editor.activeFilePath,
      commit: commitHash,
    })
      .then(() => {
        editor.openFile(filePath || editor.activeFilePath);
      })
      .catch((err) => { error = String(err); });
  }

  function formatDate(ts: string): string {
    try {
      const d = new Date(ts);
      return d.toLocaleDateString() + ' ' + d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    } catch {
      return ts;
    }
  }

  function getDiffLineClass(type: string): string {
    switch (type) {
      case 'added': return 'diff-added';
      case 'removed': return 'diff-removed';
      default: return 'diff-context';
    }
  }
</script>

<Modal title="History Browser" bind:show size="lg" {onClose}>
  <div class="history-browser">
    <div class="toolbar-row">
      <label class="vault-toggle">
        <input type="checkbox" bind:checked={showAllVault} onchange={loadHistory} />
        <span class="text-sm">Show all-vault history</span>
      </label>
      <button class="btn btn-ghost btn-sm" onclick={loadHistory}>
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
          <path d="M2 8a6 6 0 0111.3-3M14 8a6 6 0 01-11.3 3" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
          <path d="M14 2v4h-4M2 14v-4h4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
        Refresh
      </button>
    </div>

    {#if error}
      <div class="error-banner text-sm" style="color: var(--color-error); padding: 8px; background: var(--color-error-bg); border-radius: var(--radius-md);">{error}</div>
    {/if}

    {#if isLoading && commits.length === 0}
      <div class="empty-state">
        <div class="spinner" />
        <span class="text-muted text-sm">Loading history...</span>
      </div>
    {:else if commits.length === 0}
      <div class="empty-state">
        <span class="text-muted text-sm">No commit history found</span>
      </div>
    {:else}
      <div class="history-layout">
        <div class="commit-list">
          {#each commits as commit}
            <button
              class="commit-item"
              class:selected={selectedCommit === commit.hash}
              onclick={() => viewDiff(commit.hash)}
            >
              <div class="commit-message truncate">{commit.message.split('\n')[0]}</div>
              <div class="commit-meta text-xs text-muted">
                <span>{commit.author}</span>
                <span>{formatDate(commit.timestamp)}</span>
              </div>
              <div class="commit-hash text-xs font-mono text-muted">{commit.hash.substring(0, 7)}</div>
            </button>
          {/each}
        </div>

        <div class="diff-panel">
          {#if isLoading}
            <div class="empty-state">
              <div class="spinner" />
              <span class="text-muted text-sm">Loading diff...</span>
            </div>
          {:else if diffResult}
            <div class="diff-header">
              <span class="text-sm font-bold">Diff</span>
              <button
                class="btn btn-primary btn-sm"
                onclick={() => handleRestore(selectedCommit!)}
              >
                Restore to this version
              </button>
            </div>
            <div class="diff-content">
              {#each diffResult.hunks as hunk}
                <div class="hunk-header text-xs font-mono">
                  @@ -{hunk.oldStart},{hunk.oldLines} +{hunk.newStart},{hunk.newLines} @@
                </div>
                {#each hunk.lines as line}
                  <div class="diff-line {getDiffLineClass(line.type)}">
                    <span class="line-prefix">
                      {line.type === 'added' ? '+' : line.type === 'removed' ? '-' : ' '}
                    </span>
                    <span class="line-content">{line.content}</span>
                  </div>
                {/each}
              {/each}
            </div>
          {:else}
            <div class="empty-state">
              <span class="text-muted text-sm">Select a commit to view changes</span>
            </div>
          {/if}
        </div>
      </div>
    {/if}
  </div>
</Modal>

<style>
  .history-browser {
    display: flex;
    flex-direction: column;
    gap: 12px;
    height: 500px;
  }
  .toolbar-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .vault-toggle {
    display: flex;
    align-items: center;
    gap: 6px;
    cursor: pointer;
  }
  .vault-toggle input {
    width: 14px;
    height: 14px;
  }
  .empty-state {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    padding: 48px;
    color: var(--color-text-muted);
  }
  .spinner {
    width: 20px;
    height: 20px;
    border: 2px solid var(--color-border);
    border-top-color: var(--color-accent);
    border-radius: 50%;
    animation: spin 0.6s linear infinite;
  }
  @keyframes spin {
    to { transform: rotate(360deg); }
  }
  .history-layout {
    display: flex;
    gap: 12px;
    flex: 1;
    min-height: 0;
  }
  .commit-list {
    width: 220px;
    min-width: 180px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    overflow-y: auto;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    padding: 4px;
  }
  .commit-item {
    display: flex;
    flex-direction: column;
    gap: 2px;
    padding: 6px 8px;
    text-align: left;
    border-radius: var(--radius-sm);
    transition: background var(--transition-fast);
  }
  .commit-item:hover {
    background: var(--color-surface);
  }
  .commit-item.selected {
    background: color-mix(in srgb, var(--color-accent) 12%, transparent);
  }
  .commit-message {
    font-size: 13px;
    font-weight: 500;
  }
  .commit-meta {
    display: flex;
    justify-content: space-between;
  }
  .diff-panel {
    flex: 1;
    display: flex;
    flex-direction: column;
    min-width: 0;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    overflow: hidden;
  }
  .diff-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 8px 12px;
    border-bottom: 1px solid var(--color-border);
    background: var(--color-surface);
  }
  .diff-content {
    flex: 1;
    overflow-y: auto;
    padding: 8px 0;
    font-family: var(--font-code);
    font-size: 12px;
    line-height: 1.5;
  }
  .hunk-header {
    padding: 4px 12px;
    background: var(--color-surface);
    color: var(--color-text-muted);
    font-weight: 500;
  }
  .diff-line {
    display: flex;
    padding: 0 12px;
    white-space: pre-wrap;
    word-break: break-all;
  }
  .line-prefix {
    width: 16px;
    flex-shrink: 0;
    color: var(--color-text-muted);
  }
  .diff-added {
    background: var(--color-success-bg);
  }
  .diff-removed {
    background: var(--color-error-bg);
  }
  .diff-context {
    background: transparent;
  }
</style>
