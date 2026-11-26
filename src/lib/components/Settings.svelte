<script lang="ts">
  import { untrack } from 'svelte';
  import { app, QUALITY_PRESETS, resolutionAvailable } from '$lib/stores/app.svelte';
  import { formatBytes, formatPercent } from '$lib/format';
  import { t, type MessageKey } from '$lib/i18n.svelte';
  import type { Codec, Resolution } from '$lib/types';

  const codecs: { value: Codec; label: string; hint: MessageKey }[] = [
    { value: 'h264', label: 'H.264', hint: 'settings.codec.h264Hint' },
    { value: 'h265', label: 'H.265', hint: 'settings.codec.h265Hint' },
  ];
  const resolutions: { value: Resolution; label: string }[] = [
    { value: 'original', label: '' },
    { value: '1080p', label: '1080p' },
    { value: '720p', label: '720p' },
    { value: '480p', label: '480p' },
  ];

  const s = app.settings;

  let activePreset = $derived(
    t(QUALITY_PRESETS.find((p) => p.value === s.quality)?.label ?? 'settings.preset.custom'),
  );

  $effect(() => {
    const video = app.video;
    if (!video) return;
    const req = {
      input: video.path,
      quality: s.quality,
      codec: s.codec,
      resolution: s.resolution,
      useGpu: s.useGpu && app.gpuAvailable,
    };
    untrack(() => app.requestEstimate(req));
  });

  let original = $derived(app.video?.sizeBytes ?? 0);
  let est = $derived(app.estimate);
  let ratio = $derived(est && original > 0 ? est.bytes / original : null);
  let bigger = $derived(ratio != null && ratio >= 1);
</script>

<section class="panel">
  <div class="row-between">
    <span class="eyebrow">{t('settings.quality')}</span>
    <span class="value">{activePreset} · {s.quality}</span>
  </div>
  <div class="segmented">
    {#each QUALITY_PRESETS as preset}
      <button class:active={s.quality === preset.value} onclick={() => (s.quality = preset.value)}>
        {t(preset.label)}
      </button>
    {/each}
  </div>
  <input
    class="slider"
    type="range"
    min="0"
    max="100"
    step="1"
    bind:value={s.quality}
    style:--fill="{s.quality}%"
    aria-label={t('settings.quality')}
  />
  <p class="estimate small" class:stale={app.estimating} aria-live="polite">
    {#if est && ratio != null}
      {#if bigger}
        {t('settings.estimateBigger', { size: formatBytes(est.bytes) })}
      {:else}
        {t('settings.estimate', { size: formatBytes(est.bytes), pct: formatPercent((1 - ratio) * 100) })}
      {/if}
    {:else if app.estimateFailed}
      {t('settings.estimateFailed')}
    {:else}
      {t('settings.estimating')}
    {/if}
  </p>

  <hr class="divider" />

  <span class="eyebrow">{t('settings.codec')}</span>
  <div class="segmented">
    {#each codecs as c}
      <button class:active={s.codec === c.value} onclick={() => (s.codec = c.value)}>
        <span>{c.label}</span>
        <small>{t(c.hint)}</small>
      </button>
    {/each}
  </div>

  <hr class="divider" />

  <span class="eyebrow">{t('settings.resolution')}</span>
  <div class="segmented">
    {#each resolutions as r}
      {@const available = resolutionAvailable(r.value, app.video)}
      <button
        class:active={s.resolution === r.value}
        disabled={!available}
        title={available ? undefined : t('settings.resolution.noUpscale')}
        onclick={() => (s.resolution = r.value)}
      >
        {r.label || t('settings.resolution.original')}
      </button>
    {/each}
  </div>

  <hr class="divider" />

  <label class="toggle" class:disabled={!app.gpuAvailable}>
    <span class="toggle-text">
      <span class="toggle-title">{t('settings.gpu')}</span>
      <span class="muted small">
        {#if !app.encodersChecked}
          {t('settings.gpu.checking')}
        {:else if app.gpuAvailable}
          {t('settings.gpu.available')}
        {:else}
          {t('settings.gpu.unavailable')}
        {/if}
      </span>
    </span>
    <input type="checkbox" bind:checked={s.useGpu} disabled={!app.gpuAvailable} />
    <span class="switch" aria-hidden="true"></span>
  </label>
</section>

<style>
  .row-between {
    display: flex;
    align-items: baseline;
    justify-content: space-between;
  }

  .value {
    color: var(--text-normal);
    font-size: 0.8125rem;
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  .slider {
    margin: var(--space-3) 0 var(--space-1);
  }

  .estimate {
    color: var(--text-muted);
    font-style: italic;
    font-variant-numeric: tabular-nums;
    transition: opacity 0.15s;
  }

  .estimate.stale {
    opacity: 0.6;
  }

  .toggle {
    display: flex;
    align-items: center;
    gap: var(--space-4);
    cursor: pointer;
  }

  .toggle.disabled {
    cursor: not-allowed;
    opacity: 0.6;
  }

  .toggle input {
    position: absolute;
    opacity: 0;
    pointer-events: none;
  }

  .toggle-text {
    display: flex;
    flex: 1;
    flex-direction: column;
    gap: 2px;
  }

  .toggle-title {
    color: var(--text-header);
    font-weight: 500;
  }
</style>
