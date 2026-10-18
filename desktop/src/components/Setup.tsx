import type { ComponentStatus } from "../api";
import { formatBytes } from "../format";

const PURPOSE: Record<ComponentStatus["kind"], string> = {
  ytDlp: "download engine",
  ffmpeg: "merges video and converts MP3",
  deno: "solves YouTube's JavaScript checks",
};

function describe(status: ComponentStatus): string {
  switch (status.state) {
    case "ready":
      return status.source === "system" ? `Using the one installed on this computer${status.version ? ` · ${status.version}` : ""}` : `Ready${status.version ? ` · ${status.version}` : ""}`;
    case "downloading":
      return status.total
        ? `Downloading ${formatBytes(status.downloaded)} of ${formatBytes(status.total)}`
        : `Downloading ${formatBytes(status.downloaded)}`;
    case "installing":
      return "Checking and installing…";
    case "failed":
      return status.error ?? "Download failed";
    default:
      return "Waiting";
  }
}

/** First-launch panel: shows the tools StreamQ downloads instead of bundling them. */
export function SetupPanel({ components, onRetry }: { components: ComponentStatus[]; onRetry: () => void }) {
  const failed = components.some((c) => c.state === "failed");
  return (
    <section className="card setup" aria-labelledby="setup-title" aria-live="polite">
      <div>
        <h2 id="setup-title">{failed ? "Setup could not finish" : "Getting StreamQ ready"}</h2>
        <span className="muted" style={{ fontSize: 14 }}>
          {failed
            ? "Check your internet connection and try again. Links you add stay in the queue."
            : "First launch only: downloading the tools StreamQ uses. You can add links meanwhile."}
        </span>
      </div>
      {components.map((c) => {
        const percent = c.total ? Math.min(100, (c.downloaded / c.total) * 100) : null;
        return (
          <div className="setup-row" key={c.kind}>
            <b>{c.label}</b>
            <div style={{ display: "flex", flexDirection: "column", gap: 6, minWidth: 0 }}>
              {(c.state === "downloading" || c.state === "installing") && (
                <div
                  className={`progress${percent == null || c.state === "installing" ? " indeterminate" : ""}`}
                  role="progressbar"
                  aria-label={`${c.label} download`}
                  aria-valuemin={0}
                  aria-valuemax={100}
                  aria-valuenow={percent == null ? undefined : Math.floor(percent)}
                >
                  <div style={percent == null || c.state === "installing" ? undefined : { width: `${percent}%` }} />
                </div>
              )}
              <span className={`item-meta${c.state === "failed" ? " danger" : ""}`}>
                {describe(c)} · {PURPOSE[c.kind]}
              </span>
            </div>
            <span className={`pill ${c.state === "ready" ? "completed" : c.state === "failed" ? "failed" : c.state === "missing" ? "waiting" : "downloading"}`}>
              {c.state === "ready" ? "Ready" : c.state === "failed" ? "Failed" : c.state === "missing" ? "Waiting" : "Downloading"}
            </span>
          </div>
        );
      })}
      {failed && (
        <div>
          <button type="button" className="btn primary" onClick={onRetry}>
            Try again
          </button>
        </div>
      )}
    </section>
  );
}
