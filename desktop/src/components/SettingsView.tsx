import { useEffect, useState } from "react";
import { open as openDialog } from "@tauri-apps/plugin-dialog";
import type { ComponentStatus, Format, Settings } from "../api";

interface Props {
  settings: Settings;
  components: ComponentStatus[];
  audioQualities: number[];
  videoQualities: number[];
  version: string;
  onSave: (settings: Settings) => Promise<void>;
  onUpdateEngine: () => Promise<void>;
  onCheckAppUpdate: () => Promise<void>;
  onShowNotices: () => void;
}

function Switch({ id, checked, onChange }: { id: string; checked: boolean; onChange: (value: boolean) => void }) {
  return <button id={id} type="button" role="switch" aria-checked={checked} className="switch" onClick={() => onChange(!checked)} />;
}

export function SettingsView(props: Props) {
  const { settings } = props;
  const [template, setTemplate] = useState(settings.filenameTemplate);
  const [busy, setBusy] = useState<"engine" | "app" | null>(null);
  useEffect(() => setTemplate(settings.filenameTemplate), [settings.filenameTemplate]);

  const save = (patch: Partial<Settings>) => props.onSave({ ...settings, ...patch });
  const engine = props.components.find((c) => c.kind === "ytDlp");

  async function chooseFolder() {
    const dir = await openDialog({ directory: true, defaultPath: settings.outputDir, title: "Save downloads to" });
    if (typeof dir === "string") await save({ outputDir: dir });
  }

  async function run(kind: "engine" | "app", action: () => Promise<void>) {
    setBusy(kind);
    try {
      await action();
    } finally {
      setBusy(null);
    }
  }

  return (
    <div className="settings">
      <section className="card" aria-labelledby="s-downloads">
        <h2 id="s-downloads">Downloads</h2>
        <div className="setting">
          <div className="setting-label">
            <span>Save to</span>
            <span className="mono muted selectable">{settings.outputDir}</span>
          </div>
          <button type="button" className="btn" onClick={chooseFolder}>
            Change…
          </button>
        </div>
        <div className="setting">
          <span id="s-format-label">Default format</span>
          <fieldset className="segmented compact" aria-labelledby="s-format-label">
            {(["audio", "video"] as Format[]).map((format) => (
              <label key={format}>
                <input type="radio" name="s-format" checked={settings.defaultFormat === format} onChange={() => save({ defaultFormat: format })} />
                <span>{format === "audio" ? "Audio · MP3" : "Video · MP4"}</span>
              </label>
            ))}
          </fieldset>
        </div>
        <div className="setting">
          <label htmlFor="s-vq">Default video quality</label>
          <select id="s-vq" className="select" value={settings.defaultVideoQuality} onChange={(e) => save({ defaultVideoQuality: Number(e.target.value) })}>
            {[...props.videoQualities].reverse().map((q) => (
              <option key={q} value={q}>
                {q === 2160 ? "2160p · 4K" : `${q}p`}
              </option>
            ))}
          </select>
        </div>
        <div className="setting">
          <label htmlFor="s-aq">Default audio quality</label>
          <select id="s-aq" className="select" value={settings.defaultAudioQuality} onChange={(e) => save({ defaultAudioQuality: Number(e.target.value) })}>
            {[...props.audioQualities].reverse().map((q) => (
              <option key={q} value={q}>
                {q} kbps
              </option>
            ))}
          </select>
        </div>
        <div className="setting">
          <div className="setting-label">
            <span id="s-parallel-label">Parallel downloads</span>
            <span className="muted">More than 2 can trigger YouTube rate limits</span>
          </div>
          <fieldset className="segmented compact" aria-labelledby="s-parallel-label">
            {[1, 2, 3].map((n) => (
              <label key={n}>
                <input type="radio" name="s-parallel" checked={settings.parallelDownloads === n} onChange={() => save({ parallelDownloads: n })} />
                <span style={{ minWidth: 44, justifyContent: "center" }}>{n}</span>
              </label>
            ))}
          </fieldset>
        </div>
        <div className="setting">
          <div className="setting-label">
            <label htmlFor="s-template">File name</label>
            <span className="muted">yt-dlp output template; must contain %(ext)s</span>
          </div>
          <input
            id="s-template"
            className="text-input mono"
            style={{ flex: "0 1 320px" }}
            value={template}
            onChange={(e) => setTemplate(e.target.value)}
            onBlur={() => template !== settings.filenameTemplate && save({ filenameTemplate: template })}
          />
        </div>
      </section>

      <section className="card" aria-labelledby="s-engine">
        <h2 id="s-engine">Download engine</h2>
        <div className="setting">
          <div className="setting-label">
            <span>
              yt-dlp <span className="mono muted">{engine?.version ?? "not installed yet"}</span>
            </span>
            <span className="muted">Updates fix YouTube changes without reinstalling StreamQ</span>
          </div>
          <button type="button" className="btn" disabled={busy === "engine" || engine?.state !== "ready"} onClick={() => run("engine", props.onUpdateEngine)}>
            {busy === "engine" ? "Checking…" : "Check now"}
          </button>
        </div>
        <div className="setting">
          <label htmlFor="s-auto">Update the engine automatically</label>
          <Switch id="s-auto" checked={settings.autoUpdateEngine} onChange={(v) => save({ autoUpdateEngine: v })} />
        </div>
        <div className="setting">
          <div className="chips">
            {props.components.map((c) => (
              <span className="chip" key={c.kind}>
                {c.label} · {c.state === "ready" ? (c.source === "system" ? "from this computer" : "downloaded") : c.state}
                {c.version && c.kind !== "ytDlp" ? ` · ${c.version}` : ""}
              </span>
            ))}
          </div>
        </div>
      </section>

      <section className="card" aria-labelledby="s-app">
        <h2 id="s-app">App</h2>
        <div className="setting">
          <label htmlFor="s-notify">Notify me when downloads finish</label>
          <Switch id="s-notify" checked={settings.notifyOnFinish} onChange={(v) => save({ notifyOnFinish: v })} />
        </div>
        <div className="setting">
          <div className="setting-label">
            <label htmlFor="s-clip">Suggest links from the clipboard</label>
            <span className="muted">Read only when StreamQ is in front</span>
          </div>
          <Switch id="s-clip" checked={settings.suggestClipboardLinks} onChange={(v) => save({ suggestClipboardLinks: v })} />
        </div>
        <div className="setting">
          <span>
            StreamQ <span className="mono muted">{props.version}</span>
          </span>
          <div className="toolbar">
            <button type="button" className="btn" disabled={busy === "app"} onClick={() => run("app", props.onCheckAppUpdate)}>
              {busy === "app" ? "Checking…" : "Check for app update"}
            </button>
            <button type="button" className="btn" onClick={props.onShowNotices}>
              Licenses &amp; notices
            </button>
          </div>
        </div>
      </section>
    </div>
  );
}
