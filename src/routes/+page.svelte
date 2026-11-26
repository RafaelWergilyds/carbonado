<script lang="ts">
  import { onMount } from 'svelte';
  import { getCurrentWebview } from '@tauri-apps/api/webview';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { ask } from '@tauri-apps/plugin-dialog';
  import { app } from '$lib/stores/app.svelte';
  import { t, errorMessage } from '$lib/i18n.svelte';
  import Titlebar from '$lib/components/Titlebar.svelte';
  import DropZone from '$lib/components/DropZone.svelte';
  import VideoCard from '$lib/components/VideoCard.svelte';
  import Settings from '$lib/components/Settings.svelte';
  import OutputPicker from '$lib/components/OutputPicker.svelte';
  import Progress from '$lib/components/Progress.svelte';
  import Result from '$lib/components/Result.svelte';

  let dragging = $state(false);

  onMount(() => {
    const cleanups: (() => void)[] = [];

    app.init().then((u) => cleanups.push(u));

    getCurrentWebview()
      .onDragDropEvent((event) => {
        const p = event.payload;
        if (app.phase === 'compressing') return;
        if (p.type === 'enter' || p.type === 'over') dragging = true;
        else if (p.type === 'leave') dragging = false;
        else if (p.type === 'drop') {
          dragging = false;
          if (p.paths.length > 0) app.load(p.paths[0]);
        }
      })
      .then((u) => cleanups.push(u));

    getCurrentWindow()
      .onCloseRequested(async (event) => {
        if (app.phase !== 'compressing') return;
        const confirmed = await ask(t('close.message'), {
          title: t('close.title'),
          kind: 'warning',
          okLabel: t('close.confirm'),
          cancelLabel: t('close.cancel'),
        });
        if (!confirmed) event.preventDefault();
      })
      .then((u) => cleanups.push(u));

    return () => cleanups.forEach((c) => c());
  });
</script>

<div class="shell">
  <Titlebar />

  <main class="content" class:dragging={dragging && app.phase !== 'idle'}>
    {#if app.phase === 'idle' || app.phase === 'probing'}
      <DropZone {dragging} busy={app.phase === 'probing'} onfile={(p) => app.load(p)} />
    {:else if app.phase === 'error'}
      <section class="panel stack">
        <span class="eyebrow">{t('error.title')}</span>
        <div class="callout callout-error">
          <strong>{errorMessage(app.error)}</strong>
        </div>
        {#if app.error?.detail}
          <details>
            <summary class="muted small">{t('error.details')}</summary>
            <pre>{app.error.detail}</pre>
          </details>
        {/if}
      </section>
    {:else if app.video}
      <VideoCard video={app.video} />
      {#if app.phase === 'loaded'}
        <Settings />
        <OutputPicker path={app.output} onchoose={() => app.chooseOutput()} />
      {:else if app.phase === 'compressing'}
        <Progress progress={app.progress} notice={app.notice} output={app.output} />
      {:else if app.phase === 'done' && app.result}
        <Result result={app.result} />
      {/if}
    {/if}
  </main>

  {#if app.phase !== 'idle' && app.phase !== 'probing'}
    <footer class="actionbar">
      {#if app.phase === 'loaded'}
        <button class="btn btn-link" onclick={() => app.reset()}>{t('actions.changeVideo')}</button>
        <button class="btn btn-primary btn-lg" onclick={() => app.start()}>{t('actions.compress')}</button>
      {:else if app.phase === 'compressing'}
        <span class="muted small">{t('actions.keepWorking')}</span>
        <button class="btn btn-danger" onclick={() => app.cancel()}>{t('actions.cancel')}</button>
      {:else if app.phase === 'done'}
        <button class="btn btn-link" onclick={() => app.adjust()}>{t('actions.adjust')}</button>
        <button class="btn btn-primary" onclick={() => app.reset()}>{t('actions.another')}</button>
      {:else if app.phase === 'error'}
        <button class="btn btn-link" onclick={() => app.reset()}>{t('actions.chooseAnother')}</button>
        {#if app.video}
          <button class="btn btn-primary" onclick={() => app.adjust()}>{t('actions.retry')}</button>
        {/if}
      {/if}
    </footer>
  {/if}
</div>

<style>
  .shell {
    display: grid;
    grid-template-rows: auto 1fr auto;
    height: 100vh;
  }

  .content {
    display: flex;
    flex-direction: column;
    gap: var(--space-4);
    overflow-y: auto;
    padding: var(--space-4);
    outline: 2px dashed transparent;
    outline-offset: -8px;
    transition: outline-color 0.15s;
  }

  .content.dragging {
    outline-color: var(--brand);
  }

  .actionbar {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    padding: var(--space-3) var(--space-4);
    background: var(--bg-secondary);
  }

  .stack {
    display: flex;
    flex-direction: column;
    gap: var(--space-3);
  }

  pre {
    max-height: 180px;
    overflow: auto;
    margin: var(--space-2) 0 0;
    padding: var(--space-3);
    border-radius: var(--radius);
    background: var(--bg-tertiary);
    font-size: 0.75rem;
    white-space: pre-wrap;
    user-select: text;
  }
</style>
