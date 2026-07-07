<script lang="ts">
  import Modal from './Modal.svelte';
  import type { McpConfirmationRequest } from '../types';
  import { getMcpStore } from '../stores/mcp.svelte';

  const mcp = getMcpStore();

  let {
    request = null as McpConfirmationRequest | null,
    onConfirm = () => {},
    onConfirmAlways = () => {},
    onDeny = () => {},
    onClose = () => {},
  }: {
    request?: McpConfirmationRequest | null;
    onConfirm?: () => void;
    onConfirmAlways?: () => void;
    onDeny?: () => void;
    onClose?: () => void;
  } = $props();

  let prettyJson = $derived(
    request ? formatJson(request.args) : '',
  );

  function formatJson(obj: Record<string, unknown>): string {
    try {
      return JSON.stringify(obj, null, 2);
    } catch {
      return String(obj);
    }
  }

  // Syntax-highlight JSON using simple tokenization
  // We just render it in a <pre> with basic coloring via CSS
</script>

{#if request}
  <Modal title="MCP Tool Confirmation" size="md" onclose={onClose}>
    {#snippet children()}
      <div class="confirmation-dialog">
        <div class="warning-banner">
          <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M10.29 3.86L1.82 18a2 2 0 001.71 3h16.94a2 2 0 001.71-3L13.71 3.86a2 2 0 00-3.42 0z" />
            <line x1="12" y1="9" x2="12" y2="13" />
            <line x1="12" y1="17" x2="12.01" y2="17" />
          </svg>
          <span>This tool call requires your approval</span>
        </div>

        <div class="info-grid">
          <div class="info-item">
            <span class="info-label">Server</span>
            <span class="info-value">{request.server}</span>
          </div>
          <div class="info-item">
            <span class="info-label">Tool</span>
            <span class="info-value mono">{request.tool}</span>
          </div>
        </div>

        {#if request.sensitiveOps.length > 0}
          <div class="sensitive-ops">
            <h4 class="section-title">Sensitive Operations Detected</h4>
            <ul class="ops-list">
              {#each request.sensitiveOps as op}
                <li class="op-item">
                  <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="var(--color-warning)" stroke-width="2">
                    <circle cx="12" cy="12" r="10" />
                    <line x1="12" y1="8" x2="12" y2="12" />
                    <line x1="12" y1="16" x2="12.01" y2="16" />
                  </svg>
                  {op}
                </li>
              {/each}
            </ul>
          </div>
        {/if}

        <div class="args-section">
          <h4 class="section-title">Arguments</h4>
          <pre class="json-view"><code>{prettyJson}</code></pre>
        </div>

        <div class="actions">
          <button
            class="btn btn-secondary"
            onclick={onDeny}
          >
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
              <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
            </svg>
            Deny
          </button>
          <button
            class="btn btn-primary"
            onclick={onConfirm}
          >
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
              <path d="M3 8l3 3 7-7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
            Allow Once
          </button>
          <button
            class="btn btn-primary allow-always"
            onclick={onConfirmAlways}
          >
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
              <path d="M3 8l3 3 7-7" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
            </svg>
            Allow Always
          </button>
        </div>
      </div>
    {/snippet}
  </Modal>
{/if}

<style>
  .confirmation-dialog {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .warning-banner {
    display: flex;
    align-items: center;
    gap: 10px;
    padding: 10px 14px;
    background: color-mix(in srgb, var(--color-warning, #f59e0b) 12%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-warning, #f59e0b) 30%, transparent);
    border-radius: var(--radius-md);
    color: var(--color-warning, #f59e0b);
    font-size: 13px;
    font-weight: 500;
  }

  .info-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 12px;
  }

  .info-item {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .info-label {
    font-size: 11px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-muted);
  }

  .info-value {
    font-size: 14px;
    font-weight: 500;
  }

  .info-value.mono {
    font-family: var(--font-code, 'JetBrains Mono', monospace);
    font-size: 13px;
    background: var(--color-surface);
    padding: 2px 8px;
    border-radius: var(--radius-sm);
    display: inline-block;
    width: fit-content;
  }

  .section-title {
    font-size: 12px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--color-text-muted);
    margin: 0 0 8px 0;
  }

  .sensitive-ops {
    padding: 10px 14px;
    background: color-mix(in srgb, var(--color-warning, #f59e0b) 6%, transparent);
    border: 1px solid color-mix(in srgb, var(--color-warning, #f59e0b) 20%, transparent);
    border-radius: var(--radius-md);
  }

  .ops-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }

  .op-item {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    font-weight: 500;
    padding: 4px 0;
  }

  .args-section {
    display: flex;
    flex-direction: column;
  }

  .json-view {
    background: var(--color-surface);
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    padding: 12px;
    overflow-x: auto;
    font-family: var(--font-code, 'JetBrains Mono', monospace);
    font-size: 12px;
    line-height: 1.6;
    max-height: 240px;
    overflow-y: auto;
    margin: 0;
  }

  .json-view code {
    white-space: pre;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding-top: 12px;
    border-top: 1px solid var(--color-border);
  }

  .allow-always {
    background: var(--color-accent);
  }

  .allow-always:hover {
    filter: brightness(1.1);
  }
</style>
