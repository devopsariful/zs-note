<script lang="ts">
  import { getAIStore } from '../stores/ai.svelte';
  import Modal from './Modal.svelte';
  import TemplatePicker from './TemplatePicker.svelte';

  const ai = getAIStore();

  let {
    visible = false,
    onToggle = () => {},
  }: {
    visible?: boolean;
    onToggle?: () => void;
  } = $props();

  let inputText = $state('');
  let showTemplatePicker = $state(false);
  let showParams = $state(false);
  let messagesEnd: HTMLDivElement;

  $effect(() => {
    if (visible) {
      ai.loadTemplates();
    }
  });

  function handleSend() {
    if (!inputText.trim() || ai.isStreaming) return;
    const text = inputText;
    inputText = '';
    ai.sendMessage(text);
  }

  function handleKeydown(e: KeyboardEvent) {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSend();
    }
  }

  function formatTimestamp(ts: number): string {
    const d = new Date(ts);
    return d.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
  }

  function handleTemplateSelect() {
    showTemplatePicker = true;
  }

  function handleInsert(idx: number) {
    ai.insertResponse(idx);
  }

  function handleReplace(idx: number) {
    ai.replaceSelection(idx);
  }

  function handleCopy(content: string) {
    navigator.clipboard.writeText(content);
  }

  function handleRetry() {
    ai.retryLastMessage();
  }

  function handleTempInput(e: Event) {
    const val = parseFloat((e.target as HTMLInputElement).value);
    if (!isNaN(val)) ai.setTemperature(val);
  }

  function handleMaxTokensInput(e: Event) {
    const val = parseInt((e.target as HTMLInputElement).value, 10);
    if (!isNaN(val)) ai.setMaxTokens(Math.max(1, Math.min(16384, val)));
  }

  function handleTopPInput(e: Event) {
    const val = parseFloat((e.target as HTMLInputElement).value);
    if (!isNaN(val)) ai.setTopP(val);
  }
</script>

<aside class="ai-panel" class:visible={visible} role="complementary" aria-label="AI Chat">
  <div class="panel-header">
    <div class="panel-title-row">
      <h2 class="panel-title">AI Assistant</h2>
      <button class="btn btn-ghost btn-icon" onclick={onToggle} aria-label="Close AI panel">
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
          <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
      </button>
    </div>

    <div class="model-selector">
      <select
        class="model-select"
        value={ai.selectedProvider}
        onchange={(e) => ai.setProvider((e.target as HTMLSelectElement).value)}
      >
        <option value="" disabled>Select provider</option>
        {#each ai.providers as p}
          <option value={p.id}>{p.name}</option>
        {/each}
      </select>
      <select
        class="model-select"
        value={ai.selectedModel}
        onchange={(e) => ai.setModel((e.target as HTMLSelectElement).value)}
      >
        <option value="" disabled>Select model</option>
        {#each (ai.providers.find(p => p.id === ai.selectedProvider)?.models ?? []) as m}
          <option value={m}>{m}</option>
        {/each}
      </select>
    </div>

    <!-- Collapsible Parameters section -->
    <div class="params-section">
      <button
        class="params-toggle"
        onclick={() => { showParams = !showParams; }}
        aria-expanded={showParams}
      >
        <svg
          width="12"
          height="12"
          viewBox="0 0 16 16"
          fill="none"
          class:rotated={showParams}
          style="transition: transform 0.15s ease;"
        >
          <path d="M6 4l4 4-4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        Parameters
      </button>

      {#if showParams}
        <div class="params-body">
          <!-- Temperature -->
          <div class="param-row">
            <label class="param-label" for="ai-temp">Temperature</label>
            <div class="param-control">
              <input
                id="ai-temp"
                type="range"
                min="0"
                max="2"
                step="0.1"
                value={ai.temperature}
                oninput={handleTempInput}
                class="param-slider"
              />
              <span class="param-value">{ai.temperature.toFixed(1)}</span>
            </div>
          </div>

          <!-- Max Tokens -->
          <div class="param-row">
            <label class="param-label" for="ai-maxtokens">Max Tokens</label>
            <div class="param-control">
              <input
                id="ai-maxtokens"
                type="number"
                min="1"
                max="16384"
                value={ai.maxTokens}
                oninput={handleMaxTokensInput}
                class="param-number-input"
              />
            </div>
          </div>

          <!-- Top P -->
          <div class="param-row">
            <label class="param-label" for="ai-topp">Top P</label>
            <div class="param-control">
              <input
                id="ai-topp"
                type="range"
                min="0"
                max="1"
                step="0.05"
                value={ai.topP}
                oninput={handleTopPInput}
                class="param-slider"
              />
              <span class="param-value">{ai.topP.toFixed(2)}</span>
            </div>
          </div>
        </div>
      {/if}
    </div>

    <div class="panel-actions">
      <button
        class="btn btn-ghost btn-sm"
        onclick={handleTemplateSelect}
        title="Use a template"
      >
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
          <rect x="2" y="2" width="12" height="12" rx="2" stroke="currentColor" stroke-width="1.5" />
          <path d="M5 6h6M5 8h6M5 10h4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
        Templates
      </button>
      <button
        class="btn btn-ghost btn-sm"
        onclick={() => ai.clearConversation()}
        title="Clear conversation"
      >
        <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
          <path d="M2 4h12M5 4V3a1 1 0 011-1h4a1 1 0 011 1v1M6 7v5M10 7v5M3 4l1 10a1 1 0 001 1h6a1 1 0 001-1l1-10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
      </button>
    </div>
  </div>

  <div class="messages-container">
    {#if ai.messages.length === 0}
      <div class="empty-chat">
        <div class="empty-icon">
          <svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5">
            <path d="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z" />
          </svg>
        </div>
        <p class="text-muted text-sm">Ask the AI assistant anything about your notes.</p>
      </div>
    {/if}

    {#each ai.messages as msg, idx (msg.id)}
      <div class="message" class:user={msg.role === 'user'} class:assistant={msg.role === 'assistant'}>
        <div class="message-role">{msg.role === 'user' ? 'You' : 'Assistant'}</div>
        <div class="message-content">
          {msg.content || (msg.role === 'assistant' && !msg.done ? '...' : '')}
        </div>
        {#if msg.role === 'assistant' && msg.done && msg.content}
          <div class="message-actions">
            <button class="btn-action" onclick={() => handleInsert(idx)} title="Insert at cursor">
              <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
                <path d="M8 3v10M3 8h10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
              </svg>
              Insert
            </button>
            <button class="btn-action" onclick={() => handleReplace(idx)} title="Replace selection">
              <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
                <path d="M2 12l4-4-4-4M8 12h6" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
              </svg>
              Replace
            </button>
            <button class="btn-action" onclick={() => handleCopy(msg.content)} title="Copy">
              <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
                <rect x="4" y="4" width="10" height="10" rx="1" stroke="currentColor" stroke-width="1.5" />
                <path d="M2 12V3a1 1 0 011-1h9" stroke="currentColor" stroke-width="1.5" />
              </svg>
            </button>
          </div>
        {/if}
        <div class="message-time">{formatTimestamp(msg.timestamp)}</div>
      </div>
    {/each}

    {#if ai.isStreaming}
      <div class="message assistant streaming">
        <div class="message-role">Assistant</div>
        <div class="message-content">
          {#each (ai.messages[ai.messages.length - 1]?.content || '').split('') as char}
            <span>{char}</span>
          {/each}
          <span class="cursor-blink">|</span>
        </div>
      </div>
    {/if}

    <div bind:this={messagesEnd} />
  </div>

  <div class="input-area">
    {#if ai.error}
      <div class="error-bar">
        <span class="text-sm" style="color: var(--color-error)">{ai.error}</span>
      </div>
    {/if}
    <div class="input-row">
      <textarea
        class="input-textarea"
        bind:value={inputText}
        onkeydown={handleKeydown}
        placeholder="Ask AI anything... (Enter to send, Shift+Enter for newline)"
        rows="3"
        disabled={ai.isStreaming}
      ></textarea>
      {#if ai.isStreaming}
        <button class="btn btn-danger send-btn" onclick={() => ai.stopStreaming()}>
          <svg width="14" height="14" viewBox="0 0 16 16" fill="currentColor">
            <rect x="3" y="3" width="10" height="10" rx="1" />
          </svg>
        </button>
      {:else}
        <button
          class="btn btn-primary send-btn"
          onclick={handleSend}
          disabled={!inputText.trim()}
          aria-label="Send message"
        >
          <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
            <path d="M2 8l12-6-6 12-2-4-4-2z" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round" />
          </svg>
        </button>
      {/if}
    </div>
  </div>
</aside>

{#if showTemplatePicker}
  <TemplatePicker
    onClose={() => { showTemplatePicker = false; }}
    onSelect={(prompt: string) => {
      inputText = prompt;
      showTemplatePicker = false;
    }}
  />
{/if}

<style>
  .ai-panel {
    width: var(--aipanel-width);
    min-width: 300px;
    max-width: 500px;
    height: 100%;
    display: flex;
    flex-direction: column;
    background: var(--color-surface);
    border-left: 1px solid var(--color-border);
    transform: translateX(100%);
    transition: transform var(--transition-normal);
    position: absolute;
    right: 0;
    top: 0;
    z-index: var(--z-aipanel, 20);
  }
  .ai-panel.visible {
    transform: translateX(0);
    position: relative;
  }
  .panel-header {
    padding: 8px 12px;
    border-bottom: 1px solid var(--color-border);
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .panel-title-row {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }
  .panel-title {
    font-size: 13px;
    font-weight: 600;
  }
  .model-selector {
    display: flex;
    gap: 6px;
  }
  .model-select {
    flex: 1;
    font-size: 12px;
    padding: 4px 6px;
  }
  .panel-actions {
    display: flex;
    gap: 4px;
  }
  .panel-actions .btn-sm {
    font-size: 11px;
    padding: 3px 8px;
  }

  /* Parameters section */
  .params-section {
    border-top: 1px solid var(--color-border);
    padding-top: 6px;
  }
  .params-toggle {
    display: flex;
    align-items: center;
    gap: 4px;
    font-size: 11px;
    font-weight: 600;
    color: var(--color-text-muted);
    padding: 2px 4px;
    border-radius: var(--radius-sm);
    width: 100%;
    text-align: left;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    transition: color var(--transition-fast);
  }
  .params-toggle:hover {
    color: var(--color-text);
  }
  .params-toggle svg.rotated {
    transform: rotate(90deg);
  }
  .params-body {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 6px 0 2px;
  }
  .param-row {
    display: flex;
    flex-direction: column;
    gap: 2px;
  }
  .param-label {
    font-size: 11px;
    font-weight: 500;
    color: var(--color-text-muted);
  }
  .param-control {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .param-slider {
    flex: 1;
    height: 4px;
    -webkit-appearance: none;
    appearance: none;
    background: var(--color-border);
    border-radius: 2px;
    outline: none;
    cursor: pointer;
  }
  .param-slider::-webkit-slider-thumb {
    -webkit-appearance: none;
    appearance: none;
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--color-accent);
    border: 2px solid var(--color-bg);
    box-shadow: 0 1px 3px rgba(0,0,0,0.2);
    cursor: pointer;
  }
  .param-slider::-moz-range-thumb {
    width: 14px;
    height: 14px;
    border-radius: 50%;
    background: var(--color-accent);
    border: 2px solid var(--color-bg);
    box-shadow: 0 1px 3px rgba(0,0,0,0.2);
    cursor: pointer;
  }
  .param-value {
    font-size: 11px;
    font-weight: 600;
    color: var(--color-text);
    min-width: 32px;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }
  .param-number-input {
    width: 80px;
    font-size: 12px;
    padding: 2px 6px;
    text-align: right;
    font-variant-numeric: tabular-nums;
  }

  .messages-container {
    flex: 1;
    overflow-y: auto;
    padding: 8px 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .empty-chat {
    flex: 1;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 8px;
    color: var(--color-text-muted);
    opacity: 0.6;
  }
  .empty-icon {
    opacity: 0.5;
  }
  .message {
    padding: 8px 10px;
    border-radius: var(--radius-md);
    font-size: 13px;
    line-height: 1.5;
    max-width: 90%;
  }
  .message.user {
    align-self: flex-end;
    background: var(--color-accent);
    color: white;
  }
  .message.assistant {
    align-self: flex-start;
    background: var(--color-bg);
    border: 1px solid var(--color-border);
  }
  .message.streaming {
    border-color: var(--color-accent);
  }
  .message-role {
    font-size: 10px;
    font-weight: 600;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    margin-bottom: 4px;
    opacity: 0.7;
  }
  .message.user .message-role {
    color: rgba(255,255,255,0.7);
  }
  .message-content {
    white-space: pre-wrap;
    word-wrap: break-word;
  }
  .message-time {
    font-size: 10px;
    opacity: 0.5;
    margin-top: 4px;
    text-align: right;
  }
  .message-actions {
    display: flex;
    gap: 4px;
    margin-top: 6px;
    padding-top: 6px;
    border-top: 1px solid var(--color-border);
  }
  .btn-action {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    padding: 2px 6px;
    font-size: 11px;
    border-radius: var(--radius-sm);
    color: var(--color-text-muted);
    transition: all var(--transition-fast);
  }
  .btn-action:hover {
    background: var(--color-surface);
    color: var(--color-text);
  }
  .cursor-blink {
    animation: blink 1s step-end infinite;
    color: var(--color-accent);
  }
  @keyframes blink {
    50% { opacity: 0; }
  }
  .input-area {
    padding: 8px 12px;
    border-top: 1px solid var(--color-border);
  }
  .error-bar {
    padding: 4px 8px;
    margin-bottom: 6px;
    background: var(--color-error-bg);
    border-radius: var(--radius-sm);
  }
  .input-row {
    display: flex;
    gap: 6px;
    align-items: flex-end;
  }
  .input-textarea {
    flex: 1;
    padding: 8px 10px;
    font-size: 13px;
    line-height: 1.5;
    border-radius: var(--radius-md);
    resize: none;
    max-height: 120px;
    font-family: var(--font-ui);
  }
  .send-btn {
    width: 32px;
    height: 32px;
    padding: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    border-radius: 50%;
    flex-shrink: 0;
  }
</style>
