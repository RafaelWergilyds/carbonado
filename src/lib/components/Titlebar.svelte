<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { app } from '$lib/stores/app.svelte';
  import { i18n, t, type Locale } from '$lib/i18n.svelte';
  import Logo from './Logo.svelte';

  const win = getCurrentWindow();
  let maximized = $state(false);

  let engineOk = $derived(app.encoders.h264Nvenc || app.encoders.hevcNvenc);
  let engineLabel = $derived(
    t(!app.encodersChecked ? 'titlebar.checkingGpu' : engineOk ? 'titlebar.gpu' : 'titlebar.cpu'),
  );
  const locales: { id: Locale; name: string }[] = [
    { id: 'pt', name: 'Português' },
    { id: 'en', name: 'English' },
  ];
  let langOpen = $state(false);
  let langEl: HTMLDivElement | undefined = $state();

  function pick(locale: Locale) {
    i18n.set(locale);
    langOpen = false;
  }

  function onWindowPointer(event: PointerEvent) {
    if (langOpen && langEl && !langEl.contains(event.target as Node)) langOpen = false;
  }

  function onWindowKey(event: KeyboardEvent) {
    if (langOpen && event.key === 'Escape') langOpen = false;
  }

  onMount(() => {
    const sync = () => win.isMaximized().then((m) => (maximized = m));
    sync();
    const unlisten = win.onResized(sync);
    return () => {
      unlisten.then((u) => u());
    };
  });
</script>

<svelte:window onpointerdown={onWindowPointer} onkeydown={onWindowKey} />

<header class="titlebar" data-tauri-drag-region>
  <div class="brand" data-tauri-drag-region>
    <Logo size={20} />
    <span class="title" data-tauri-drag-region>Carbonado</span>
  </div>

  <span
    class="chip"
    class:ok={app.encodersChecked && engineOk}
    class:off={app.encodersChecked && !engineOk}
    title={app.encodersChecked && !engineOk ? t('titlebar.gpuUnavailable') : undefined}
    data-tauri-drag-region
  >
    <span class="dot" data-tauri-drag-region></span>{engineLabel}
  </span>

  <div class="lang" bind:this={langEl}>
    <button
      class="lang-trigger"
      class:open={langOpen}
      aria-label={t('titlebar.language')}
      aria-haspopup="listbox"
      aria-expanded={langOpen}
      onclick={() => (langOpen = !langOpen)}
    >
      {i18n.locale.toUpperCase()}
      <svg viewBox="0 0 10 10" aria-hidden="true"><path d="M2 3.5l3 3 3-3" /></svg>
    </button>
    {#if langOpen}
      <ul class="lang-menu" role="listbox" aria-label={t('titlebar.language')}>
        {#each locales as locale (locale.id)}
          <li>
            <button
              role="option"
              aria-selected={i18n.locale === locale.id}
              class:active={i18n.locale === locale.id}
              onclick={() => pick(locale.id)}
            >
              <span class="code">{locale.id.toUpperCase()}</span>{locale.name}
            </button>
          </li>
        {/each}
      </ul>
    {/if}
  </div>

  <div class="controls">
    <button aria-label={t('titlebar.minimize')} onclick={() => win.minimize()}>
      <svg viewBox="0 0 12 12" aria-hidden="true"><rect x="1" y="5.5" width="10" height="1" /></svg>
    </button>
    <button aria-label={t(maximized ? 'titlebar.restore' : 'titlebar.maximize')} onclick={() => win.toggleMaximize()}>
      {#if maximized}
        <svg viewBox="0 0 12 12" aria-hidden="true" class="stroke">
          <rect x="1.5" y="3.5" width="7" height="7" />
          <path d="M3.5 3.5v-2h7v7h-2" />
        </svg>
      {:else}
        <svg viewBox="0 0 12 12" aria-hidden="true" class="stroke"><rect x="1.5" y="1.5" width="9" height="9" /></svg>
      {/if}
    </button>
    <button class="close" aria-label={t('titlebar.close')} onclick={() => win.close()}>
      <svg viewBox="0 0 12 12" aria-hidden="true" class="stroke"><path d="M1.5 1.5l9 9m0-9l-9 9" /></svg>
    </button>
  </div>
</header>

<style>
  .titlebar {
    position: relative;
    z-index: 1;
    display: flex;
    align-items: center;
    gap: var(--space-3);
    height: 40px;
    padding-left: var(--space-4);
    background: var(--bg-tertiary);
  }

  .brand {
    display: flex;
    flex: 1;
    align-items: center;
    gap: var(--space-2);
    height: 100%;
  }

  .title {
    color: var(--text-header);
    font-size: 0.875rem;
    font-weight: 600;
  }

  .chip {
    display: inline-flex;
    align-items: center;
    gap: 6px;
    padding: 2px 8px;
    border-radius: 999px;
    background: var(--bg-secondary);
    color: var(--text-muted);
    font-size: 0.75rem;
    font-weight: 600;
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    background: var(--switch-off);
  }

  .chip.ok .dot {
    background: var(--green);
    box-shadow: 0 0 6px var(--green);
  }

  .chip.off .dot {
    background: var(--red);
    box-shadow: 0 0 6px var(--red);
  }

  .lang {
    position: relative;
  }

  .lang-trigger {
    display: inline-flex;
    align-items: center;
    gap: 3px;
    padding: 3px 6px;
    border: none;
    border-radius: 4px;
    background: transparent;
    color: var(--text-muted);
    font-size: 0.6875rem;
    font-weight: 600;
    letter-spacing: 0.04em;
    transition: background 0.15s, color 0.15s;
  }

  .lang-trigger:hover,
  .lang-trigger.open {
    background: var(--bg-hover);
    color: var(--text-header);
  }

  .lang-trigger svg {
    width: 9px;
    height: 9px;
    fill: none;
    stroke: currentColor;
    stroke-width: 1.4;
    transition: transform 0.15s;
  }

  .lang-trigger.open svg {
    transform: rotate(180deg);
  }

  .lang-menu {
    position: absolute;
    top: calc(100% + 6px);
    right: 0;
    min-width: 140px;
    margin: 0;
    padding: 4px;
    list-style: none;
    border: 1px solid var(--bg-hover);
    border-radius: 6px;
    background: var(--bg-tertiary);
    box-shadow: 0 8px 20px rgb(0 0 0 / 0.35);
    animation: drop 0.12s ease-out;
  }

  .lang-menu button {
    display: flex;
    align-items: center;
    gap: var(--space-2);
    width: 100%;
    padding: 6px 8px;
    border: none;
    border-radius: 4px;
    background: transparent;
    color: var(--text-normal);
    font-size: 0.8125rem;
    text-align: left;
  }

  .lang-menu button:hover {
    background: var(--bg-hover);
    color: var(--text-header);
  }

  .lang-menu button.active {
    color: var(--text-header);
    font-weight: 600;
  }

  .lang-menu .code {
    width: 20px;
    color: var(--text-muted);
    font-size: 0.6875rem;
    font-weight: 600;
    letter-spacing: 0.04em;
  }

  @keyframes drop {
    from {
      opacity: 0;
      transform: translateY(-4px);
    }
  }

  .controls {
    display: flex;
    height: 100%;
  }

  .controls button {
    display: grid;
    place-items: center;
    width: 46px;
    height: 100%;
    border: none;
    background: transparent;
    color: var(--text-muted);
    transition: background 0.1s, color 0.1s;
  }

  .controls button:hover {
    background: var(--bg-hover);
    color: var(--text-header);
  }

  .controls button.close:hover {
    background: #da373c;
    color: #fff;
  }

  .controls svg {
    width: 10px;
    height: 10px;
    fill: currentColor;
  }

  .controls svg.stroke {
    fill: none;
    stroke: currentColor;
    stroke-width: 1;
  }
</style>
