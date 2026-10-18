import { useCallback, useEffect, useMemo, useRef, useState, type DragEvent, type ReactNode } from "react";
import { openUrl } from "@tauri-apps/plugin-opener";
import { api, onComponents, onProgress, onQueue, type Format, type Item, type Settings, type Snapshot } from "./api";
import { extractUrls, itemTitle } from "./format";
import { AddBar } from "./components/AddBar";
import { LogDialog, NoticesDialog } from "./components/Dialogs";
import { AlertIcon, CheckCircleIcon, GearIcon, ListIcon, LogoMark, PauseIcon, PlayIcon, RetryIcon } from "./components/Icons";
import { QueueItem } from "./components/QueueItem";
import { SettingsView } from "./components/SettingsView";
import { SetupPanel } from "./components/Setup";

type View = "queue" | "completed" | "failed" | "settings";
type Toast = { id: number; message: string; error?: boolean };

const VIEW_FILTER: Record<Exclude<View, "settings">, (item: Item) => boolean> = {
  queue: (item) => item.status !== "completed",
  completed: (item) => item.status === "completed",
  failed: (item) => item.status === "failed" || item.status === "cancelled",
};

function errorText(error: unknown): string {
  return typeof error === "string" ? error : error instanceof Error ? error.message : "Something went wrong";
}

export default function App() {
  const [snapshot, setSnapshot] = useState<Snapshot | null>(null);
  const [view, setView] = useState<View>("queue");
  const [format, setFormat] = useState<Format>("video");
  const [quality, setQuality] = useState(1080);
  const [toasts, setToasts] = useState<Toast[]>([]);
  const [log, setLog] = useState<{ title: string; lines: string[] } | null>(null);
  const [noticesOpen, setNoticesOpen] = useState(false);
  const [dropActive, setDropActive] = useState(false);
  const toastId = useRef(0);

  const toast = useCallback((message: string, error = false) => {
    const id = ++toastId.current;
    setToasts((list) => [...list, { id, message, error }]);
    window.setTimeout(() => setToasts((list) => list.filter((t) => t.id !== id)), error ? 7000 : 4000);
  }, []);

  // Wrap a command so failures surface as a toast instead of an unhandled rejection
  const attempt = useCallback(
    async <T,>(action: () => Promise<T>): Promise<T | undefined> => {
      try {
        return await action();
      } catch (error) {
        toast(errorText(error), true);
        return undefined;
      }
    },
    [toast],
  );

  useEffect(() => {
    let disposed = false;
    const unlisten = [
      onQueue((queue) => setSnapshot((s) => (s ? { ...s, queue } : s))),
      onComponents((components) => setSnapshot((s) => (s ? { ...s, components } : s))),
      onProgress((id, progress) =>
        setSnapshot((s) =>
          s ? { ...s, queue: { ...s.queue, items: s.queue.items.map((i) => (i.id === id ? { ...i, progress } : i)) } } : s,
        ),
      ),
    ];
    api.snapshot().then((snap) => {
      if (disposed) return;
      setSnapshot(snap);
      setFormat(snap.settings.defaultFormat);
      setQuality(snap.settings.defaultFormat === "audio" ? snap.settings.defaultAudioQuality : snap.settings.defaultVideoQuality);
    });
    return () => {
      disposed = true;
      unlisten.forEach((p) => p.then((fn) => fn()));
    };
  }, []);

  const items = snapshot?.queue.items ?? [];
  const knownUrls = useMemo(() => new Set(items.map((i) => i.url)), [items]);
  const ready = snapshot?.components.every((c) => c.state === "ready") ?? false;

  const addUrls = useCallback(
    async (urls: string[]): Promise<string[]> => {
      const result = await attempt(() => api.addUrls(urls, format, quality));
      if (result?.added) {
        setView("queue");
        toast(result.added === 1 ? "Added to the queue" : `Added ${result.added} links to the queue`);
      }
      return result?.invalid ?? [];
    },
    [attempt, format, quality, toast],
  );

  // Paste a link anywhere outside a text field to add it
  useEffect(() => {
    const onPaste = (event: ClipboardEvent) => {
      const target = event.target as HTMLElement | null;
      if (target?.closest("input, textarea, select")) return;
      const urls = extractUrls(event.clipboardData?.getData("text") ?? "");
      if (urls.length) {
        event.preventDefault();
        void addUrls(urls);
      }
    };
    document.addEventListener("paste", onPaste);
    return () => document.removeEventListener("paste", onPaste);
  }, [addUrls]);

  function onDragOver(event: DragEvent) {
    const types = event.dataTransfer.types;
    if (types.includes("text/uri-list") || types.includes("text/plain")) {
      event.preventDefault();
      setDropActive(true);
    }
  }

  function onDrop(event: DragEvent) {
    event.preventDefault();
    setDropActive(false);
    const text = event.dataTransfer.getData("text/uri-list") || event.dataTransfer.getData("text/plain");
    const urls = extractUrls(text);
    if (urls.length) void addUrls(urls);
  }

  function changeFormat(next: Format) {
    setFormat(next);
    if (snapshot) setQuality(next === "audio" ? snapshot.settings.defaultAudioQuality : snapshot.settings.defaultVideoQuality);
  }

  async function saveSettings(settings: Settings) {
    const saved = await attempt(() => api.saveSettings(settings));
    if (saved) setSnapshot((s) => (s ? { ...s, settings: saved } : s));
  }

  async function updateEngine() {
    const version = await attempt(() => api.updateEngine());
    if (version !== undefined) toast(version ? `yt-dlp updated to ${version}` : "yt-dlp is up to date");
  }

  async function checkAppUpdate() {
    const update = await attempt(() => api.checkAppUpdate());
    if (update === null) toast("StreamQ is up to date");
    if (update) {
      toast(`StreamQ ${update.version} is available — opening the download page`);
      await attempt(() => openUrl(update.url));
    }
  }

  async function viewLog(item: Item) {
    const lines = await attempt(() => api.log(item.id));
    if (lines) setLog({ title: itemTitle(item), lines });
  }

  if (!snapshot) {
    return <div className="empty" style={{ height: "100%", justifyContent: "center" }}>Loading…</div>;
  }

  const counts = {
    queue: items.filter(VIEW_FILTER.queue).length,
    completed: items.filter(VIEW_FILTER.completed).length,
    failed: items.filter((i) => i.status === "failed").length,
  };
  const downloading = items.filter((i) => i.status === "downloading").length;
  const waiting = items.filter((i) => i.status === "waiting").length;
  const retryable = items.filter((i) => i.status === "failed" || i.status === "cancelled").length;
  const cancelled = items.filter((i) => i.status === "cancelled").length;
  const engine = snapshot.components.find((c) => c.kind === "ytDlp");
  const paused = snapshot.queue.paused;
  const visible = view === "settings" ? [] : items.filter(VIEW_FILTER[view]);

  const summary = [
    downloading && `${downloading} downloading`,
    waiting && `${waiting} waiting`,
    counts.failed && `${counts.failed} failed`,
    cancelled && `${cancelled} cancelled`,
  ].filter(Boolean);

  const navItem = (id: View, label: string, icon: ReactNode, count?: number, tone?: "accent" | "danger") => (
    <button type="button" className="nav-link" aria-current={view === id ? "page" : undefined} onClick={() => setView(id)}>
      {icon}
      <span className="label">{label}</span>
      {count ? <span className={`count${tone ? ` ${tone}` : ""}`}>{count}</span> : null}
    </button>
  );

  return (
    <div className="app">
      <nav className="sidebar" aria-label="Main">
        <div className="brand">
          <div className="brand-mark">
            <LogoMark />
          </div>
          <span>StreamQ</span>
        </div>
        <div className="nav">
          {navItem("queue", "Queue", <ListIcon />, counts.queue, "accent")}
          {navItem("completed", "Completed", <CheckCircleIcon />, counts.completed)}
          {navItem("failed", "Failed", <AlertIcon />, counts.failed, "danger")}
        </div>
        <div className="spacer" />
        {navItem("settings", "Settings", <GearIcon />)}
        <div className="engine-card">
          <span className="engine-state" style={{ color: ready ? "var(--success)" : engine?.state === "failed" ? "var(--danger)" : "var(--accent-strong)" }}>
            <span className="dot" />
            {ready ? "Engine ready" : engine?.state === "failed" || snapshot.components.some((c) => c.state === "failed") ? "Setup failed" : "Setting up…"}
          </span>
          <span className="mono muted" style={{ fontSize: 12 }}>
            yt-dlp {engine?.version ?? "—"}
          </span>
        </div>
      </nav>

      <main className="main" onDragOver={onDragOver} onDragLeave={() => setDropActive(false)} onDrop={onDrop}>
        <div className="main-scroll">
          {view === "settings" ? (
            <>
              <div className="page-header">
                <h1>Settings</h1>
              </div>
              <SettingsView
                settings={snapshot.settings}
                components={snapshot.components}
                audioQualities={snapshot.audioQualities}
                videoQualities={snapshot.videoQualities}
                version={snapshot.version}
                onSave={saveSettings}
                onUpdateEngine={updateEngine}
                onCheckAppUpdate={checkAppUpdate}
                onShowNotices={() => setNoticesOpen(true)}
              />
            </>
          ) : (
            <>
              <div className="page-header">
                <div style={{ display: "flex", flexDirection: "column", gap: 4 }}>
                  <h1>{view === "queue" ? "Queue" : view === "completed" ? "Completed" : "Failed"}</h1>
                  <span className="muted">
                    {view === "queue"
                      ? summary.length
                        ? summary.join(" · ") + (paused ? " · paused" : "")
                        : "Nothing in the queue"
                      : `${visible.length} item${visible.length === 1 ? "" : "s"}`}
                  </span>
                </div>
                <div className="toolbar">
                  {view === "queue" && (
                    <>
                      <button type="button" className="btn" onClick={() => attempt(() => api.setPaused(!paused))} disabled={!paused && downloading + waiting === 0}>
                        {paused ? <PlayIcon /> : <PauseIcon />}
                        {paused ? "Resume all" : "Pause all"}
                      </button>
                      <button type="button" className="btn danger" onClick={() => attempt(() => api.cancelAll())} disabled={downloading + waiting === 0}>
                        Cancel all
                      </button>
                    </>
                  )}
                  {view === "failed" && (
                    <button type="button" className="btn" onClick={() => attempt(() => api.retry())} disabled={retryable === 0}>
                      <RetryIcon />
                      Retry all
                    </button>
                  )}
                  {view === "completed" && (
                    <>
                      <button type="button" className="btn" onClick={() => attempt(() => api.openOutputDir())}>
                        Open folder
                      </button>
                      <button type="button" className="btn" onClick={() => attempt(() => api.clearCompleted())} disabled={visible.length === 0}>
                        Clear list
                      </button>
                    </>
                  )}
                </div>
              </div>

              {!ready && <SetupPanel components={snapshot.components} onRetry={() => attempt(() => api.retrySetup())} />}

              {view === "queue" && (
                <AddBar
                  format={format}
                  quality={quality}
                  audioQualities={snapshot.audioQualities}
                  videoQualities={snapshot.videoQualities}
                  suggestClipboard={snapshot.settings.suggestClipboardLinks}
                  knownUrls={knownUrls}
                  dropActive={dropActive}
                  onFormatChange={changeFormat}
                  onQualityChange={setQuality}
                  onAdd={addUrls}
                />
              )}

              {visible.length === 0 ? (
                <div className="empty">
                  <b>{view === "queue" ? "Your queue is empty" : view === "completed" ? "No finished downloads yet" : "No failed downloads"}</b>
                  <span>{view === "queue" ? "Paste a link above, press Ctrl/⌘+V anywhere, or drop a link onto this window." : "They will show up here."}</span>
                </div>
              ) : (
                <ul className="items" aria-label="Queue items">
                  {visible.map((item) => (
                    <QueueItem
                      key={item.id}
                      item={item}
                      ready={ready}
                      paused={paused}
                      audioQualities={snapshot.audioQualities}
                      videoQualities={snapshot.videoQualities}
                      onCancel={() => attempt(() => api.cancel(item.id))}
                      onRemove={() => attempt(() => api.remove(item.id))}
                      onRetry={() => attempt(() => api.retry([item.id]))}
                      onReveal={() => attempt(() => api.revealOutput(item.id))}
                      onViewLog={() => viewLog(item)}
                      onExpand={async () => {
                        const count = await attempt(() => api.expandPlaylist(item.id));
                        if (count) toast(`Playlist expanded into ${count} items`);
                      }}
                      onOptions={(f, q) => attempt(() => api.setItemOptions(item.id, f, q))}
                    />
                  ))}
                </ul>
              )}
            </>
          )}
        </div>
        <footer className="statusbar">
          <span title={snapshot.settings.outputDir}>
            <button type="button" className="link-button" onClick={() => attempt(() => api.openOutputDir())}>
              Open folder
            </button>{" "}
            · Saving to <span className="mono selectable">{snapshot.settings.outputDir}</span>
          </span>
          <span>
            {downloading} of {snapshot.settings.parallelDownloads} parallel slot{snapshot.settings.parallelDownloads === 1 ? "" : "s"} in use
          </span>
        </footer>
      </main>

      <div className="toasts" role="status" aria-live="polite">
        {toasts.map((t) => (
          <div key={t.id} className={`toast${t.error ? " error" : ""}`}>
            {t.message}
          </div>
        ))}
      </div>
      <LogDialog lines={log?.lines ?? null} title={log?.title ?? ""} onClose={() => setLog(null)} />
      <NoticesDialog open={noticesOpen} onClose={() => setNoticesOpen(false)} onOpenUrl={(url) => attempt(() => openUrl(url))} />
    </div>
  );
}
