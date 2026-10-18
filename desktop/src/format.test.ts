import { describe, expect, it } from "vitest";
import { extractUrls, formatBytes, formatDuration, formatEta, progressLine, qualityLabel } from "./format";

describe("format helpers", () => {
  it("formats bytes in decimal units", () => {
    expect(formatBytes(512)).toBe("512 B");
    expect(formatBytes(1_400_000_000)).toBe("1.4 GB");
    expect(formatBytes(236_000_000)).toBe("236 MB");
    expect(formatBytes(null)).toBe("");
  });

  it("formats durations and ETAs", () => {
    expect(formatDuration(61)).toBe("1:01");
    expect(formatDuration(3 * 3600 + 5)).toBe("3:00:05");
    expect(formatEta(30)).toBe("less than a minute left");
    expect(formatEta(130)).toBe("about 2 min left");
  });

  it("builds the progress line", () => {
    const line = progressLine({
      percent: 62.4,
      downloaded: 1_400_000_000,
      total: 2_300_000_000,
      speed: 8_400_000,
      eta: 120,
      entry: null,
      entries: null,
      postprocessing: false,
    });
    expect(line).toBe("62% · 1.4 of 2.3 GB · 8.4 MB/s · about 2 min left");
  });

  it("labels qualities", () => {
    expect(qualityLabel("audio", 320)).toBe("MP3 · 320 kbps");
    expect(qualityLabel("video", 2160)).toBe("MP4 · 2160p 4K");
  });

  it("extracts links from pasted text", () => {
    const text = "watch https://youtu.be/abc, and\nhttps://www.youtube.com/watch?v=x1 (https://youtu.be/abc)";
    expect(extractUrls(text)).toEqual(["https://youtu.be/abc", "https://www.youtube.com/watch?v=x1"]);
    expect(extractUrls("no links here")).toEqual([]);
  });
});
