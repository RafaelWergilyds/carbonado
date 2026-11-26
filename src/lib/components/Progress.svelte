<script lang="ts">
  import type { ProgressEvent } from '$lib/types';
  import { splitPath } from '$lib/api';
  import { formatDuration, formatPercent, formatSpeed } from '$lib/format';
  import { t, type MessageKey } from '$lib/i18n.svelte';

  let {
    progress,
    notice,
    output,
  }: { progress: ProgressEvent | null; notice: MessageKey | null; output: string | null } = $props();

  let percent = $derived(progress?.percent ?? 0);
</script>

<section class="panel stack">
  <div class="row">
    <span class="eyebrow">{t('progress.title')}</span>
    <strong class="pct">{formatPercent(percent)}</strong>
  </div>
  <div class="bar" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow={Math.round(percent)}>
    <div class="fill" style:clip-path="inset(0 {100 - percent}% 0 0 round 4px)"></div>
  </div>
  <div class="row muted small">
    <span>{t('progress.speed', { speed: formatSpeed(progress?.speed ?? null) })}</span>
    <span>
      {progress?.etaS != null
        ? t('progress.remaining', { time: formatDuration(progress.etaS) })
        : t('progress.calculating')}
    </span>
  </div>
  {#if output}
    <p class="muted small ellipsis" title={output}>{t('progress.savingAs')} <strong>{splitPath(output).name}</strong></p>
  {/if}
  {#if notice}
    <div class="callout callout-warn">{t(notice)}</div>
  {/if}
</section>

<style>
  .stack {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  .row {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
    font-variant-numeric: tabular-nums;
  }

  .row .eyebrow {
    margin: 0;
  }

  .pct {
    color: var(--text-header);
    font-size: 1.25rem;
  }

  .bar {
    height: 8px;
    border-radius: 4px;
    background: var(--carbon-track);
    overflow: hidden;
  }

  .fill {
    height: 100%;
    background: var(--carbon);
    transition: clip-path 0.2s linear;
  }

  strong {
    color: var(--text-normal);
  }
</style>
