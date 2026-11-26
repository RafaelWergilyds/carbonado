import { save } from '@tauri-apps/plugin-dialog';
import * as api from '../api';
import { t, type MessageKey } from '../i18n.svelte';
import type {
  AppError,
  Codec,
  EncoderSupport,
  EstimateRequest,
  JobResult,
  ProgressEvent,
  Resolution,
  SizeEstimate,
  VideoInfo,
} from '../types';

const LAST_DIR_KEY = 'carbonado.lastOutputDir';

function readLastDir(): string | null {
  try {
    return localStorage.getItem(LAST_DIR_KEY);
  } catch {
    return null;
  }
}

function writeLastDir(dir: string) {
  try {
    localStorage.setItem(LAST_DIR_KEY, dir);
  } catch {
  }
}

export type Phase = 'idle' | 'probing' | 'loaded' | 'compressing' | 'done' | 'error';

export const QUALITY_PRESETS = [
  { label: 'settings.preset.low', value: 35 },
  { label: 'settings.preset.balanced', value: 60 },
  { label: 'settings.preset.high', value: 85 },
] as const;

const RESOLUTION_HEIGHT: Record<Exclude<Resolution, 'original'>, number> = {
  '1080p': 1080,
  '720p': 720,
  '480p': 480,
};

export function resolutionAvailable(r: Resolution, video: VideoInfo | null): boolean {
  if (r === 'original' || !video) return true;
  return RESOLUTION_HEIGHT[r] < Math.min(video.width, video.height);
}

class AppStore {
  phase = $state<Phase>('idle');
  video = $state<VideoInfo | null>(null);
  encoders = $state<EncoderSupport>({ h264Nvenc: false, hevcNvenc: false });
  encodersChecked = $state(false);
  settings = $state({
    quality: 60,
    codec: 'h264' as Codec,
    resolution: 'original' as Resolution,
    useGpu: true,
  });
  progress = $state<ProgressEvent | null>(null);
  result = $state<JobResult | null>(null);
  error = $state<AppError | null>(null);
  notice = $state<MessageKey | null>(null);
  output = $state<string | null>(null);

  estimate = $state<SizeEstimate | null>(null);
  estimating = $state(false);
  estimateFailed = $state(false);
  private estimateCache = new Map<string, SizeEstimate>();
  private estimateTimer: ReturnType<typeof setTimeout> | undefined;
  private estimateSeq = 0;

  gpuAvailable = $derived(
    this.settings.codec === 'h264' ? this.encoders.h264Nvenc : this.encoders.hevcNvenc,
  );

  async init(): Promise<() => void> {
    const unlisten = await api.subscribeCompress({
      onProgress: (p) => (this.progress = p),
      onFallback: () => {
        this.notice = 'progress.gpuFallback';
        this.progress = { percent: 0, speed: null, etaS: null };
      },
      onDone: (r) => {
        this.result = r;
        this.phase = 'done';
      },
      onError: (e) => this.fail(e),
      onCancelled: () => {
        this.progress = null;
        this.phase = this.video ? 'loaded' : 'idle';
      },
    });

    api
      .detectEncoders()
      .then((e) => (this.encoders = e))
      .catch(() => {})
      .finally(() => (this.encodersChecked = true));

    return unlisten;
  }

  async load(path: string) {
    if (this.phase === 'compressing') return;
    if (!api.isVideoPath(path)) {
      this.fail({
        kind: 'invalidInput',
        code: 'unsupportedFormat',
        message: 'Unsupported file format',
        detail: path,
      });
      return;
    }
    this.phase = 'probing';
    this.error = null;
    this.result = null;
    this.clearEstimate();
    try {
      this.video = await api.probeVideo(path);
      if (!resolutionAvailable(this.settings.resolution, this.video)) {
        this.settings.resolution = 'original';
      }
      this.output = await api.suggestOutput(path, readLastDir());
      this.phase = 'loaded';
    } catch (e) {
      this.video = null;
      this.fail(api.toAppError(e));
    }
  }

  requestEstimate(req: EstimateRequest) {
    clearTimeout(this.estimateTimer);
    const seq = ++this.estimateSeq;
    const key = `${req.input}|${req.quality}|${req.codec}|${req.resolution}|${req.useGpu}`;

    const cached = this.estimateCache.get(key);
    if (cached) {
      this.estimate = cached;
      this.estimating = false;
      this.estimateFailed = false;
      return;
    }

    this.estimating = true;
    this.estimateTimer = setTimeout(async () => {
      try {
        const result = await api.estimateSize(req);
        if (seq !== this.estimateSeq) return;
        if (result) {
          this.estimateCache.set(key, result);
          this.estimate = result;
          this.estimateFailed = false;
        }
      } catch {
        if (seq === this.estimateSeq) {
          this.estimate = null;
          this.estimateFailed = true;
        }
      } finally {
        if (seq === this.estimateSeq) this.estimating = false;
      }
    }, 400);
  }

  private clearEstimate() {
    clearTimeout(this.estimateTimer);
    this.estimateSeq++;
    this.estimateCache.clear();
    this.estimate = null;
    this.estimating = false;
    this.estimateFailed = false;
  }

  async chooseOutput(): Promise<boolean> {
    if (!this.video) return false;
    const picked = await save({
      title: t('output.dialogTitle'),
      defaultPath: this.output ?? undefined,
      filters: [{ name: t('output.filter'), extensions: ['mp4'] }],
    });
    if (!picked) return false;
    this.output = picked;
    const { dir } = api.splitPath(picked);
    if (dir) writeLastDir(dir);
    return true;
  }

  async start() {
    if (!this.video) return;
    if (!this.output && !(await this.chooseOutput())) return;
    clearTimeout(this.estimateTimer);
    this.estimateSeq++;
    this.estimating = false;
    this.phase = 'compressing';
    this.progress = { percent: 0, speed: null, etaS: null };
    this.notice = null;
    this.error = null;
    try {
      await api.compress({
        input: this.video.path,
        quality: this.settings.quality,
        codec: this.settings.codec,
        resolution: this.settings.resolution,
        useGpu: this.settings.useGpu && this.gpuAvailable,
        output: this.output,
      });
    } catch (e) {
      this.fail(api.toAppError(e));
    }
  }

  cancel() {
    api.cancelCompression();
  }

  adjust() {
    this.result = null;
    this.error = null;
    this.phase = this.video ? 'loaded' : 'idle';
  }

  reset() {
    this.video = null;
    this.output = null;
    this.clearEstimate();
    this.result = null;
    this.error = null;
    this.progress = null;
    this.notice = null;
    this.phase = 'idle';
  }

  private fail(e: AppError) {
    this.error = e;
    this.phase = 'error';
  }
}

export const app = new AppStore();
