<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import Modal from './Modal.svelte';

  interface GeneratedImageResult {
    data: string;
    mime_type: string;
    model: string;
    prompt: string;
    seed: number | null;
    revised_prompt: string | null;
  }

  let {
    show = false,
    onClose = () => {},
    onInsert = (_dataUrl: string) => {},
  }: {
    show?: boolean;
    onClose?: () => void;
    onInsert?: (dataUrl: string) => void;
  } = $props();

  let prompt = $state('');
  let selectedModel = $state('dall-e-3');
  let selectedSize = $state('1024x1024');
  let selectedQuality = $state('standard');
  let isGenerating = $state(false);
  let error = $state<string | null>(null);
  let result = $state<GeneratedImageResult | null>(null);

  const models = [
    { id: 'dall-e-3', name: 'DALL-E 3' },
    { id: 'dall-e-2', name: 'DALL-E 2' },
    { id: 'stability-ai', name: 'Stability AI (SD3)' },
  ];

  const sizes = ['1024x1024', '1792x1024', '1024x1792', '768x768', '512x512'];

  const qualityOptions = ['standard', 'hd'];

  async function handleGenerate() {
    if (!prompt.trim()) return;

    isGenerating = true;
    error = null;
    result = null;

    try {
      const imageResult = await invoke<GeneratedImageResult>('generate_image', {
        prompt: prompt.trim(),
        model: selectedModel,
        size: selectedSize,
        quality: selectedQuality,
      });
      result = imageResult;
    } catch (err) {
      error = String(err);
    } finally {
      isGenerating = false;
    }
  }

  function handleInsert() {
    if (!result) return;
    const dataUrl = `data:${result.mime_type};base64,${result.data}`;
    onInsert(dataUrl);
    onClose();
  }

  function handleSave() {
    if (!result) return;
    // Dispatch custom event so the app can handle saving to assets/
    const event = new CustomEvent('image:save-to-assets', {
      detail: {
        data: result.data,
        mimeType: result.mime_type,
        prompt: result.prompt,
        model: result.model,
      },
    });
    window.dispatchEvent(event);
    onClose();
  }

  function handleClose() {
    prompt = '';
    error = null;
    result = null;
    onClose();
  }

  function getImageDataUrl(): string | null {
    if (!result) return null;
    return `data:${result.mime_type};base64,${result.data}`;
  }
</script>

<Modal show={show} title="Generate Image" size="lg" onclose={handleClose}>
  {#snippet children()}
    <div class="image-gen-dialog">
      <div class="dialog-body">
        <div class="input-column">
          <div class="input-group">
            <label for="ig-prompt">Prompt</label>
            <textarea
              id="ig-prompt"
              class="prompt-input"
              bind:value={prompt}
              placeholder="Describe the image you want to generate..."
              rows="4"
              disabled={isGenerating}
            ></textarea>
          </div>

          <div class="input-row">
            <div class="input-group">
              <label for="ig-model">Model</label>
              <select
                id="ig-model"
                bind:value={selectedModel}
                disabled={isGenerating}
              >
                {#each models as m}
                  <option value={m.id}>{m.name}</option>
                {/each}
              </select>
            </div>

            <div class="input-group">
              <label for="ig-size">Size</label>
              <select
                id="ig-size"
                bind:value={selectedSize}
                disabled={isGenerating}
              >
                {#each sizes as s}
                  <option value={s}>{s}</option>
                {/each}
              </select>
            </div>

            {#if selectedModel === 'dall-e-3'}
              <div class="input-group">
                <label for="ig-quality">Quality</label>
                <select
                  id="ig-quality"
                  bind:value={selectedQuality}
                  disabled={isGenerating}
                >
                  {#each qualityOptions as q}
                    <option value={q}>{q}</option>
                  {/each}
                </select>
              </div>
            {/if}
          </div>

          <button
            class="btn btn-primary generate-btn"
            onclick={handleGenerate}
            disabled={!prompt.trim() || isGenerating}
          >
            {#if isGenerating}
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="spinner">
                <circle cx="12" cy="12" r="10" opacity="0.3" />
                <path d="M12 2a10 10 0 0 1 10 10" />
              </svg>
              Generating…
            {:else}
              <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round">
                <rect x="3" y="3" width="18" height="18" rx="2" />
                <circle cx="8.5" cy="8.5" r="1.5" />
                <polyline points="21 15 16 10 5 21" />
              </svg>
              Generate
            {/if}
          </button>

          {#if error}
            <div class="error-message">
              <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
                <circle cx="8" cy="8" r="6" stroke="currentColor" stroke-width="1.5" />
                <path d="M8 5v3M8 10.5v.5" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
              </svg>
              {error}
            </div>
          {/if}
        </div>

        <div class="preview-column">
          {#if isGenerating}
            <div class="preview-placeholder">
              <div class="generating-indicator">
                <svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" class="spinner">
                  <circle cx="12" cy="12" r="10" opacity="0.3" />
                  <path d="M12 2a10 10 0 0 1 10 10" />
                </svg>
                <span>Generating image…</span>
              </div>
            </div>
          {:else if result}
            <div class="preview-container">
              <img
                class="preview-image"
                src={getImageDataUrl()}
                alt={result.revised_prompt ?? result.prompt}
              />
              {#if result.revised_prompt}
                <p class="revised-prompt text-sm text-muted">{result.revised_prompt}</p>
              {/if}
            </div>
          {:else}
            <div class="preview-placeholder">
              <svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" opacity="0.3">
                <rect x="3" y="3" width="18" height="18" rx="2" />
                <circle cx="8.5" cy="8.5" r="1.5" />
                <polyline points="21 15 16 10 5 21" />
              </svg>
              <span class="text-muted text-sm">Generated image will appear here</span>
            </div>
          {/if}
        </div>
      </div>

      {#if result}
        <div class="dialog-footer">
          <button class="btn btn-primary" onclick={handleInsert}>
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
              <path d="M8 3v10M3 8h10" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
            </svg>
            Insert into document
          </button>
          <button class="btn btn-secondary" onclick={handleSave}>
            <svg width="14" height="14" viewBox="0 0 16 16" fill="none">
              <path d="M13 13H3V3h5l5 5v5z" stroke="currentColor" stroke-width="1.5" stroke-linejoin="round" />
              <path d="M6 13V9h4v4" stroke="currentColor" stroke-width="1.5" />
            </svg>
            Save to assets/
          </button>
        </div>
      {/if}
    </div>
  {/snippet}
</Modal>

<style>
  .image-gen-dialog {
    display: flex;
    flex-direction: column;
    gap: 16px;
    min-height: 400px;
  }
  .dialog-body {
    display: flex;
    gap: 16px;
    flex: 1;
  }
  .input-column {
    flex: 1;
    display: flex;
    flex-direction: column;
    gap: 12px;
    min-width: 240px;
  }
  .prompt-input {
    font-family: var(--font-ui);
    min-height: 80px;
    resize: vertical;
  }
  .input-row {
    display: flex;
    gap: 8px;
    flex-wrap: wrap;
  }
  .input-row .input-group {
    flex: 1;
    min-width: 100px;
  }
  .input-row select {
    width: 100%;
  }
  .generate-btn {
    width: 100%;
    justify-content: center;
    padding: 8px 16px;
  }
  .generate-btn:disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }
  .error-message {
    display: flex;
    align-items: center;
    gap: 6px;
    padding: 8px 10px;
    background: var(--color-error-bg);
    border-radius: var(--radius-md);
    color: var(--color-error);
    font-size: 13px;
  }

  .preview-column {
    flex: 1;
    min-width: 240px;
    max-width: 400px;
    display: flex;
    align-items: center;
    justify-content: center;
  }
  .preview-placeholder {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    width: 100%;
    height: 300px;
    border: 2px dashed var(--color-border);
    border-radius: var(--radius-lg);
    background: var(--color-surface);
  }
  .preview-container {
    display: flex;
    flex-direction: column;
    gap: 8px;
    align-items: center;
    width: 100%;
  }
  .preview-image {
    max-width: 100%;
    max-height: 320px;
    border-radius: var(--radius-md);
    border: 1px solid var(--color-border);
    box-shadow: var(--shadow-sm);
    object-fit: contain;
  }
  .revised-prompt {
    text-align: center;
    font-style: italic;
    max-width: 100%;
  }
  .generating-indicator {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    color: var(--color-text-muted);
  }
  .spinner {
    animation: spin 1s linear infinite;
  }
  @keyframes spin {
    from { transform: rotate(0deg); }
    to { transform: rotate(360deg); }
  }

  .dialog-footer {
    display: flex;
    gap: 8px;
    padding-top: 12px;
    border-top: 1px solid var(--color-border);
  }
</style>
