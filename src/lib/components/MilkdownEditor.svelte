<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { createMilkdownEditor, type MilkdownSetupOptions } from '../milkdown/setup';
  import FormatToolbar from './FormatToolbar.svelte';

  let {
    content = '',
    readOnly = false,
    onChange = (_md: string) => {},
    onSave = () => {},
  }: {
    content?: string;
    readOnly?: boolean;
    onChange?: (md: string) => void;
    onSave?: () => void;
  } = $props();

  let containerEl: HTMLDivElement;
  let editorInstance: ReturnType<typeof createMilkdownEditor> | null = null;
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;
  let activeFormats = $state(new Set<string>());

  onMount(() => {
    if (!containerEl) return;

    editorInstance = createMilkdownEditor({
      content,
      readOnly,
      root: containerEl,
      onChange: (md) => {
        if (debounceTimer) clearTimeout(debounceTimer);
        debounceTimer = setTimeout(() => {
          onChange(md);
        }, 500);
      },
      onSave,
      onFormat: handleFormat,
    });

    editorInstance.editor.create().catch(console.error);
  });

  onDestroy(() => {
    if (editorInstance) {
      editorInstance.destroy();
    }
    if (debounceTimer) clearTimeout(debounceTimer);
  });

  function handleFormat(action: string) {
    const next = new Set(activeFormats);
    if (next.has(action)) {
      next.delete(action);
    } else {
      next.add(action);
    }
    activeFormats = next;

    window.dispatchEvent(new CustomEvent('editor:format', { detail: { action } }));
  }

  async function handlePaste(e: ClipboardEvent) {
    const items = e.clipboardData?.items;
    if (!items) return;

    const { handleImagePaste } = await import('../milkdown/image-handler');

    for (const item of Array.from(items)) {
      if (item.type.startsWith('image/')) {
        e.preventDefault();
        const file = item.getAsFile();
        if (!file) continue;

        const reader = new FileReader();
        reader.onload = async (ev) => {
          const dataUrl = ev.target?.result as string;
          const md = await handleImagePaste(dataUrl, file.name);
          if (md && editorInstance) {
            const content = await editorInstance.getContent();
            onChange(content + '\n' + md + '\n');
          }
        };
        reader.readAsDataURL(file);
      }
    }
  }

  async function handleDrop(e: DragEvent) {
    const files = e.dataTransfer?.files;
    if (!files || files.length === 0) return;

    let hasImage = false;
    for (const f of Array.from(files)) {
      if (f.type.startsWith('image/')) { hasImage = true; break; }
    }
    if (!hasImage) return;
    e.preventDefault();

    const { handleImageDrop } = await import('../milkdown/image-handler');
    const markdowns = await handleImageDrop(files);
    if (markdowns.length > 0 && editorInstance) {
      const content = await editorInstance.getContent();
      onChange(content + '\n' + markdowns.join('\n') + '\n');
    }
  }
</script>

<div class="milkdown-wrapper">
  <FormatToolbar {onFormat} {activeFormats} />
  <div
    class="milkdown-editor"
    class:readonly={readOnly}
    bind:this={containerEl}
    onpaste={handlePaste}
    ondrop={handleDrop}
    ondragover={(e) => e.preventDefault()}
    role="textbox"
    aria-label="WYSIWYG editor"
    aria-multiline="true"
  ></div>
</div>

<style>
  .milkdown-wrapper {
    display: flex;
    flex-direction: column;
    height: 100%;
    overflow: hidden;
  }
  .milkdown-editor {
    flex: 1;
    overflow-y: auto;
    padding: 16px 24px;
    font-size: 16px;
    line-height: 1.7;
    outline: none;
  }
  .milkdown-editor.readonly {
    cursor: default;
  }
  .milkdown-editor :global(.editor) {
    max-width: 800px;
    margin: 0 auto;
  }
  .milkdown-editor :global(h1) { font-size: 2em; font-weight: 700; margin: 0.5em 0 0.25em; }
  .milkdown-editor :global(h2) { font-size: 1.5em; font-weight: 600; margin: 0.5em 0 0.25em; }
  .milkdown-editor :global(h3) { font-size: 1.25em; font-weight: 600; margin: 0.5em 0 0.25em; }
  .milkdown-editor :global(p) { margin: 0.5em 0; }
  .milkdown-editor :global(ul), .milkdown-editor :global(ol) { padding-left: 1.5em; margin: 0.5em 0; }
  .milkdown-editor :global(blockquote) {
    border-left: 3px solid var(--color-accent);
    padding-left: 12px;
    margin: 0.5em 0;
    color: var(--color-text-muted);
  }
  .milkdown-editor :global(pre) {
    background: var(--color-surface);
    border-radius: var(--radius-md);
    padding: 12px;
    overflow-x: auto;
  }
  .milkdown-editor :global(code) {
    font-family: var(--font-code);
    font-size: 0.9em;
  }
  .milkdown-editor :global(img) {
    max-width: 100%;
    border-radius: var(--radius-md);
  }
  .milkdown-editor :global(table) {
    border-collapse: collapse;
    width: 100%;
    margin: 0.5em 0;
  }
  .milkdown-editor :global(th), .milkdown-editor :global(td) {
    padding: 8px 12px;
    border: 1px solid var(--color-border);
  }
  .milkdown-editor :global(th) {
    background: var(--color-surface);
    font-weight: 600;
  }
  .milkdown-editor :global(a) {
    color: var(--color-accent);
    text-decoration: underline;
  }
  .milkdown-editor :global(.math) {
    font-family: serif;
  }
  .milkdown-editor :global(.mermaid) {
    text-align: center;
    margin: 1em 0;
  }
</style>
