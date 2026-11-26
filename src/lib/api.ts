import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type {
  AppError,
  CompressOptions,
  EncoderSupport,
  EstimateRequest,
  JobResult,
  ProgressEvent,
  SizeEstimate,
  VideoInfo,
} from './types';

export const VIDEO_EXTENSIONS = ['mp4', 'mov', 'mkv', 'avi', 'webm', 'm4v', 'wmv', 'flv', 'ts', 'mts'];

export const probeVideo = (path: string) => invoke<VideoInfo>('probe_video', { path });
export const detectEncoders = () => invoke<EncoderSupport>('detect_encoders');
export const suggestOutput = (input: string, dir: string | null) =>
  invoke<string>('suggest_output', { input, dir });
export const estimateSize = (request: EstimateRequest) =>
  invoke<SizeEstimate | null>('estimate_size', { request });
export const compress = (options: CompressOptions) => invoke<void>('compress', { options });
export const cancelCompression = () => invoke<void>('cancel_compression');

export interface CompressHandlers {
  onProgress: (e: ProgressEvent) => void;
  onFallback: (reason: string) => void;
  onDone: (r: JobResult) => void;
  onError: (e: AppError) => void;
  onCancelled: () => void;
}

export async function subscribeCompress(h: CompressHandlers): Promise<UnlistenFn> {
  const unlisteners = await Promise.all([
    listen<ProgressEvent>('compress://progress', (e) => h.onProgress(e.payload)),
    listen<{ reason: string }>('compress://fallback', (e) => h.onFallback(e.payload.reason)),
    listen<JobResult>('compress://done', (e) => h.onDone(e.payload)),
    listen<AppError>('compress://error', (e) => h.onError(e.payload)),
    listen('compress://cancelled', () => h.onCancelled()),
  ]);
  return () => unlisteners.forEach((u) => u());
}

export function toAppError(e: unknown): AppError {
  if (e && typeof e === 'object' && 'kind' in e && 'message' in e) {
    return e as AppError;
  }
  return { kind: 'unknown', message: String(e), detail: null };
}

export function splitPath(path: string): { dir: string; name: string } {
  const i = Math.max(path.lastIndexOf('\\'), path.lastIndexOf('/'));
  return i < 0 ? { dir: '', name: path } : { dir: path.slice(0, i), name: path.slice(i + 1) };
}

export function isVideoPath(path: string): boolean {
  const ext = path.split('.').pop()?.toLowerCase() ?? '';
  return VIDEO_EXTENSIONS.includes(ext);
}
