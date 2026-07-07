<script lang="ts">
  import { getI18n } from '../i18n';

  const i18n = getI18n();
</script>

<!-- svelte-ignore a11y_no_static_element_interactions -->
<a
  href="#main-content"
  class="skip-link"
  onclick={(e) => {
    e.preventDefault();
    const target = document.getElementById('main-content');
    if (target) {
      target.focus();
      target.scrollIntoView({ behavior: 'smooth' });
    }
  }}
>
  {i18n.t('a11y.skip')}
</a>
