<script lang="ts">
  import type { VideoInfo } from '$lib/types';
  import { formatBytes, formatDuration } from '$lib/format';
  import { t } from '$lib/i18n.svelte';

  let { video }: { video: VideoInfo } = $props();
</script>

<section class="panel video">
  <div class="thumb" aria-hidden="true">
    <svg viewBox="0 0 24 24"><path d="M8 5.5v13l10.5-6.5L8 5.5Z" /></svg>
  </div>
  <div class="info">
    <strong class="name ellipsis" title={video.path}>{video.fileName}</strong>
    <div class="meta muted small">
      <span>{formatBytes(video.sizeBytes)}</span>
      <span>{video.width}×{video.height}</span>
      <span>{formatDuration(video.durationS)}</span>
      <span>{video.codec.toUpperCase()}</span>
      {#if video.hdr}
        <span class="hdr" title={t('video.hdrHint')}>{t('video.hdr')}</span>
      {/if}
    </div>
  </div>
</section>

<style>
  .video {
    display: flex;
    align-items: center;
    gap: var(--space-3);
  }

  .thumb {
    display: grid;
    flex: none;
    place-items: center;
    width: 44px;
    height: 44px;
    border-radius: var(--radius-lg);
    background: var(--btn-primary);
    box-shadow: inset 0 1px 0 rgb(255 255 255 / 0.1);
  }

  .thumb svg {
    width: 20px;
    height: 20px;
    fill: var(--btn-primary-text);
  }

  .info {
    display: flex;
    flex-direction: column;
    gap: 2px;
    min-width: 0;
  }

  .name {
    color: var(--text-header);
  }

  .meta {
    display: flex;
    flex-wrap: wrap;
    font-variant-numeric: tabular-nums;
  }

  .hdr {
    color: var(--yellow);
    font-weight: 600;
  }

  .meta span:not(:last-child)::after {
    content: '•';
    margin: 0 6px;
    opacity: 0.6;
  }
</style>
