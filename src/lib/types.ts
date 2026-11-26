export type Codec = 'h264' | 'h265';
export type Resolution = 'original' | '1080p' | '720p' | '480p';
export type Encoder = 'x264' | 'x265' | 'nvencH264' | 'nvencHevc';

export interface VideoInfo {
  path: string;
  fileName: string;
  durationS: number;
  width: number;
  height: number;
  codec: string;
  sizeBytes: number;
  bitrate: number | null;
  hasAudio: boolean;
  hdr: boolean;
}

export interface CompressOptions {
  input: string;
  quality: number;
  codec: Codec;
  resolution: Resolution;
  useGpu: boolean;
  output: string | null;
}

export interface EstimateRequest {
  input: string;
  quality: number;
  codec: Codec;
  resolution: Resolution;
  useGpu: boolean;
}

export interface SizeEstimate {
  bytes: number;
  usedGpu: boolean;
}

export interface EncoderSupport {
  h264Nvenc: boolean;
  hevcNvenc: boolean;
}

export interface ProgressEvent {
  percent: number;
  speed: number | null;
  etaS: number | null;
}

export interface JobResult {
  outputPath: string | null;
  originalBytes: number;
  finalBytes: number;
  skipped: boolean;
  encoder: Encoder;
}

export interface AppError {
  kind: string;
  code?: string;
  message: string;
  detail: string | null;
}
