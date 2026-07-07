<script lang="ts">
  import { invoke } from '@tauri-apps/api/core';
  import { getEditorStore } from '../stores/editor.svelte';
  import { getConfigStore } from '../stores/config.svelte';
  import type { PublishTarget, PublishOptions, PublicationRecord, HeaderPair } from '../types';

  const editor = getEditorStore();
  const config = getConfigStore();

  let {
    onClose = () => {},
  }: {
    onClose?: () => void;
  } = $props();

  // --- State ---
  let selectedTargetId = $state('');
  let publishOptions = $state<PublishOptions>({
    uploadImages: true,
    convertWikilinks: true,
    stripPrivate: false,
    generateRss: false,
    customEndpoint: '',
    customHeaders: [],
  });
  let preview = $state('');
  let rssPreview = $state('');
  let isPublishing = $state(false);
  let isPreviewLoading = $state(false);
  let isRssLoading = $state(false);
  let error = $state<string | null>(null);
  let success = $state<string | null>(null);
  let publicationHistory = $state<PublicationRecord[]>([]);
  let showHistory = $state(false);
  let isHistoryLoading = $state(false);
  let showCustomEndpoint = $state(false);
  let showCustomHeaders = $state(false);
  let newHeaderKey = $state('');
  let newHeaderValue = $state('');

  // --- Image hosting config (inline for quick access) ---
  let imgHostType = $state<'github' | 'cloudflare'>('github');
  let imgRepo = $state('');
  let imgBranch = $state('main');
  let imgToken = $state('');
  let imgAccountId = $state('');
  let imgApiToken = $state('');
  let showImageHostConfig = $state(false);

  let selectedTarget = $derived(
    config.publishTargets.find(t => t.id === selectedTargetId),
  );

  // Load options from selected target
  $effect(() => {
    if (selectedTarget) {
      publishOptions = {
        uploadImages: selectedTarget.uploadImages,
        convertWikilinks: selectedTarget.convertWikilinks,
        stripPrivate: selectedTarget.stripPrivate,
        generateRss: selectedTarget.generateRss,
        customEndpoint: selectedTarget.endpoint || '',
        customHeaders: [],
      };
      showCustomEndpoint = selectedTarget.type === 'custom_api';
    }
  });

  // Load publication history on mount
  $effect(() => {
    loadHistory();
  });

  // --- Functions ---

  async function loadHistory() {
    isHistoryLoading = true;
    try {
      publicationHistory = await invoke<PublicationRecord[]>('list_publications');
    } catch (err) {
      // Silently fail — history may not be available
      publicationHistory = [];
    } finally {
      isHistoryLoading = false;
    }
  }

  function loadPreview() {
    if (!editor.activeFilePath) return;
    isPreviewLoading = true;
    error = null;

    invoke<string>('publish_preview', { filePath: editor.activeFilePath })
      .then((p) => { preview = p; })
      .catch((err) => { error = String(err); })
      .finally(() => { isPreviewLoading = false; });
  }

  async function generateRssFeed() {
    if (!selectedTarget) return;
    isRssLoading = true;
    error = null;
    rssPreview = '';

    try {
      const xml = await invoke<string>('generate_rss', { targetId: selectedTarget.id });
      rssPreview = xml;
    } catch (err) {
      error = String(err);
    } finally {
      isRssLoading = false;
    }
  }

  async function handlePublish() {
    if (!selectedTarget || !editor.activeFilePath) return;

    isPublishing = true;
    error = null;
    success = null;

    // Build target with inline custom endpoint/headers override
    const effectiveTarget = { ...selectedTarget };
    if (showCustomEndpoint && publishOptions.customEndpoint) {
      effectiveTarget.endpoint = publishOptions.customEndpoint;
    }

    const effectiveOptions = { ...publishOptions };
    // Remove frontend-only fields before sending to Rust
    if (!effectiveOptions.customEndpoint) {
      delete (effectiveOptions as any).customEndpoint;
    }
    if (!effectiveOptions.customHeaders || effectiveOptions.customHeaders.length === 0) {
      delete (effectiveOptions as any).customHeaders;
    }

    try {
      const resultStr = await invoke<string>('publish_now', {
        target: effectiveTarget,
        filePath: editor.activeFilePath,
        options: effectiveOptions,
      });
      const result: PublicationRecord = JSON.parse(resultStr);
      success = `Published to ${selectedTarget.name}${result.url ? ` — ${result.url}` : ''}`;
      // Refresh history
      loadHistory();
    } catch (err) {
      error = String(err);
    } finally {
      isPublishing = false;
    }
  }

  async function handleUploadImage() {
    if (!editor.activeFilePath || !selectedTarget) return;
    error = null;
    success = null;

    try {
      const url = await invoke<string>('upload_image', {
        filePath: editor.activeFilePath,
        targetId: selectedTarget.id,
      });
      success = `Image uploaded: ${url}`;
    } catch (err) {
      error = String(err);
    }
  }

  function addHeader() {
    if (!newHeaderKey.trim()) return;
    if (!publishOptions.customHeaders) {
      publishOptions.customHeaders = [];
    }
    publishOptions.customHeaders = [
      ...publishOptions.customHeaders,
      { key: newHeaderKey.trim(), value: newHeaderValue.trim() },
    ];
    newHeaderKey = '';
    newHeaderValue = '';
  }

  function removeHeader(index: number) {
    if (!publishOptions.customHeaders) return;
    publishOptions.customHeaders = publishOptions.customHeaders.filter((_, i) => i !== index);
  }

  function copyToClipboard(text: string) {
    navigator.clipboard.writeText(text).catch(() => {});
  }

  function formatDate(iso: string): string {
    try {
      return new Date(iso).toLocaleString();
    } catch {
      return iso;
    }
  }
</script>

<div class="publish-panel">
  <h3 class="panel-title">Publish</h3>

  <!-- File Info -->
  <div class="file-info">
    <span class="text-sm text-muted">File:</span>
    <span class="text-sm truncate">{editor.activeFilePath || 'No file open'}</span>
  </div>

  <!-- Target Selection -->
  <div class="input-group">
    <label for="target-select">Target</label>
    <select id="target-select" bind:value={selectedTargetId}>
      <option value="">Select a publish target...</option>
      {#each config.publishTargets as target}
        <option value={target.id}>{target.name} ({target.type})</option>
      {/each}
    </select>
  </div>

  {#if selectedTarget}
    <!-- Options -->
    <div class="options-section">
      <h4 class="options-title text-sm font-bold">Options</h4>
      <label class="option-row">
        <input type="checkbox" bind:checked={publishOptions.uploadImages} />
        <span>Upload images</span>
      </label>
      <label class="option-row">
        <input type="checkbox" bind:checked={publishOptions.convertWikilinks} />
        <span>Convert [[wikilinks]]</span>
      </label>
      <label class="option-row">
        <input type="checkbox" bind:checked={publishOptions.stripPrivate} />
        <span>Strip private tags</span>
      </label>
      <label class="option-row">
        <input type="checkbox" bind:checked={publishOptions.generateRss} />
        <span>Generate RSS feed</span>
      </label>
    </div>

    <!-- Custom API Configuration -->
    {#if selectedTarget.type === 'custom_api' || showCustomEndpoint}
      <div class="section-card">
        <button
          class="section-toggle"
          onclick={() => { showCustomEndpoint = !showCustomEndpoint; }}
        >
          <span class="text-sm font-bold">Custom API Config</span>
          <span class="toggle-icon">{showCustomEndpoint ? '▾' : '▸'}</span>
        </button>
        {#if showCustomEndpoint}
          <div class="section-body">
            <div class="input-group">
              <label>Endpoint URL</label>
              <input
                type="text"
                bind:value={publishOptions.customEndpoint}
                placeholder="https://api.example.com/publish"
              />
            </div>
            <div class="input-group">
              <button
                class="btn btn-ghost btn-sm"
                onclick={() => { showCustomHeaders = !showCustomHeaders; }}
              >
                {showCustomHeaders ? 'Hide Headers' : 'Custom Headers'}
              </button>
            </div>
            {#if showCustomHeaders}
              <div class="headers-section">
                {#each publishOptions.customHeaders ?? [] as header, idx}
                  <div class="header-row">
                    <input
                      type="text"
                      bind:value={header.key}
                      placeholder="Key"
                      class="header-key"
                    />
                    <input
                      type="text"
                      bind:value={header.value}
                      placeholder="Value"
                      class="header-value"
                    />
                    <button class="btn btn-ghost btn-sm danger" onclick={() => removeHeader(idx)}>
                      <svg width="10" height="10" viewBox="0 0 16 16" fill="none">
                        <path d="M4 4l8 8M12 4l-8 8" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
                      </svg>
                    </button>
                  </div>
                {/each}
                <div class="header-row">
                  <input
                    type="text"
                    bind:value={newHeaderKey}
                    placeholder="Key"
                    class="header-key"
                  />
                  <input
                    type="text"
                    bind:value={newHeaderValue}
                    placeholder="Value"
                    class="header-value"
                  />
                  <button class="btn btn-primary btn-sm" onclick={addHeader}>+</button>
                </div>
              </div>
            {/if}
          </div>
        {/if}
      </div>
    {/if}

    <!-- Image Hosting Configuration -->
    <div class="section-card">
      <button
        class="section-toggle"
        onclick={() => { showImageHostConfig = !showImageHostConfig; }}
      >
        <span class="text-sm font-bold">Image Hosting</span>
        <span class="toggle-icon">{showImageHostConfig ? '▾' : '▸'}</span>
      </button>
      {#if showImageHostConfig}
        <div class="section-body">
          <div class="input-group">
            <label>Host Type</label>
            <select bind:value={imgHostType}>
              <option value="github">GitHub</option>
              <option value="cloudflare">Cloudflare Images</option>
            </select>
          </div>
          {#if imgHostType === 'github'}
            <div class="input-group">
              <label>Repository (owner/repo)</label>
              <input type="text" bind:value={imgRepo} placeholder="username/repo" />
            </div>
            <div class="input-group">
              <label>Branch</label>
              <input type="text" bind:value={imgBranch} placeholder="main" />
            </div>
            <div class="input-group">
              <label>Token</label>
              <input type="password" bind:value={imgToken} placeholder="ghp_..." />
            </div>
          {:else}
            <div class="input-group">
              <label>Account ID</label>
              <input type="text" bind:value={imgAccountId} placeholder="Cloudflare Account ID" />
            </div>
            <div class="input-group">
              <label>API Token</label>
              <input type="password" bind:value={imgApiToken} placeholder="Cloudflare API Token" />
            </div>
          {/if}
          <button
            class="btn btn-secondary btn-sm"
            onclick={handleUploadImage}
            disabled={!editor.activeFilePath}
          >
            Upload Image
          </button>
        </div>
      {/if}
    </div>
  {/if}

  <!-- Preview -->
  <div class="preview-section">
    <button class="btn btn-secondary btn-sm" onclick={loadPreview} disabled={!editor.activeFilePath}>
      {isPreviewLoading ? 'Loading...' : 'Preview Changes'}
    </button>
    {#if preview}
      <div class="preview-content">
        <pre class="preview-text">{preview}</pre>
      </div>
    {/if}
  </div>

  <!-- RSS Feed Section -->
  {#if selectedTarget && (selectedTarget.type === 'rss' || publishOptions.generateRss)}
    <div class="section-card">
      <h4 class="text-sm font-bold">RSS Feed</h4>
      <button
        class="btn btn-secondary btn-sm"
        onclick={generateRssFeed}
        disabled={isRssLoading}
      >
        {isRssLoading ? 'Generating...' : 'Generate RSS Feed'}
      </button>
      {#if rssPreview}
        <div class="rss-preview">
          <div class="rss-toolbar">
            <span class="text-xs text-muted">RSS 2.0 XML</span>
            <button class="btn btn-ghost btn-sm" onclick={() => copyToClipboard(rssPreview)}>
              Copy
            </button>
          </div>
          <pre class="rss-xml">{rssPreview}</pre>
        </div>
      {/if}
    </div>
  {/if}

  <!-- Publication History -->
  <div class="section-card">
    <button
      class="section-toggle"
      onclick={() => { showHistory = !showHistory; }}
    >
      <span class="text-sm font-bold">Publication History ({publicationHistory.length})</span>
      <span class="toggle-icon">{showHistory ? '▾' : '▸'}</span>
    </button>
    {#if showHistory}
      <div class="section-body">
        {#if isHistoryLoading}
          <span class="text-sm text-muted">Loading...</span>
        {:else if publicationHistory.length === 0}
          <span class="text-sm text-muted">No publications yet</span>
        {:else}
          <div class="history-list">
            {#each publicationHistory as entry}
              <div class="history-item">
                <div class="history-item-main">
                  <span class="text-sm font-bold">{entry.targetName}</span>
                  <span class="text-xs text-muted">{formatDate(entry.publishedAt)}</span>
                </div>
                <div class="history-item-detail">
                  <span class="text-xs truncate">{entry.filePath}</span>
                  <span
                    class="status-badge"
                    class:success={entry.status === 'success'}
                    class:error={entry.status !== 'success'}
                  >
                    {entry.status}
                  </span>
                </div>
                {#if entry.url}
                  <a class="text-xs history-url" href={entry.url} target="_blank" rel="noopener">
                    {entry.url}
                  </a>
                {/if}
              </div>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  </div>

  {#if error}
    <div class="message error-message">
      <span class="text-sm" style="color: var(--color-error)">{error}</span>
    </div>
  {/if}

  {#if success}
    <div class="message success-message">
      <span class="text-sm" style="color: var(--color-success)">{success}</span>
    </div>
  {/if}

  <!-- Actions -->
  <div class="actions">
    <button class="btn btn-secondary" onclick={onClose}>Cancel</button>
    <button
      class="btn btn-primary"
      onclick={handlePublish}
      disabled={!selectedTarget || !editor.activeFilePath || isPublishing}
    >
      {isPublishing ? 'Publishing...' : 'Publish Now'}
    </button>
  </div>
</div>

<style>
  .publish-panel {
    display: flex;
    flex-direction: column;
    gap: 10px;
    font-size: 13px;
  }
  .panel-title {
    font-size: 16px;
    font-weight: 600;
  }
  .file-info {
    display: flex;
    gap: 8px;
    align-items: center;
    padding: 8px;
    background: var(--color-surface);
    border-radius: var(--radius-md);
  }
  .options-section {
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding: 8px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
  }
  .options-title {
    margin-bottom: 2px;
  }
  .option-row {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    cursor: pointer;
  }
  .option-row input[type="checkbox"] {
    width: 14px;
    height: 14px;
  }
  .section-card {
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
    overflow: hidden;
  }
  .section-toggle {
    display: flex;
    align-items: center;
    justify-content: space-between;
    width: 100%;
    padding: 8px 10px;
    background: var(--color-surface);
    transition: background var(--transition-fast);
  }
  .section-toggle:hover {
    background: color-mix(in srgb, var(--color-accent) 8%, transparent);
  }
  .toggle-icon {
    font-size: 10px;
    color: var(--color-muted);
  }
  .section-body {
    padding: 8px 10px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    border-top: 1px solid var(--color-border);
  }
  .headers-section {
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .header-row {
    display: flex;
    gap: 4px;
    align-items: center;
  }
  .header-key {
    flex: 2;
    padding: 4px 6px;
    font-size: 12px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
  }
  .header-value {
    flex: 3;
    padding: 4px 6px;
    font-size: 12px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
  }
  .preview-section {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .preview-content {
    max-height: 200px;
    overflow-y: auto;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
  }
  .preview-text {
    font-size: 12px;
    padding: 8px;
    white-space: pre-wrap;
    word-break: break-all;
  }
  .rss-preview {
    max-height: 250px;
    overflow-y: auto;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-md);
  }
  .rss-toolbar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 4px 8px;
    background: var(--color-surface);
    border-bottom: 1px solid var(--color-border);
    position: sticky;
    top: 0;
  }
  .rss-xml {
    font-size: 11px;
    padding: 8px;
    white-space: pre-wrap;
    word-break: break-all;
    font-family: 'JetBrains Mono', monospace;
    line-height: 1.4;
  }
  .history-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    max-height: 200px;
    overflow-y: auto;
  }
  .history-item {
    padding: 6px 8px;
    border: 1px solid var(--color-border);
    border-radius: var(--radius-sm);
  }
  .history-item-main {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }
  .history-item-detail {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-top: 2px;
  }
  .history-url {
    display: block;
    margin-top: 2px;
    color: var(--color-accent);
    overflow: hidden;
    text-overflow: ellipsis;
  }
  .status-badge {
    font-size: 10px;
    padding: 1px 6px;
    border-radius: 99px;
    font-weight: 500;
  }
  .status-badge.success {
    background: color-mix(in srgb, var(--color-success) 15%, transparent);
    color: var(--color-success);
  }
  .status-badge.error {
    background: color-mix(in srgb, var(--color-error) 15%, transparent);
    color: var(--color-error);
  }
  .message {
    padding: 6px 10px;
    border-radius: var(--radius-md);
  }
  .error-message {
    background: var(--color-error-bg);
  }
  .success-message {
    background: var(--color-success-bg);
  }
  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 8px;
    padding-top: 8px;
    border-top: 1px solid var(--color-border);
  }
  .danger {
    color: var(--color-error);
  }
</style>
