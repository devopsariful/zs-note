<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { getEditorStore } from '../stores/editor.svelte';
  import MilkdownEditor from './MilkdownEditor.svelte';
  import SourceEditor from './SourceEditor.svelte';

  const editor = getEditorStore();

  let {
    content = '',
    filePath = '',
    mode = 'wysiwyg' as 'wysiwyg' | 'source' | 'split',
    onContentChange = (_content: string) => {},
    onSave = () => {},
  }: {
    content?: string;
    filePath?: string;
    mode?: 'wysiwyg' | 'source' | 'split';
    onContentChange?: (content: string) => void;
    onSave?: () => void;
  } = $props();

  let showNoFile = $derived(!filePath && !content);
  let splitRatio = $state(50);

  // ── Drag-and-drop ingestion state ─────────────────────────────────────────
  let isDragOver = $state(false);
  let dragCounter = $state(0);

  const acceptedExtensions = [
    '.md', '.markdown', '.txt',
    '.pdf', '.docx', '.pptx', '.xlsx',
    '.epub', '.csv', '.html', '.htm',
    '.png', '.jpg', '.jpeg', '.gif', '.webp', '.svg',
    '.bmp', '.tiff', '.tif',
  ];

  function handleDragOver(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    if (e.dataTransfer) {
      e.dataTransfer.dropEffect = 'copy';
    }
  }

  function handleDragEnter(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    dragCounter++;
    if (dragCounter > 0) {
      isDragOver = true;
    }
  }

  function handleDragLeave(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    dragCounter--;
    if (dragCounter <= 0) {
      dragCounter = 0;
      isDragOver = false;
    }
  }

  async function handleDrop(e: DragEvent) {
    e.preventDefault();
    e.stopPropagation();
    isDragOver = false;
    dragCounter = 0;

    const files = e.dataTransfer?.files;
    if (!files || files.length === 0) return;

    const fileArray = Array.from(files);
    const supported = fileArray.filter((f) => {
      const ext = '.' + f.name.split('.').pop()?.toLowerCase();
      return acceptedExtensions.includes(ext);
    });

    if (supported.length === 0) {
      editor.error = 'No supported files dropped. Accepted formats: ' + acceptedExtensions.join(', ');
      return;
    }

    // Process each file
    for (const file of supported) {
      const ext = '.' + file.name.split('.').pop()?.toLowerCase();
      editor.addIngestItem({
        fileName: file.name,
        percent: 0,
        status: 'pending',
      });

      try {
        // Read the file as ArrayBuffer
        const buffer = await file.arrayBuffer();
        editor.updateIngestProgress(file.name, 20, 'ingesting');

        // Write the file to a temp location in the vault using Tauri fs plugin
        const tempDir = await invoke<string>('get_temp_dir');
        const tempFilePath = `${tempDir}/${file.name}`;

        await invoke('write_file', {
          path: tempFilePath,
          content: Array.from(new Uint8Array(buffer)),
        });

        editor.updateIngestProgress(file.name, 50, 'ingesting');

        // Invoke the ingest command with the file path
        const outputName = file.name.replace(/\.[^.]+$/, '.md');
        const outputPath = filePath
          ? filePath.substring(0, filePath.lastIndexOf('/') + 1) + outputName
          : outputName;

        await invoke('ingest_file', {
          source: tempFilePath,
          outputPath,
          mimeHint: file.type || null,
        });

        // Clean up temp file
        try {
          await invoke('delete_file', { path: tempFilePath });
        } catch {
          // Ignore cleanup errors
        }

        editor.updateIngestProgress(file.name, 100, 'done');
      } catch (err) {
        editor.updateIngestProgress(file.name, 0, 'error', String(err));
      }
    }
  }

  function handleContentChange(newContent: string) {
    editor.updateContent(newContent);
    onContentChange(newContent);
  }

  function handleSave() {
    editor.saveFile().then(() => onSave());
  }

  function handleSplitDrag(e: MouseEvent) {
    e.preventDefault();
    const container = (e.target as HTMLElement).parentElement;
    if (!container) return;
    const rect = container.getBoundingClientRect();

    function onMove(ev: MouseEvent) {
      const pct = ((ev.clientX - rect.left) / rect.width) * 100;
      splitRatio = Math.max(20, Math.min(80, pct));
    }

    function onUp() {
      document.removeEventListener('mousemove', onMove);
      document.removeEventListener('mouseup', onUp);
    }

    document.addEventListener('mousemove', onMove);
    document.addEventListener('mouseup', onUp);
  }
</script>

<div
  class="editor-viewport"
  ondragover={handleDragOver}
  ondragenter={handleDragEnter}
  ondragleave={handleDragLeave}
  ondrop={handleDrop}
  role="region"
  aria-label="Editor"
>
  {#if showNoFile}
    <div class="empty-editor">
      <div class="empty-icon">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M14 2H6a2 2 0 00-2 2v16a2 2 0 002 2h12a2 2 0 002-2V8z" />
          <polyline points="14 2 14 8 20 8" />
          <line x1="12" y1="18" x2="12" y2="12" />
          <line x1="9" y1="15" x2="15" y2="15" />
        </svg>
      </div>
      <h2 class="empty-title">No file open</h2>
      <p class="empty-description text-muted">
        Select a file from the sidebar or create a new note to get started.
      </p>
      <p class="empty-hint text-muted text-sm">
        Drop files here to ingest them into your knowledge base (.md, .txt, .pdf, .docx, .pptx, .xlsx, .epub, .csv, .html, images)
      </p>
    </div>
  {:else if mode === 'wysiwyg'}
    <MilkdownEditor
      content={editor.content}
      onChange={handleContentChange}
      onSave={handleSave}
    />
  {:else if mode === 'source'}
    <SourceEditor
      onContentChange={handleContentChange}
      onSave={handleSave}
    />
  {:else if mode === 'split'}
    <div class="split-container">
      <div class="split-pane source-pane" style="width: {splitRatio}%">
        <div class="pane-label">Source</div>
        <SourceEditor
          onContentChange={handleContentChange}
          onSave={handleSave}
        />
      </div>
      <div
        class="split-divider"
        role="separator"
        tabindex="0"
        onmousedown={handleSplitDrag}
        aria-label="Resize editor panes"
      >
        <div class="divider-handle" />
      </div>
      <div class="split-pane preview-pane" style="flex: 1">
        <div class="pane-label">Preview</div>
        <MilkdownEditor
          content={editor.content}
          readOnly={true}
          onChange={() => {}}
          onSave={() => {}}
        />
      </div>
    </div>
  {/if}

  <!-- Drag-over overlay -->
  {#if isDragOver}
    <div class="drag-overlay" aria-live="polite">
      <div class="drag-overlay-content">
        <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
          <path d="M21 15v4a2 2 0 01-2 2H5a2 2 0 01-2-2v-4" />
          <polyline points="7 10 12 15 17 10" />
          <line x1="12" y1="15" x2="12" y2="3" />
        </svg>
        <h3>Drop files to ingest</h3>
        <p class="text-sm text-muted">Supported: .md, .txt, .pdf, .docx, .pptx, .xlsx, .epub, .csv, .html, images</p>
        {#if editor.ingestProgress.length > 0}
          <div class="ingest-progress-list">
            {#each editor.ingestProgress as item}
              <div class="ingest-item">
                <span class="ingest-name">{item.fileName}</span>
                <span class="ingest-status" class:done={item.status === 'done'} class:error={item.status === 'error'}>
                  {#if item.status === 'pending'}⏳ Pending{/if}
                  {#if item.status === 'ingesting'}⏳ Ingesting ({item.percent}%){/if}
                  {#if item.status === 'indexing'}📚 Indexing...{/if}
                  {#if item.status === 'done'}✅ Done{/if}
                  {#if item.status === 'error'}❌ Error{/if}
                </span>
              </div>
              {#if item.error}
                <div class="ingest-error">{item.error}</div>
              {/if}
            {/each}
          </div>
        {/if}
      </div>
    </div>
  {/if}
</div>

<style>
  .editor-viewport {
    flex: 1;
    display: flex;
    flex-direction: column;
    overflow: hidden;
    background: var(--color-bg);
    position: relative;
  }
  .empty-editor {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    padding: 32px;
  }
  .empty-icon {
    color: var(--color-text-muted);
    opacity: 0.4;
  }
  .empty-title {
    font-size: 18px;
    font-weight: 600;
  }
  .empty-description {
    font-size: 14px;
    text-align: center;
    max-width: 320px;
  }
  .empty-hint {
    font-size: 12px;
    text-align: center;
    max-width: 400px;
    padding: 8px 16px;
    background: var(--color-surface);
    border-radius: var(--radius-md);
    border: 1px dashed var(--color-border);
    margin-top: 8px;
  }
  .split-container {
    display: flex;
    flex: 1;
    overflow: hidden;
  }
  .split-pane {
    display: flex;
    flex-direction: column;
    overflow: hidden;
    position: relative;
  }
  .source-pane {
    border-right: 1px solid var(--color-border);
  }
  .pane-label {
    position: absolute;
    top: 4px;
    right: 8px;
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-muted);
    z-index: 1;
    padding: 2px 6px;
    background: var(--color-surface);
    border-radius: var(--radius-sm);
    opacity: 0.7;
    pointer-events: none;
  }
  .split-divider {
    width: 4px;
    cursor: col-resize;
    background: transparent;
    display: flex;
    align-items: center;
    justify-content: center;
    flex-shrink: 0;
    position: relative;
    z-index: 2;
  }
  .split-divider:hover, .split-divider:focus-visible {
    background: color-mix(in srgb, var(--color-accent) 20%, transparent);
  }
  .divider-handle {
    width: 2px;
    height: 32px;
    border-radius: 1px;
    background: var(--color-border);
  }
  .split-divider:hover .divider-handle {
    background: var(--color-accent);
  }

  /* Drag overlay */
  .drag-overlay {
    position: absolute;
    inset: 0;
    background: color-mix(in srgb, var(--color-accent) 12%, var(--color-bg));
    backdrop-filter: blur(2px);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: var(--z-drag-overlay, 30);
    animation: fadeIn 150ms ease;
    border: 2px dashed var(--color-accent);
    pointer-events: none;
  }
  .drag-overlay-content {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 12px;
    padding: 32px;
    color: var(--color-accent);
    text-align: center;
  }
  .drag-overlay-content h3 {
    font-size: 18px;
    font-weight: 600;
  }
  .ingest-progress-list {
    width: 100%;
    max-width: 400px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    margin-top: 8px;
    background: var(--color-bg);
    border-radius: var(--radius-md);
    padding: 8px 12px;
    border: 1px solid var(--color-border);
  }
  .ingest-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 8px;
    font-size: 12px;
    padding: 2px 0;
  }
  .ingest-name {
    font-weight: 500;
    flex: 1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }
  .ingest-status {
    font-size: 11px;
    color: var(--color-text-muted);
    flex-shrink: 0;
  }
  .ingest-status.done {
    color: var(--color-success, #22c55e);
  }
  .ingest-status.error {
    color: var(--color-error);
  }
  .ingest-error {
    font-size: 11px;
    color: var(--color-error);
    padding: 2px 0 2px 12px;
  }
  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }
</style>

