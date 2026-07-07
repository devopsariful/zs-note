<script lang="ts">
  import { open, ask } from '@tauri-apps/plugin-dialog';
  import { importFiles } from '../commands/import';

  let {
    show = false,
    vaultPath = '',
    onClose = () => {},
    onImported = (_paths: string[]) => {},
  }: {
    show?: boolean;
    vaultPath?: string;
    onClose?: () => void;
    onImported?: (paths: string[]) => void;
  } = $props();

  let selectedFiles = $state<string[]>([]);
  let importing = $state(false);
  let result = $state<string[]>([]);
  let errorMsg = $state('');

  async function handleSelectFiles() {
    try {
      const files = await open({
        multiple: true,
        filters: [
          { name: 'All Supported', extensions: ['md', 'txt', 'pdf', 'docx', 'pptx', 'xlsx', 'epub', 'csv', 'html', 'png', 'jpg', 'jpeg', 'gif', 'svg', 'webp'] },
          { name: 'Documents', extensions: ['md', 'txt', 'pdf', 'docx', 'pptx', 'xlsx', 'epub', 'csv', 'html'] },
          { name: 'Images', extensions: ['png', 'jpg', 'jpeg', 'gif', 'svg', 'webp'] },
          { name: 'All Files', extensions: ['*'] },
        ],
      });

      if (files) {
        selectedFiles = Array.isArray(files) ? files : [files];
      }
    } catch (e) {
      errorMsg = String(e);
    }
  }

  async function handleImport() {
    if (selectedFiles.length === 0) return;
    importing = true;
    errorMsg = '';

    try {
      const imported = await importFiles(selectedFiles, '');
      result = imported;
      onImported(imported);
    } catch (e) {
      errorMsg = String(e);
    }
    importing = false;
  }

  function reset() {
    selectedFiles = [];
    result = [];
    errorMsg = '';
  }

  function handleClose() {
    reset();
    onClose();
  }

  function getFileName(fullPath: string): string {
    const parts = fullPath.replace(/\\/g, '/').split('/');
    return parts[parts.length - 1] || fullPath;
  }
</script>

{#if show}
  <div class="modal-overlay" role="dialog" aria-label="Import files" onclick={handleClose}>
    <div class="modal-content" onclick={(e) => e.stopPropagation()}>
      <div class="modal-header">
        <h2 class="modal-title">Import Files</h2>
        <button class="close-btn" onclick={handleClose} aria-label="Close">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
          </svg>
        </button>
      </div>

      <div class="modal-body">
        {#if result.length > 0}
          <div class="import-success">
            <svg width="24" height="24" viewBox="0 0 24 24" fill="none" stroke="var(--color-success, #38a169)" stroke-width="2">
              <path d="M22 11.08V12a10 10 0 11-5.93-9.14" />
              <polyline points="22 4 12 14.01 9 11.01" />
            </svg>
            <p class="text-sm">Successfully imported {result.length} file(s)</p>
            <ul class="imported-list">
              {#each result as path}
                <li class="text-muted text-xs">{getFileName(path)}</li>
              {/each}
            </ul>
            <button class="btn btn-ghost btn-sm" onclick={reset}>Import more</button>
          </div>
        {:else}
          <div class="file-select-area">
            <button class="btn btn-primary" onclick={handleSelectFiles}>
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4" />
                <polyline points="17 8 12 3 7 8" />
                <line x1="12" y1="3" x2="12" y2="15" />
              </svg>
              Select Files
            </button>

            {#if selectedFiles.length > 0}
              <div class="selected-files">
                <p class="text-sm" style="font-weight: 600;">{selectedFiles.length} file(s) selected</p>
                <ul class="file-list">
                  {#each selectedFiles as f}
                    <li class="text-muted text-xs">{getFileName(f)}</li>
                  {/each}
                </ul>
              </div>

              <button class="btn btn-primary" onclick={handleImport} disabled={importing}>
                {importing ? 'Importing...' : `Import ${selectedFiles.length} file(s)`}
              </button>
            {:else}
              <p class="text-muted text-sm" style="margin-top: 12px;">
                Select files to import into your vault. PDF, DOCX, images, and Markdown files are supported.
              </p>
            {/if}

            {#if errorMsg}
              <div class="error-msg">{errorMsg}</div>
            {/if}
          </div>
        {/if}
      </div>

      <div class="modal-footer">
        <button class="btn btn-ghost" onclick={handleClose}>Close</button>
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-overlay {
    position: fixed;
    inset: 0;
    background: rgba(0,0,0,0.4);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: 1000;
  }
  .modal-content {
    background: var(--color-surface);
    border-radius: var(--radius-lg, 12px);
    width: 480px;
    max-width: 90vw;
    max-height: 80vh;
    display: flex;
    flex-direction: column;
    box-shadow: 0 8px 32px rgba(0,0,0,0.2);
  }
  .modal-header {
    padding: 16px 20px;
    border-bottom: 1px solid var(--color-border);
    position: relative;
  }
  .modal-title {
    font-size: 16px;
    font-weight: 700;
  }
  .close-btn {
    position: absolute;
    top: 12px;
    right: 12px;
    padding: 4px;
    border-radius: var(--radius-sm);
    color: var(--color-text-muted);
  }
  .close-btn:hover {
    background: var(--color-bg);
  }
  .modal-body {
    flex: 1;
    overflow-y: auto;
    padding: 20px;
  }
  .modal-footer {
    padding: 12px 20px;
    border-top: 1px solid var(--color-border);
    display: flex;
    justify-content: flex-end;
  }
  .file-select-area {
    display: flex;
    flex-direction: column;
    gap: 12px;
    align-items: center;
    text-align: center;
  }
  .selected-files {
    width: 100%;
    padding: 12px;
    background: var(--color-bg);
    border-radius: var(--radius-md, 8px);
    border: 1px solid var(--color-border);
    text-align: left;
  }
  .file-list {
    margin-top: 8px;
    max-height: 160px;
    overflow-y: auto;
  }
  .file-list li {
    padding: 2px 0;
  }
  .import-success {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    text-align: center;
  }
  .imported-list {
    width: 100%;
    max-height: 200px;
    overflow-y: auto;
    padding: 8px;
    background: var(--color-bg);
    border-radius: var(--radius-md);
  }
  .imported-list li {
    padding: 2px 4px;
  }
  .error-msg {
    font-size: 12px;
    color: var(--color-error, #e53e3e);
    padding: 8px;
    background: var(--color-error-bg, #fff5f5);
    border-radius: var(--radius-sm);
    width: 100%;
    text-align: center;
  }
</style>
