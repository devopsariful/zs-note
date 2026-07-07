<script lang="ts">
  import { onMount } from 'svelte';

  let { show = true, title = '', size = 'md', onclose }: {
    show?: boolean;
    title?: string;
    size?: 'sm' | 'md' | 'lg';
    onclose?: () => void;
  } = $props();

  let visible = $state(show);
  let animatingOut = $state(false);

  $effect(() => {
    visible = show;
  });

  function handleBackdrop(e: MouseEvent) {
    if (e.target === e.currentTarget) {
      close();
    }
  }

  function close() {
    animatingOut = true;
    setTimeout(() => {
      visible = false;
      animatingOut = false;
      onclose?.();
    }, 150);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Escape') {
      close();
    }
  }

  onMount(() => {
    document.addEventListener('keydown', handleKeydown);
    return () => document.removeEventListener('keydown', handleKeydown);
  });

  const sizeMap: Record<string, string> = {
    sm: '360px',
    md: '560px',
    lg: '800px',
  };
</script>

{#if visible}
  <div
    class="modal-backdrop"
    role="presentation"
    onclick={handleBackdrop}
    class:animating-out={animatingOut}
  >
    <div
      class="modal"
      role="dialog"
      aria-modal="true"
      aria-label={title || 'Modal'}
      style="max-width: {sizeMap[size] || sizeMap.md}"
      class:animating-out={animatingOut}
    >
      <div class="modal-header">
        <h2 class="modal-title">{title}</h2>
        <button class="btn btn-ghost btn-icon" onclick={close} aria-label="Close modal">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
          </svg>
        </button>
      </div>
      <div class="modal-body">
        {@render children?.()}
      </div>
    </div>
  </div>
{/if}

<style>
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.5);
    display: flex;
    align-items: center;
    justify-content: center;
    z-index: var(--z-modal, 50);
    animation: fadeIn 150ms ease;
  }
  .modal-backdrop.animating-out {
    animation: fadeOut 150ms ease;
  }
  .modal {
    background: var(--color-bg);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-lg);
    box-shadow: var(--shadow-lg);
    width: 90vw;
    max-height: 85vh;
    display: flex;
    flex-direction: column;
    animation: slideUp 150ms ease;
  }
  .modal.animating-out {
    animation: slideDown 150ms ease;
  }
  .modal-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 12px 16px;
    border-bottom: 1px solid var(--color-border);
  }
  .modal-title {
    font-size: 16px;
    font-weight: 600;
  }
  .modal-body {
    flex: 1;
    overflow-y: auto;
    padding: 16px;
  }
  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }
  @keyframes fadeOut {
    from { opacity: 1; }
    to { opacity: 0; }
  }
  @keyframes slideUp {
    from { transform: translateY(16px); opacity: 0; }
    to { transform: translateY(0); opacity: 1; }
  }
  @keyframes slideDown {
    from { transform: translateY(0); opacity: 1; }
    to { transform: translateY(16px); opacity: 0; }
  }
</style>
