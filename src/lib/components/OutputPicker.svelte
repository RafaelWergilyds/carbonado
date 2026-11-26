<script lang="ts">
  import { splitPath } from '$lib/api';
  import { t } from '$lib/i18n.svelte';

  let { path, onchoose }: { path: string | null; onchoose: () => void } = $props();

  let parts = $derived(path ? splitPath(path) : null);
</script>

<section class="panel">
  <span class="eyebrow">{t('output.title')}</span>
  <div class="field-box">
    <svg class="folder" viewBox="0 0 24 24" aria-hidden="true">
      <path d="M3 6.5A2.5 2.5 0 0 1 5.5 4h4l2 2h7A2.5 2.5 0 0 1 21 8.5v9a2.5 2.5 0 0 1-2.5 2.5h-13A2.5 2.5 0 0 1 3 17.5v-11Z" />
    </svg>
    <div class="path" title={path ?? ''}>
      {#if parts}
        <span class="name ellipsis">{parts.name}</span>
        <span class="dir ellipsis muted small">{parts.dir}</span>
      {:else}
        <span class="muted">{t('output.none')}</span>
      {/if}
    </div>
    <button class="btn btn-sm" onclick={onchoose}>{t('output.change')}</button>
  </div>
</section>

<style>
  .folder {
    flex: none;
    width: 20px;
    height: 20px;
    fill: var(--text-muted);
  }

  .path {
    display: flex;
    flex: 1;
    flex-direction: column;
    min-width: 0;
  }

  .name {
    color: var(--text-header);
    font-weight: 500;
  }
</style>
