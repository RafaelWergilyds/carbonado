<script lang="ts">
  import { open } from '@tauri-apps/plugin-dialog';
  import { VIDEO_EXTENSIONS } from '$lib/api';
  import { t } from '$lib/i18n.svelte';
  import Logo from './Logo.svelte';

  let { dragging, busy, onfile }: { dragging: boolean; busy: boolean; onfile: (path: string) => void } =
    $props();

  async function choose() {
    const path = await open({
      multiple: false,
      directory: false,
      filters: [{ name: t('drop.filter'), extensions: VIDEO_EXTENSIONS }],
    });
    if (typeof path === 'string') onfile(path);
  }
</script>

<div class="zone" class:dragging>
  <div class="icon">
    <Logo size={76} />
  </div>
  {#if busy}
    <h2>{t('drop.reading')}</h2>
    <p class="muted">{t('drop.wait')}</p>
  {:else}
    <h2>{t(dragging ? 'drop.release' : 'drop.drag')}</h2>
    <p class="muted">{t('drop.formats')}</p>
    <button class="btn btn-primary" onclick={choose}>{t('drop.choose')}</button>
  {/if}
</div>

<style>
  .zone {
    display: flex;
    flex: 1;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: var(--space-2);
    min-height: 320px;
    padding: var(--space-6);
    border: 2px dashed var(--bg-tertiary);
    border-radius: var(--radius-lg);
    background: var(--bg-secondary);
    text-align: center;
    transition: border-color 0.15s, background 0.15s;
  }

  .zone.dragging {
    border-color: var(--brand);
    background: var(--brand-soft);
  }

  .icon {
    margin-bottom: var(--space-2);
    filter: drop-shadow(0 6px 14px rgb(0 0 0 / 0.35));
    transition: transform 0.2s ease;
  }

  .zone.dragging .icon {
    transform: translateY(-4px) scale(1.06);
  }

  h2 {
    font-size: 1.25rem;
  }

  .btn {
    margin-top: var(--space-4);
  }
</style>
