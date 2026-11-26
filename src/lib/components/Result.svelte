<script lang="ts">
  import { revealItemInDir } from '@tauri-apps/plugin-opener';
  import type { JobResult } from '$lib/types';
  import { splitPath } from '$lib/api';
  import { formatBytes, formatPercent } from '$lib/format';
  import { t } from '$lib/i18n.svelte';

  let { result }: { result: JobResult } = $props();

  let reduction = $derived(
    result.originalBytes > 0 ? (1 - result.finalBytes / result.originalBytes) * 100 : 0,
  );
  let usedGpu = $derived(result.encoder === 'nvencH264' || result.encoder === 'nvencHevc');
</script>

<section class="panel">
  {#if result.skipped}
    <span class="eyebrow">{t('result.skipped')}</span>
    <p class="detail small">
      {t('result.skippedText')}
    </p>
  {:else}
    <span class="eyebrow">{t('result.done')}</span>
    {#if result.outputPath}
      <p class="file ellipsis" title={result.outputPath}>{splitPath(result.outputPath).name}</p>
    {/if}
    <p class="detail small">
      {formatBytes(result.originalBytes)} → {formatBytes(result.finalBytes)} (−{formatPercent(reduction)}) · {usedGpu
        ? 'GPU'
        : 'CPU'}
    </p>
    {#if result.outputPath}
      <button class="btn btn-link small" onclick={() => revealItemInDir(result.outputPath!)}>{t('result.reveal')}</button>
    {/if}
  {/if}
</section>

<style>
  .file {
    color: var(--text-header);
    font-weight: 500;
  }

  .detail {
    margin-top: 2px;
    color: var(--text-muted);
    font-style: italic;
    font-variant-numeric: tabular-nums;
  }

  .btn-link {
    margin-top: var(--space-2);
  }
</style>
