import { useEffect, useState, type FormEvent } from "react";
import { readText } from "@tauri-apps/plugin-clipboard-manager";
import type { Format } from "../api";
import { extractUrls } from "../format";
import { ClipboardIcon, LinkIcon, PlusIcon } from "./Icons";

interface Props {
  format: Format;
  quality: number;
  audioQualities: number[];
  videoQualities: number[];
  suggestClipboard: boolean;
  knownUrls: Set<string>;
  dropActive: boolean;
  onFormatChange: (format: Format) => void;
  onQualityChange: (quality: number) => void;
  onAdd: (urls: string[]) => Promise<string[]>;
}

const isMac = navigator.userAgent.includes("Mac");

export function AddBar(props: Props) {
  const [value, setValue] = useState("");
  const [error, setError] = useState<string | null>(null);
  const [suggestion, setSuggestion] = useState<string | null>(null);
  const [dismissed, setDismissed] = useState<Set<string>>(new Set());
  const qualities = props.format === "audio" ? props.audioQualities : props.videoQualities;

  // Offer the clipboard link only when the person opted in, and only while StreamQ is in front
  useEffect(() => {
    if (!props.suggestClipboard) {
      setSuggestion(null);
      return;
    }
    const check = async () => {
      try {
        const text = await readText();
        const url = extractUrls(text ?? "").find((u) => !props.knownUrls.has(u) && !dismissed.has(u));
        setSuggestion(url ?? null);
      } catch {
        setSuggestion(null);
      }
    };
    void check();
    window.addEventListener("focus", check);
    return () => window.removeEventListener("focus", check);
  }, [props.suggestClipboard, props.knownUrls, dismissed]);

  async function submit(event: FormEvent) {
    event.preventDefault();
    const urls = extractUrls(value);
    if (urls.length === 0) {
      setError(value.trim() ? "That doesn't look like a link. Paste a full http(s) address." : "Paste a YouTube link first.");
      return;
    }
    const invalid = await props.onAdd(urls);
    setError(invalid.length ? `Skipped ${invalid.length} invalid link(s).` : null);
    setValue("");
  }

  async function addSuggestion() {
    if (!suggestion) return;
    await props.onAdd([suggestion]);
    setDismissed((d) => new Set(d).add(suggestion));
    setSuggestion(null);
  }

  return (
    <form className={`card addbar${props.dropActive ? " drop-target" : ""}`} onSubmit={submit}>
      <div className="addbar-row">
        <label htmlFor="add-url" className="sr-only">
          YouTube link or playlist
        </label>
        <div className="url-field">
          <LinkIcon />
          <input
            id="add-url"
            type="text"
            inputMode="url"
            autoComplete="off"
            spellCheck={false}
            placeholder="Paste a YouTube link"
            value={value}
            onChange={(e) => {
              setValue(e.target.value);
              setError(null);
            }}
            aria-invalid={error ? true : undefined}
            aria-describedby={error ? "add-error" : undefined}
          />
          <kbd>{isMac ? "⌘ V" : "Ctrl V"}</kbd>
        </div>
        <fieldset className="segmented" aria-label="Format">
          <label>
            <input type="radio" name="add-format" checked={props.format === "audio"} onChange={() => props.onFormatChange("audio")} />
            <span>Audio · MP3</span>
          </label>
          <label>
            <input type="radio" name="add-format" checked={props.format === "video"} onChange={() => props.onFormatChange("video")} />
            <span>Video · MP4</span>
          </label>
        </fieldset>
        <label className="quality-field">
          Quality
          <select className="select" value={props.quality} onChange={(e) => props.onQualityChange(Number(e.target.value))}>
            {[...qualities].reverse().map((q) => (
              <option key={q} value={q}>
                {props.format === "audio" ? `${q} kbps` : q === 2160 ? "2160p · 4K" : `${q}p`}
              </option>
            ))}
          </select>
        </label>
        <button type="submit" className="btn primary large">
          <PlusIcon />
          Add to queue
        </button>
      </div>
      {error && (
        <div id="add-error" className="form-error" role="alert">
          {error}
        </div>
      )}
      {suggestion && (
        <div className="hint-row">
          <ClipboardIcon />
          <span>
            Link in clipboard: <span className="mono">{suggestion.length > 60 ? `${suggestion.slice(0, 60)}…` : suggestion}</span>
          </span>
          <button type="button" className="btn soft small" onClick={addSuggestion}>
            Add it
          </button>
          <button type="button" className="link-button" onClick={() => setDismissed((d) => new Set(d).add(suggestion))}>
            Dismiss
          </button>
        </div>
      )}
    </form>
  );
}
