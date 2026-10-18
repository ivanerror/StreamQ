import type { Format, Item, Progress } from "./api";

export function formatBytes(bytes: number | null | undefined): string {
  if (bytes == null || !Number.isFinite(bytes)) return "";
  const units = ["B", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1000 && unit < units.length - 1) {
    value /= 1000;
    unit += 1;
  }
  const digits = unit === 0 || value >= 100 ? 0 : 1;
  return `${value.toFixed(digits)} ${units[unit]}`;
}

export function formatSpeed(bytesPerSecond: number | null | undefined): string {
  if (!bytesPerSecond) return "";
  return `${formatBytes(bytesPerSecond)}/s`;
}

export function formatEta(seconds: number | null | undefined): string {
  if (seconds == null) return "";
  if (seconds < 60) return "less than a minute left";
  const minutes = Math.round(seconds / 60);
  if (minutes < 60) return `about ${minutes} min left`;
  const hours = Math.floor(minutes / 60);
  return `about ${hours} h ${minutes % 60} min left`;
}

export function formatDuration(seconds: number | null | undefined): string {
  if (seconds == null || !Number.isFinite(seconds)) return "";
  const total = Math.round(seconds);
  const h = Math.floor(total / 3600);
  const m = Math.floor((total % 3600) / 60);
  const s = total % 60;
  const pad = (n: number) => String(n).padStart(2, "0");
  return h > 0 ? `${h}:${pad(m)}:${pad(s)}` : `${m}:${pad(s)}`;
}

export function qualityLabel(format: Format, quality: number): string {
  if (format === "audio") return `MP3 · ${quality} kbps`;
  if (quality === 2160) return "MP4 · 2160p 4K";
  return `MP4 · ${quality}p`;
}

/** "62% · 1.4 of 2.3 GB · 8.4 MB/s · about 2 min left" */
export function progressLine(progress: Progress): string {
  const parts: string[] = [`${Math.floor(progress.percent)}%`];
  if (progress.entry && progress.entries) parts.unshift(`Video ${progress.entry} of ${progress.entries}`);
  if (progress.downloaded != null && progress.total) {
    const done = formatBytes(progress.downloaded);
    const total = formatBytes(progress.total);
    // "1.4 of 2.3 GB" when both share a unit
    const unit = total.split(" ")[1];
    parts.push(`${done.endsWith(` ${unit}`) ? done.split(" ")[0] : done} of ${total}`);
  }
  const speed = formatSpeed(progress.speed);
  if (speed) parts.push(speed);
  const eta = formatEta(progress.eta);
  if (eta && progress.percent < 100) parts.push(eta);
  return parts.join(" · ");
}

export function itemTitle(item: Item): string {
  return item.title ?? item.url.replace(/^https?:\/\/(www\.)?/, "");
}

/** Split pasted or dropped text into candidate links. */
export function extractUrls(text: string): string[] {
  const found = text.match(/https?:\/\/[^\s"'<>]+/g) ?? [];
  return [...new Set(found.map((url) => url.replace(/[),.]+$/, "")))];
}

export function isActive(item: Item): boolean {
  return item.status === "waiting" || item.status === "downloading";
}
