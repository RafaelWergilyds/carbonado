import { i18n } from './i18n.svelte';

const formatters = new Map<string, Intl.NumberFormat>();

function nf(digits: number): Intl.NumberFormat {
  const key = `${i18n.intl}|${digits}`;
  let f = formatters.get(key);
  if (!f) {
    f = new Intl.NumberFormat(i18n.intl, { maximumFractionDigits: digits });
    formatters.set(key, f);
  }
  return f;
}

export function formatBytes(bytes: number): string {
  const units = ['B', 'KB', 'MB', 'GB', 'TB'];
  let value = bytes;
  let i = 0;
  while (value >= 1024 && i < units.length - 1) {
    value /= 1024;
    i++;
  }
  return `${nf(i === 0 ? 0 : 1).format(value)} ${units[i]}`;
}

export function formatDuration(totalSeconds: number): string {
  const s = Math.max(0, Math.round(totalSeconds));
  const h = Math.floor(s / 3600);
  const m = Math.floor((s % 3600) / 60);
  const sec = s % 60;
  const pad = (n: number) => n.toString().padStart(2, '0');
  return h > 0 ? `${h}:${pad(m)}:${pad(sec)}` : `${m}:${pad(sec)}`;
}

export function formatPercent(value: number): string {
  return `${nf(0).format(value)}%`;
}

export function formatSpeed(speed: number | null): string {
  return speed == null ? '—' : `${nf(1).format(speed)}x`;
}
