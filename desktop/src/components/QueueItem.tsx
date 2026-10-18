import { useState } from "react";
import type { Format, Item } from "../api";
import { formatDuration, itemTitle, progressLine, qualityLabel } from "../format";
import { CloseIcon, FolderIcon, MusicIcon, PlaylistIcon, RetryIcon, VideoIcon } from "./Icons";

interface Props {
  item: Item;
  ready: boolean;
  paused: boolean;
  audioQualities: number[];
  videoQualities: number[];
  onCancel: () => void;
  onRemove: () => void;
  onRetry: () => void;
  onReveal: () => void;
  onViewLog: () => void;
  onExpand: () => void;
  onOptions: (format: Format, quality: number) => void;
}

const STATUS_LABEL: Record<Item["status"], string> = {
  waiting: "Waiting",
  downloading: "Downloading",
  completed: "Completed",
  failed: "Failed",
  cancelled: "Cancelled",
};

function Thumbnail({ item }: { item: Item }) {
  const [broken, setBroken] = useState(false);
  const kind = item.isPlaylist ? "playlist" : item.format;
  return (
    <div className={`thumb ${kind}`}>
      {item.thumbnail && !broken ? (
        <img src={item.thumbnail} alt="" loading="lazy" referrerPolicy="no-referrer" onError={() => setBroken(true)} />
      ) : item.isPlaylist ? (
        <PlaylistIcon />
      ) : item.format === "audio" ? (
        <MusicIcon />
      ) : (
        <VideoIcon />
      )}
    </div>
  );
}

export function QueueItem(props: Props) {
  const { item } = props;
  const progress = item.progress;
  const postprocessing = item.status === "downloading" && progress?.postprocessing;
  const status = postprocessing ? "Converting" : STATUS_LABEL[item.status];
  const canEdit = item.status !== "downloading" && item.status !== "completed";
  // yt-dlp reads the page before the first byte arrives
  const started = !!progress && (progress.downloaded != null || progress.percent > 0);

  function meta() {
    switch (item.status) {
      case "downloading":
        if (postprocessing) return <span className="item-meta">Converting with FFmpeg…</span>;
        return <span className="item-meta">{started ? progressLine(progress!) : "Starting…"}</span>;
      case "failed":
        return (
          <span className="item-meta danger">
            {item.error ?? "Download failed"} ·{" "}
            <button type="button" className="link-button danger" onClick={props.onViewLog}>
              View log
            </button>
          </span>
        );
      case "cancelled":
        return <span className="item-meta">Cancelled · the partial download is kept for a retry</span>;
      case "completed":
        return <span className="item-meta">Saved{item.output ? ` · ${item.output.split(/[\\/]/).pop()}` : ""}</span>;
      case "waiting": {
        const parts: string[] = [];
        if (item.duration) parts.push(formatDuration(item.duration));
        if (item.uploader) parts.push(item.uploader);
        parts.push(!props.ready ? "Starts after setup" : props.paused ? "Paused" : "Starts when a slot frees up");
        if (item.isPlaylist && item.entryCount) {
          return (
            <span className="item-meta">
              {item.entryCount} videos ·{" "}
              <button type="button" className="link-button" onClick={props.onExpand}>
                Expand into separate items
              </button>
            </span>
          );
        }
        return <span className="item-meta">{parts.join(" · ")}</span>;
      }
    }
  }

  return (
    <li className={`item ${item.status}`}>
      <Thumbnail item={item} />
      <div className="item-body">
        <div className="item-title">
          {item.info === "pending" && !item.title ? (
            <span className="skeleton" aria-label="Fetching details" />
          ) : (
            <b title={item.url}>{itemTitle(item)}</b>
          )}
          {canEdit ? (
            <select
              className="select small"
              aria-label="Format and quality"
              value={`${item.format}:${item.quality}`}
              onChange={(e) => {
                const [format, quality] = e.target.value.split(":");
                props.onOptions(format as Format, Number(quality));
              }}
            >
              <optgroup label="Video">
                {[...props.videoQualities].reverse().map((q) => (
                  <option key={`v${q}`} value={`video:${q}`}>
                    {qualityLabel("video", q)}
                  </option>
                ))}
              </optgroup>
              <optgroup label="Audio">
                {[...props.audioQualities].reverse().map((q) => (
                  <option key={`a${q}`} value={`audio:${q}`}>
                    {qualityLabel("audio", q)}
                  </option>
                ))}
              </optgroup>
            </select>
          ) : (
            <span className="badge">{qualityLabel(item.format, item.quality)}</span>
          )}
        </div>
        {item.status === "downloading" && (
          <div
            className={`progress${postprocessing || !started ? " indeterminate" : ""}`}
            role="progressbar"
            aria-label={`Progress for ${itemTitle(item)}`}
            aria-valuemin={0}
            aria-valuemax={100}
            aria-valuenow={postprocessing || !started ? undefined : Math.floor(progress!.percent)}
          >
            <div style={postprocessing || !started ? undefined : { width: `${progress!.percent}%` }} />
          </div>
        )}
        {meta()}
      </div>
      <span className={`pill ${item.status}`}>{status}</span>
      <div className="item-actions">
        {(item.status === "failed" || item.status === "cancelled") && (
          <button type="button" className="btn" onClick={props.onRetry}>
            <RetryIcon />
            Retry
          </button>
        )}
        {item.status === "completed" && item.output && (
          <button type="button" className="btn" onClick={props.onReveal}>
            <FolderIcon />
            Show in folder
          </button>
        )}
        {item.status === "downloading" ? (
          <button type="button" className="icon-btn" aria-label={`Cancel ${itemTitle(item)}`} onClick={props.onCancel}>
            <CloseIcon />
          </button>
        ) : (
          <button type="button" className="icon-btn" aria-label={`Remove ${itemTitle(item)} from the list`} onClick={props.onRemove}>
            <CloseIcon />
          </button>
        )}
      </div>
    </li>
  );
}
