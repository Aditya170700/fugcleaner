/**
 * Format bytes to human readable decimal string (Base 1000, matching macOS Finder)
 */
export function formatBytes(bytes: number, decimals: number = 1): string {
  if (bytes === 0) return '0 B';
  if (!bytes || bytes < 0) return '0 B';

  const k = 1000;
  const dm = decimals < 0 ? 0 : decimals;
  const sizes = ['B', 'kB', 'MB', 'GB', 'TB', 'PB'];

  const i = Math.floor(Math.log(bytes) / Math.log(k));
  const clampedIndex = Math.min(i, sizes.length - 1);
  const value = bytes / Math.pow(k, clampedIndex);

  return `${value.toFixed(dm)} ${sizes[clampedIndex]}`;
}

/**
 * Format relative time in Indonesian
 */
export function formatRelativeTime(timestampMs: number): string {
  if (!timestampMs) return '-';

  const now = Date.now();
  const diffMs = now - timestampMs;

  if (diffMs < 0) return 'baru saja';

  const seconds = Math.floor(diffMs / 1000);
  const minutes = Math.floor(seconds / 60);
  const hours = Math.floor(minutes / 60);
  const days = Math.floor(hours / 24);
  const months = Math.floor(days / 30);
  const years = Math.floor(days / 365);

  if (years > 0) return `${years} tahun lalu`;
  if (months > 0) return `${months} bulan lalu`;
  if (days > 0) return `${days} hari lalu`;
  if (hours > 0) return `${hours} jam lalu`;
  if (minutes > 0) return `${minutes} menit lalu`;
  return 'baru saja';
}

/**
 * Format localized date time
 */
export function formatDateTime(timestampMs: number): string {
  if (!timestampMs) return '-';
  const date = new Date(timestampMs);
  return date.toLocaleString('id-ID', {
    year: 'numeric',
    month: 'short',
    day: 'numeric',
    hour: '2-digit',
    minute: '2-digit',
  });
}
