<script lang="ts">
  import { getI18n } from './index';

  const i18n = getI18n();

  let {
    variant: _variant = 'default' as 'default' | 'compact',
  } = $props();

  const locales = [
    { value: 'en' as const, label: 'English' },
    { value: 'bn' as const, label: 'বাংলা' },
    { value: 'ar' as const, label: 'العربية' },
  ];
</script>

<div class="locale-switcher" class:compact={_variant === 'compact'}>
  <label for="locale-select" class="sr-only">Language</label>
  <select
    id="locale-select"
    value={i18n.locale}
    onchange={(e) => {
      i18n.setLocale((e.target as HTMLSelectElement).value as 'en' | 'bn' | 'ar');
    }}
    class="locale-select"
  >
    {#each locales as loc}
      <option value={loc.value}>{loc.label}</option>
    {/each}
  </select>
</div>

<style>
  .locale-switcher {
    display: inline-flex;
    align-items: center;
  }
  .locale-select {
    padding: 4px 8px;
    font-size: 12px;
    border-radius: var(--radius-sm);
    border: 1px solid var(--color-border);
    background: var(--color-bg);
    color: var(--color-text);
    cursor: pointer;
    outline: none;
    transition: border-color var(--transition-fast);
  }
  .locale-select:focus-visible {
    outline: 2px solid var(--color-accent);
    outline-offset: 2px;
    border-color: transparent;
  }
  .compact .locale-select {
    padding: 2px 6px;
    font-size: 11px;
  }
</style>
