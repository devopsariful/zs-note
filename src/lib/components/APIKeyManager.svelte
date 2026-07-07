<script lang="ts">
  import { storeApiKey, getApiKey, deleteApiKey, listApiKeys } from '../commands/credentials';

  interface ProviderInfo {
    id: string;
    name: string;
    url: string;
    keyLabel: string;
  }

  const providers: ProviderInfo[] = [
    { id: 'openai', name: 'OpenAI', url: 'https://platform.openai.com/api-keys', keyLabel: 'sk-...' },
    { id: 'anthropic', name: 'Anthropic Claude', url: 'https://console.anthropic.com/keys', keyLabel: 'sk-ant-...' },
    { id: 'google', name: 'Google Gemini', url: 'https://aistudio.google.com/app/apikey', keyLabel: 'AIza...' },
    { id: 'deepseek', name: 'DeepSeek', url: 'https://platform.deepseek.com/api_keys', keyLabel: 'sk-...' },
  ];

  let {
    show = false,
    onClose = () => {},
  }: {
    show?: boolean;
    onClose?: () => void;
  } = $props();

  let keyStates = $state<Record<string, { key: string; hasKey: boolean; saving: boolean; visible: boolean; error: string }>>({});
  let loading = $state(true);
  let globalError = $state('');

  $effect(() => {
    if (show) {
      loadKeys();
    }
  });

  async function loadKeys() {
    loading = true;
    globalError = '';
    const states: Record<string, any> = {};

    for (const p of providers) {
      states[p.id] = { key: '', hasKey: false, saving: false, visible: false, error: '' };
      try {
        const existing = await getApiKey(p.id);
        if (existing) {
          states[p.id].hasKey = true;
          states[p.id].key = existing;
        }
      } catch (e) {
        states[p.id].error = String(e);
      }
    }

    keyStates = states;
    loading = false;
  }

  async function handleSave(providerId: string) {
    const state = keyStates[providerId];
    if (!state || !state.key.trim()) return;
    state.saving = true;
    state.error = '';
    try {
      await storeApiKey(providerId, state.key.trim());
      state.hasKey = true;
      state.key = state.key.trim();
    } catch (e) {
      state.error = String(e);
    }
    state.saving = false;
  }

  async function handleDelete(providerId: string) {
    const state = keyStates[providerId];
    if (!state) return;
    state.saving = true;
    state.error = '';
    try {
      await deleteApiKey(providerId);
      state.key = '';
      state.hasKey = false;
    } catch (e) {
      state.error = String(e);
    }
    state.saving = false;
  }

  function toggleVisibility(providerId: string) {
    const state = keyStates[providerId];
    if (state) state.visible = !state.visible;
  }
</script>

{#if show}
  <div class="modal-overlay" role="dialog" aria-label="API Key Manager" onclick={onClose}>
    <div class="modal-content" onclick={(e) => e.stopPropagation()}>
      <div class="modal-header">
        <h2 class="modal-title">API Key Manager</h2>
        <p class="text-muted text-sm">API keys are stored in your operating system keychain.</p>
        <button class="close-btn" onclick={onClose} aria-label="Close">
          <svg width="16" height="16" viewBox="0 0 16 16" fill="none">
            <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="2" stroke-linecap="round" />
          </svg>
        </button>
      </div>

      <div class="modal-body">
        {#if loading}
          <div class="loading">Loading keychain...</div>
        {:else}
          {#each providers as p}
            <div class="provider-row">
              <div class="provider-info">
                <span class="provider-name">{p.name}</span>
                <a href={p.url} target="_blank" rel="noopener noreferrer" class="provider-link">
                  Get API key →
                </a>
              </div>

              <div class="key-input-row">
                <div class="key-input-wrapper">
                  <input
                    type={keyStates[p.id]?.visible ? 'text' : 'password'}
                    class="key-input"
                    placeholder={p.keyLabel}
                    value={keyStates[p.id]?.key ?? ''}
                    oninput={(e) => { if (keyStates[p.id]) keyStates[p.id].key = (e.target as HTMLInputElement).value; }}
                    disabled={keyStates[p.id]?.hasKey}
                  />
                  <button
                    class="toggle-vis-btn"
                    onclick={() => toggleVisibility(p.id)}
                    aria-label={keyStates[p.id]?.visible ? 'Hide key' : 'Show key'}
                    tabindex="-1"
                  >
                    <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                      {#if keyStates[p.id]?.visible}
                        <path d="M17.94 17.94A10.07 10.07 0 0112 20c-7 0-11-8-11-8a18.45 18.45 0 015.06-5.94M9.9 4.24A9.12 9.12 0 0112 4c7 0 11 8 11 8a18.5 18.5 0 01-2.16 3.19m-6.72-1.07a3 3 0 11-4.24-4.24" />
                        <line x1="1" y1="1" x2="23" y2="23" />
                      {:else}
                        <path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" />
                        <circle cx="12" cy="12" r="3" />
                      {/if}
                    </svg>
                  </button>
                </div>

                <div class="key-actions">
                  {#if keyStates[p.id]?.hasKey}
                    <button class="btn btn-ghost btn-sm" onclick={() => handleDelete(p.id)} disabled={keyStates[p.id]?.saving}>
                      {keyStates[p.id]?.saving ? '...' : 'Remove'}
                    </button>
                    <span class="key-status saved">Saved ✓</span>
                  {:else}
                    <button class="btn btn-primary btn-sm" onclick={() => handleSave(p.id)} disabled={!keyStates[p.id]?.key?.trim() || keyStates[p.id]?.saving}>
                      {keyStates[p.id]?.saving ? 'Saving...' : 'Save'}
                    </button>
                  {/if}
                </div>
              </div>

              {#if keyStates[p.id]?.error}
                <div class="error-msg">{keyStates[p.id]?.error}</div>
              {/if}
            </div>
          {/each}
        {/if}
      </div>

      <div class="modal-footer">
        <button class="btn btn-ghost" onclick={onClose}>Done</button>
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
    width: 520px;
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
    margin-bottom: 4px;
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
    padding: 16px 20px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }
  .modal-footer {
    padding: 12px 20px;
    border-top: 1px solid var(--color-border);
    display: flex;
    justify-content: flex-end;
  }
  .loading {
    text-align: center;
    padding: 24px;
    color: var(--color-text-muted);
  }
  .provider-row {
    padding: 12px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md, 8px);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .provider-info {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .provider-name {
    font-weight: 600;
    font-size: 14px;
  }
  .provider-link {
    font-size: 12px;
    color: var(--color-accent);
    text-decoration: none;
  }
  .provider-link:hover {
    text-decoration: underline;
  }
  .key-input-row {
    display: flex;
    gap: 8px;
    align-items: center;
  }
  .key-input-wrapper {
    flex: 1;
    display: flex;
    align-items: center;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm, 6px);
    overflow: hidden;
  }
  .key-input {
    flex: 1;
    padding: 6px 10px;
    font-size: 13px;
    font-family: var(--font-code, monospace);
    border: none;
    outline: none;
    background: transparent;
  }
  .toggle-vis-btn {
    padding: 4px 8px;
    color: var(--color-text-muted);
  }
  .toggle-vis-btn:hover {
    color: var(--color-text);
  }
  .key-actions {
    display: flex;
    align-items: center;
    gap: 6px;
    flex-shrink: 0;
  }
  .key-status.saved {
    font-size: 11px;
    color: var(--color-success, #38a169);
    font-weight: 500;
  }
  .error-msg {
    font-size: 12px;
    color: var(--color-error, #e53e3e);
    padding: 4px 8px;
    background: var(--color-error-bg, #fff5f5);
    border-radius: var(--radius-sm);
  }
</style>
