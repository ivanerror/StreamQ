// Typed bridge to the Rust core (src-tauri/src/lib.rs commands and app.rs events).
import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export type Format = "audio" | "video";
export type Status = "waiting" | "downloading" | "completed" | "failed" | "cancelled";
export type InfoState = "pending" | "ready" | "failed";

export interface Progress {
  percent: number;
  downloaded: number | null;
  total: number | null;
  speed: number | null;
  eta: number | null;
  entry: number | null;
  entries: number | null;
  postprocessing: boolean;
}

export interface Item {
  id: string;
  url: string;
  title: string | null;
  uploader: string | null;
  duration: number | null;
  thumbnail: string | null;
  isPlaylist: boolean;
  entryCount: number | null;
  info: InfoState;
  format: Format;
  quality: number;
  status: Status;
  progress: Progress | null;
  error: string | null;
  output: string | null;
  addedAt: number;
  finishedAt: number | null;
}

export interface QueueSnapshot {
  items: Item[];
  paused: boolean;
}

export interface Settings {
  outputDir: string;
  defaultFormat: Format;
  defaultVideoQuality: number;
  defaultAudioQuality: number;
  parallelDownloads: number;
  filenameTemplate: string;
  autoUpdateEngine: boolean;
  notifyOnFinish: boolean;
  suggestClipboardLinks: boolean;
}

export type ComponentKind = "ytDlp" | "ffmpeg" | "deno";
export type ComponentState = "missing" | "downloading" | "installing" | "ready" | "failed";

export interface ComponentStatus {
  kind: ComponentKind;
  label: string;
  state: ComponentState;
  source: "managed" | "system" | null;
  version: string | null;
  downloaded: number;
  total: number | null;
  error: string | null;
}

export interface Snapshot {
  queue: QueueSnapshot;
  settings: Settings;
  components: ComponentStatus[];
  audioQualities: number[];
  videoQualities: number[];
  version: string;
}

export interface AppUpdate {
  version: string;
  url: string;
}

export const api = {
  snapshot: () => invoke<Snapshot>("get_snapshot"),
  addUrls: (urls: string[], format: Format, quality: number) =>
    invoke<{ added: number; invalid: string[] }>("add_urls", { urls, format, quality }),
  setItemOptions: (id: string, format: Format, quality: number) =>
    invoke<void>("set_item_options", { id, format, quality }),
  retry: (ids?: string[]) => invoke<number>("retry_items", { ids: ids ?? null }),
  cancel: (id: string) => invoke<void>("cancel_item", { id }),
  cancelAll: () => invoke<void>("cancel_all"),
  setPaused: (paused: boolean) => invoke<void>("set_paused", { paused }),
  remove: (id: string) => invoke<void>("remove_item", { id }),
  clearCompleted: () => invoke<number>("clear_completed"),
  expandPlaylist: (id: string) => invoke<number>("expand_playlist", { id }),
  log: (id: string) => invoke<string[]>("get_log", { id }),
  saveSettings: (settings: Settings) => invoke<Settings>("save_settings", { settings }),
  updateEngine: () => invoke<string | null>("update_engine"),
  retrySetup: () => invoke<void>("retry_setup"),
  revealOutput: (id: string) => invoke<void>("reveal_output", { id }),
  openOutputDir: () => invoke<void>("open_output_dir"),
  checkAppUpdate: () => invoke<AppUpdate | null>("check_app_update"),
};

export function onQueue(handler: (snapshot: QueueSnapshot) => void): Promise<UnlistenFn> {
  return listen<QueueSnapshot>("queue-changed", (event) => handler(event.payload));
}

export function onProgress(handler: (id: string, progress: Progress) => void): Promise<UnlistenFn> {
  return listen<{ id: string; progress: Progress }>("item-progress", (event) =>
    handler(event.payload.id, event.payload.progress),
  );
}

export function onComponents(handler: (components: ComponentStatus[]) => void): Promise<UnlistenFn> {
  return listen<ComponentStatus[]>("components-changed", (event) => handler(event.payload));
}
