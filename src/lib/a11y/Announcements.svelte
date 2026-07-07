<script lang="ts">
  import { onMount } from 'svelte';
  import { initAnnouncements } from './announce';

  onMount(() => {
    initAnnouncements();
  });
</script>

<!--
  Live region for screen reader announcements.
  The DOM elements are referenced by announce() in announce.ts.
  Must be placed once at the app root.
-->
<div
  id="a11y-polite"
  class="sr-only"
  role="status"
  aria-live="polite"
  aria-atomic="true"
></div>

<div
  id="a11y-assertive"
  class="sr-only"
  role="alert"
  aria-live="assertive"
  aria-atomic="true"
></div>
