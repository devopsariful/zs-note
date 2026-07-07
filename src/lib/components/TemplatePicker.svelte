<script lang="ts">
  import Modal from './Modal.svelte';
  import { getAIStore } from '../stores/ai.svelte';

  const ai = getAIStore();

  let {
    show = false,
    onClose = () => {},
    onSelect = (_prompt: string) => {},
  }: {
    show?: boolean;
    onClose?: () => void;
    onSelect?: (prompt: string) => void;
  } = $props();

  let searchTerm = $state('');
  let selectedTemplate = $state<(typeof ai.templates)[0] | null>(null);
  let variableValues = $state<Record<string, string>>({});

  let filteredTemplates = $derived(
    ai.templates.filter(t =>
      t.name.toLowerCase().includes(searchTerm.toLowerCase()) ||
      t.description.toLowerCase().includes(searchTerm.toLowerCase()),
    ),
  );

  function selectTemplate(t: (typeof ai.templates)[0]) {
    selectedTemplate = t;
    variableValues = {};
    for (const v of t.variables) {
      variableValues[v.name] = v.defaultValue || '';
    }
  }

  function handleGenerate() {
    if (!selectedTemplate) return;

    let prompt = selectedTemplate.prompt;
    for (const [key, value] of Object.entries(variableValues)) {
      prompt = prompt.replaceAll(`{{${key}}}`, value || '');
    }

    onSelect(prompt);
    selectedTemplate = null;
    searchTerm = '';
  }

  function goBack() {
    selectedTemplate = null;
  }
</script>

<Modal
  title="AI Templates"
  bind:show
  size="lg"
  {onClose}
>
  {#if !selectedTemplate}
    <div class="template-picker">
      <input
        type="text"
        class="search-input"
        placeholder="Search templates..."
        bind:value={searchTerm}
      />
      <div class="template-list">
        {#if filteredTemplates.length === 0}
          <div class="empty-state">
            <p class="text-muted text-sm">No templates found</p>
          </div>
        {:else}
          {#each filteredTemplates as tpl}
            <button class="template-card" onclick={() => selectTemplate(tpl)}>
              <div class="template-name">{tpl.name}</div>
              <div class="template-desc text-muted text-sm">{tpl.description}</div>
              {#if tpl.category}
                <span class="badge badge-category">{tpl.category}</span>
              {/if}
            </button>
          {/each}
        {/if}
      </div>
    </div>
  {:else}
    <div class="template-variables">
      <button class="btn btn-ghost btn-sm" onclick={goBack}>
        <svg width="12" height="12" viewBox="0 0 16 16" fill="none">
          <path d="M10 4l-4 4 4 4" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" />
        </svg>
        Back
      </button>
      <h3 class="template-name">{selectedTemplate.name}</h3>
      <p class="text-muted text-sm">{selectedTemplate.description}</p>
      <div class="variables-form">
        {#each selectedTemplate.variables as varDef}
          <div class="input-group">
            <label for={`var-${varDef.name}`}>
              {varDef.label || varDef.name}
              {#if varDef.required}<span style="color: var(--color-error)">*</span>{/if}
            </label>
            {#if varDef.type === 'textarea'}
              <textarea
                id={`var-${varDef.name}`}
                bind:value={variableValues[varDef.name]}
                placeholder={varDef.description}
                rows={3}
              />
            {:else if varDef.type === 'select'}
              <select
                id={`var-${varDef.name}`}
                bind:value={variableValues[varDef.name]}
              >
                <option value="">Select...</option>
                {#each varDef.options ?? [] as opt}
                  <option value={opt}>{opt}</option>
                {/each}
              </select>
            {:else if varDef.type === 'boolean'}
              <label class="checkbox-label">
                <input
                  type="checkbox"
                  checked={variableValues[varDef.name] === 'true'}
                  onchange={(e) => { variableValues[varDef.name] = (e.target as HTMLInputElement).checked ? 'true' : 'false'; }}
                />
                {varDef.description}
              </label>
            {:else}
              <input
                id={`var-${varDef.name}`}
                type="text"
                bind:value={variableValues[varDef.name]}
                placeholder={varDef.description}
              />
            {/if}
          </div>
        {/each}
      </div>
      <div class="generate-actions">
        <button class="btn btn-primary" onclick={handleGenerate}>
          Generate & Insert
        </button>
      </div>
    </div>
  {/if}
</Modal>

<style>
  .template-picker {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .search-input {
    width: 100%;
    padding: 8px 12px;
  }
  .template-list {
    display: flex;
    flex-direction: column;
    gap: 6px;
    max-height: 400px;
    overflow-y: auto;
  }
  .empty-state {
    padding: 24px;
    text-align: center;
  }
  .template-card {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 10px 12px;
    border-radius: var(--radius-md);
    border: 1px solid var(--color-border);
    text-align: left;
    transition: all var(--transition-fast);
  }
  .template-card:hover {
    border-color: var(--color-accent);
    background: color-mix(in srgb, var(--color-accent) 6%, transparent);
  }
  .template-name {
    font-weight: 600;
    font-size: 13px;
  }
  .template-desc {
    font-size: 12px;
  }
  .badge-category {
    align-self: flex-start;
    background: color-mix(in srgb, var(--color-accent) 12%, transparent);
    color: var(--color-accent);
  }
  .template-variables {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .variables-form {
    display: flex;
    flex-direction: column;
    gap: 12px;
  }
  .checkbox-label {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 13px;
    cursor: pointer;
  }
  .generate-actions {
    display: flex;
    justify-content: flex-end;
    padding-top: 8px;
    border-top: 1px solid var(--color-border);
  }
</style>
