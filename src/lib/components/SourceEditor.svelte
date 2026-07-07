<script lang="ts">
  import { getEditorStore } from '../stores/editor.svelte';
  import { getConfigStore } from '../stores/config.svelte';

  let {
    onContentChange = (_content: string) => {},
    onSave = () => {},
  }: {
    onContentChange?: (content: string) => void;
    onSave?: () => void;
  } = $props();

  const editor = getEditorStore();
  const config = getConfigStore();

  let textareaEl: HTMLTextAreaElement;
  let showLineNumbers = $state(true);
  let debounceTimer: ReturnType<typeof setTimeout> | null = null;

  $effect(() => {
    showLineNumbers = config.editorSettings.lineNumbers;
  });

  function handleInput() {
    const value = textareaEl.value;
    editor.updateContent(value);

    if (debounceTimer) clearTimeout(debounceTimer);
    debounceTimer = setTimeout(() => {
      onContentChange(value);
    }, 500);

    adjustHeight();
  }

  function handleKeydown(e: KeyboardEvent) {
    if ((e.ctrlKey || e.metaKey) && e.key === 's') {
      e.preventDefault();
      onSave();
    }

    if ((e.ctrlKey || e.metaKey) && e.key === 'Enter') {
      e.preventDefault();
      onSave();
    }
  }

  function handleScroll() {
    syncScroll();
  }

  function syncScroll() {
    const gutter = textareaEl?.parentElement?.querySelector('.line-numbers');
    if (gutter) {
      gutter.scrollTop = textareaEl.scrollTop;
    }
  }

  function adjustHeight() {
    if (textareaEl) {
      textareaEl.style.height = 'auto';
      textareaEl.style.height = textareaEl.scrollHeight + 'px';
    }
  }

  $effect(() => {
    if (textareaEl) {
      adjustHeight();
    }
  });

  function handleCursorUpdate() {
    const pos = textareaEl.selectionStart;
    const text = textareaEl.value.substring(0, pos);
    const line = (text.match(/\n/g) || []).length + 1;
    const lastNewline = text.lastIndexOf('\n');
    const col = pos - lastNewline;
    editor.setCursorPosition(line, col);
  }
</script>

<div class="source-editor-wrapper" class:with-line-numbers={showLineNumbers}>
  {#if showLineNumbers}
    <div class="line-numbers" aria-hidden="true">
      {#each editor.content.split('\n') as _, i}
        <span class="line-number">{i + 1}</span>
      {/each}
    </div>
  {/if}
  <textarea
    bind:this={textareaEl}
    class="source-textarea"
    class:with-gutter={showLineNumbers}
    value={editor.content}
    oninput={handleInput}
    onkeydown={handleKeydown}
    onscroll={handleScroll}
    onselect={handleCursorUpdate}
    onmousedown={handleCursorUpdate}
    spellcheck={false}
    placeholder="Start writing in Markdown..."
    aria-label="Source editor"
  ></textarea>
</div>

<style>
  .source-editor-wrapper {
    display: flex;
    height: 100%;
    overflow: hidden;
    background: var(--color-bg);
  }
  .line-numbers {
    display: flex;
    flex-direction: column;
    padding: 12px 8px;
    text-align: right;
    color: var(--color-text-muted);
    font-family: var(--font-code);
    font-size: 13px;
    line-height: 1.7;
    user-select: none;
    overflow: hidden;
    min-width: 40px;
    background: var(--color-surface);
    border-right: 1px solid var(--color-border);
  }
  .line-number {
    display: block;
    line-height: 1.7;
    font-size: 12px;
  }
  .source-textarea {
    flex: 1;
    padding: 12px 16px;
    font-family: var(--font-code);
    font-size: 14px;
    line-height: 1.7;
    border: none;
    border-radius: 0;
    background: transparent;
    resize: none;
    outline: none;
    color: var(--color-text);
    tab-size: 2;
    white-space: pre-wrap;
    word-wrap: break-word;
    overflow-y: auto;
  }
  .source-textarea::placeholder {
    color: var(--color-text-muted);
    opacity: 0.6;
  }
  .source-textarea:focus {
    box-shadow: none;
  }
</style>
