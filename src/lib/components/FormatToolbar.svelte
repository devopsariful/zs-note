<script lang="ts">
  import VoiceRecorder from './VoiceRecorder.svelte';

  interface FormatAction {
    label: string;
    icon: string;
    action: string;
    shortcut?: string;
    active?: () => boolean;
  }

  let {
    onFormat = (_action: string) => {},
    activeFormats = new Set<string>(),
    onTranscript = (_text: string) => {},
  }: {
    onFormat?: (action: string) => void;
    activeFormats?: Set<string>;
    onTranscript?: (text: string) => void;
  } = $props();

  const actions: FormatAction[] = [
    { label: 'Heading 1', icon: 'H1', action: 'h1', shortcut: '⌘1' },
    { label: 'Heading 2', icon: 'H2', action: 'h2', shortcut: '⌘2' },
    { label: 'Heading 3', icon: 'H3', action: 'h3', shortcut: '⌘3' },
    { label: 'Bold', icon: 'B', action: 'bold', shortcut: '⌘B' },
    { label: 'Italic', icon: 'I', action: 'italic', shortcut: '⌘I' },
    { label: 'Strikethrough', icon: 'S', action: 'strikethrough', shortcut: '⌘⇧S' },
    { label: 'Link', icon: '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"/><path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"/></svg>', action: 'link', shortcut: '⌘K' },
    { label: 'Image', icon: '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><rect x="3" y="3" width="18" height="18" rx="2" ry="2"/><circle cx="8.5" cy="8.5" r="1.5"/><polyline points="21 15 16 10 5 21"/></svg>', action: 'image' },
    { label: 'Code', icon: '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="16 18 22 12 16 6"/><polyline points="8 6 2 12 8 18"/></svg>', action: 'code', shortcut: '⌘⇧C' },
    { label: 'Math', icon: '∑', action: 'math', shortcut: '⌘⇧M' },
    { label: 'Table', icon: '⊞', action: 'table' },
    { label: 'Bullet List', icon: '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="8" y1="6" x2="21" y2="6"/><line x1="8" y1="12" x2="21" y2="12"/><line x1="8" y1="18" x2="21" y2="18"/><circle cx="3" cy="6" r="0.5" fill="currentColor"/><circle cx="3" cy="12" r="0.5" fill="currentColor"/><circle cx="3" cy="18" r="0.5" fill="currentColor"/></svg>', action: 'bulletList' },
    { label: 'Ordered List', icon: '1.', action: 'orderedList' },
    { label: 'Task List', icon: '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><polyline points="9 11 12 14 22 4"/><path d="M21 12v7a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11"/></svg>', action: 'taskList' },
    { label: 'Blockquote', icon: '"', action: 'blockquote', shortcut: '⌘⇧.' },
    { label: 'Horizontal Rule', icon: '—', action: 'hr' },
    { label: 'Diagram', icon: '◇', action: 'diagram' },
  ];

  function isActive(action: string): boolean {
    return activeFormats.has(action);
  }
</script>

<div class="toolbar" role="toolbar" aria-label="Formatting toolbar">
  {#each actions as act}
    <button
      class="toolbar-btn"
      class:active={isActive(act.action)}
      onclick={() => onFormat(act.action)}
      title={act.shortcut ? `${act.label} (${act.shortcut})` : act.label}
      aria-label={act.label}
      aria-pressed={isActive(act.action)}
    >
      <span class="toolbar-icon">{@html act.icon}</span>
    </button>
    {#if act.action === 'h3' || act.action === 'strikethrough' || act.action === 'image' || act.action === 'table' || act.action === 'blockquote'}
      <div class="separator" />
    {/if}
  {/each}

  <div class="spacer" />

  <VoiceRecorder onTranscript={(text) => onTranscript(text)} />
</div>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 1px;
    padding: 4px 8px;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    overflow-x: auto;
    flex-wrap: nowrap;
  }
  .toolbar-btn {
    display: inline-flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    padding: 0;
    border-radius: var(--radius-sm);
    font-size: 12px;
    font-weight: 600;
    transition: all var(--transition-fast);
    flex-shrink: 0;
  }
  .toolbar-btn:hover {
    background: color-mix(in srgb, var(--color-accent) 10%, transparent);
  }
  .toolbar-btn.active {
    background: color-mix(in srgb, var(--color-accent) 20%, transparent);
    color: var(--color-accent);
  }
  .toolbar-icon {
    line-height: 1;
  }
  .separator {
    width: 1px;
    height: 20px;
    background: var(--color-border);
    margin: 0 4px;
    flex-shrink: 0;
  }
  .spacer {
    flex: 1;
    min-width: 4px;
  }
</style>
