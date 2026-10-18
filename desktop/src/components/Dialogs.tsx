import { useEffect, useRef, type ReactNode } from "react";
import { writeText } from "@tauri-apps/plugin-clipboard-manager";

/** Native modal <dialog>: focus trapping, Escape and backdrop come from the browser. */
export function Modal({ title, open, onClose, children }: { title: string; open: boolean; onClose: () => void; children: ReactNode }) {
  const ref = useRef<HTMLDialogElement>(null);
  useEffect(() => {
    const dialog = ref.current;
    if (!dialog) return;
    if (open && !dialog.open) dialog.showModal();
    if (!open && dialog.open) dialog.close();
  }, [open]);
  return (
    <dialog ref={ref} onClose={onClose} aria-labelledby="modal-title">
      {open && (
        <div className="dialog-body">
          <div style={{ display: "flex", justifyContent: "space-between", alignItems: "center", gap: 16 }}>
            <h2 id="modal-title">{title}</h2>
            <button type="button" className="btn" onClick={onClose}>
              Close
            </button>
          </div>
          {children}
        </div>
      )}
    </dialog>
  );
}

export function LogDialog({ lines, title, onClose }: { lines: string[] | null; title: string; onClose: () => void }) {
  const text = (lines ?? []).join("\n");
  return (
    <Modal title={`Log · ${title}`} open={lines !== null} onClose={onClose}>
      <pre className="log">{text || "No output was recorded for this download."}</pre>
      <div>
        <button type="button" className="btn" onClick={() => void writeText(text)} disabled={!text}>
          Copy log
        </button>
      </div>
    </Modal>
  );
}

const NOTICES = [
  {
    name: "StreamQ",
    license: "MIT License",
    detail: "Copyright (c) 2025-2026 ivanerror.",
    url: "https://github.com/ivanerror/streamq",
  },
  {
    name: "yt-dlp",
    license: "The Unlicense",
    detail: "Downloaded on first launch from the official yt-dlp releases and updated automatically.",
    url: "https://github.com/yt-dlp/yt-dlp",
  },
  {
    name: "FFmpeg and FFprobe",
    license: "GNU GPL v3",
    detail:
      "Downloaded on first launch as separate programs: builds by yt-dlp/FFmpeg-Builds (Windows, Linux) and martin-riedl.de (macOS). Source code: ffmpeg.org.",
    url: "https://ffmpeg.org/legal.html",
  },
  {
    name: "Deno",
    license: "MIT License",
    detail: "Downloaded on first launch from the official Deno releases.",
    url: "https://github.com/denoland/deno",
  },
  {
    name: "Tauri, React, Geist",
    license: "MIT / Apache-2.0 / SIL Open Font License",
    detail: "Application framework, UI library and typeface.",
    url: "https://tauri.app",
  },
];

export function NoticesDialog({ open, onClose, onOpenUrl }: { open: boolean; onClose: () => void; onOpenUrl: (url: string) => void }) {
  return (
    <Modal title="Licenses & notices" open={open} onClose={onClose}>
      <ul className="notice-list">
        {NOTICES.map((n) => (
          <li key={n.name} className="card" style={{ padding: 16, display: "flex", flexDirection: "column", gap: 4 }}>
            <b>
              {n.name} <span className="muted">· {n.license}</span>
            </b>
            <span className="muted">{n.detail}</span>
            <span>
              <button type="button" className="link-button" onClick={() => onOpenUrl(n.url)}>
                {n.url.replace("https://", "")}
              </button>
            </span>
          </li>
        ))}
      </ul>
    </Modal>
  );
}
