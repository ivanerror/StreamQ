//! Drives the yt-dlp binary: builds its command line, streams its machine-readable
//! progress, and stops it (with the FFmpeg processes it started) on cancel.

use std::path::Path;
use std::process::Stdio;
use std::time::Duration;

use process_wrap::tokio::*;
use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::sync::watch;

use crate::components::Tools;
use crate::error::{Error, Result};

const DOWNLOAD_PREFIX: &str = "[sq-dl]";
const POSTPROCESS_PREFIX: &str = "[sq-pp]";
const FILE_PREFIX: &str = "[sq-file]";
const LOG_LIMIT: usize = 300;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Format {
    Audio,
    Video,
}

/// What to download, as stored on a queue item.
#[derive(Debug, Clone)]
pub struct Job {
    pub url: String,
    pub format: Format,
    pub quality: u32,
    pub is_playlist: bool,
    pub output_dir: String,
    pub filename_template: String,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Progress {
    pub percent: f64,
    pub downloaded: Option<u64>,
    pub total: Option<u64>,
    pub speed: Option<f64>,
    pub eta: Option<u64>,
    /// Current entry and entry count while downloading a playlist
    pub entry: Option<u32>,
    pub entries: Option<u32>,
    /// Set while FFmpeg merges or converts the download
    pub postprocessing: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StopReason {
    Cancel,
    Pause,
}

#[derive(Debug)]
pub enum Outcome {
    Completed { output: Option<String> },
    Stopped(StopReason),
    Failed { error: String },
}

#[derive(Debug)]
pub struct Finished {
    pub outcome: Outcome,
    pub log: Vec<String>,
}

/// Keep yt-dlp from opening a console window when spawned from the GUI on Windows.
pub fn hide_console(command: &mut tokio::process::Command) {
    #[cfg(windows)]
    {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        command.creation_flags(CREATE_NO_WINDOW);
    }
    #[cfg(not(windows))]
    let _ = command;
}

/// Arguments every yt-dlp call gets: ignore user config files and point at our tools.
pub fn common_args(tools: &Tools) -> Vec<String> {
    let mut args = vec!["--ignore-config".to_string(), "--no-colors".to_string()];
    if let Some(dir) = &tools.ffmpeg_dir {
        args.push("--ffmpeg-location".into());
        args.push(dir.to_string_lossy().into_owned());
    }
    if let Some(deno) = &tools.deno {
        args.push("--js-runtimes".into());
        args.push(format!("deno:{}", deno.to_string_lossy()));
    }
    args
}

/// A `watch?v=…&list=…` link means "this video", not the whole playlist.
pub fn wants_single_video(url: &str) -> bool {
    let lower = url.to_ascii_lowercase();
    lower.contains("watch?v=") || lower.contains("&v=") || lower.contains("youtu.be/") || lower.contains("/shorts/")
}

/// Format selection, ported unchanged from StreamQ 1.x.
pub fn format_args(format: Format, quality: u32) -> Vec<String> {
    match format {
        Format::Audio => vec![
            "-f".into(),
            format!("bestaudio[abr<={quality}]/bestaudio/best"),
            "-x".into(),
            "--audio-format".into(),
            "mp3".into(),
            "--audio-quality".into(),
            format!("{quality}K"),
        ],
        Format::Video => vec![
            "-f".into(),
            format!("bestvideo[height<={quality}]+bestaudio/best[height<={quality}]/best"),
            // Prefer H.264/AAC so the merged MP4 plays everywhere
            "-S".into(),
            format!("res:{quality},vcodec:h264,acodec:aac"),
            "--merge-output-format".into(),
            "mp4".into(),
        ],
    }
}

pub fn download_args(job: &Job, tools: &Tools) -> Vec<String> {
    let mut args = common_args(tools);
    args.extend(
        [
            "--newline",
            "--progress",
            "--no-simulate",
            "--print",
            "after_move:[sq-file]%(filepath)s",
            "--progress-template",
            "download:[sq-dl]%(info.playlist_index)s|%(info.n_entries)s|%(progress)j",
            "--progress-template",
            "postprocess:[sq-pp]%(progress.postprocessor)s|%(progress.status)s",
        ]
        .map(String::from),
    );
    let template = if job.is_playlist {
        format!("%(playlist_title,playlist_id)s/%(playlist_index)03d - {}", job.filename_template)
    } else {
        job.filename_template.clone()
    };
    args.push("-o".into());
    args.push(Path::new(&job.output_dir).join(template).to_string_lossy().into_owned());
    args.extend(format_args(job.format, job.quality));
    args.push(if job.is_playlist { "--yes-playlist" } else { "--no-playlist" }.into());
    args.push("--".into());
    args.push(job.url.clone());
    args
}

#[derive(Debug, Deserialize)]
struct RawProgress {
    status: Option<String>,
    downloaded_bytes: Option<f64>,
    total_bytes: Option<f64>,
    total_bytes_estimate: Option<f64>,
    speed: Option<f64>,
    eta: Option<f64>,
    _percent: Option<f64>,
}

fn parse_count(text: &str) -> Option<u32> {
    text.trim().parse().ok()
}

/// Parse one `[sq-dl]index|count|{json}` line.
pub fn parse_download_line(line: &str) -> Option<Progress> {
    let rest = line.strip_prefix(DOWNLOAD_PREFIX)?;
    let mut parts = rest.splitn(3, '|');
    let entry = parse_count(parts.next()?);
    let entries = parse_count(parts.next()?);
    let raw: RawProgress = serde_json::from_str(parts.next()?).ok()?;
    let total = raw.total_bytes.or(raw.total_bytes_estimate).map(|t| t as u64);
    let downloaded = raw.downloaded_bytes.map(|d| d as u64);
    let percent = match (raw._percent, downloaded, total) {
        (Some(p), _, _) => p,
        (None, Some(d), Some(t)) if t > 0 => d as f64 / t as f64 * 100.0,
        _ if raw.status.as_deref() == Some("finished") => 100.0,
        _ => 0.0,
    };
    Some(Progress {
        percent: percent.clamp(0.0, 100.0),
        downloaded,
        total,
        speed: raw.speed,
        eta: raw.eta.map(|e| e as u64),
        entry,
        entries,
        postprocessing: false,
    })
}

/// `[sq-pp]FFmpegExtractAudio|started` → true while a converting step runs.
pub fn parse_postprocess_line(line: &str) -> Option<bool> {
    let rest = line.strip_prefix(POSTPROCESS_PREFIX)?;
    let (name, status) = rest.split_once('|')?;
    // Moving files is instant; only show "Converting…" for FFmpeg work
    if !name.starts_with("FFmpeg") && name != "Merger" {
        return Some(false);
    }
    Some(status.trim() == "started" || status.trim() == "processing")
}

/// The most useful line to show a person when yt-dlp fails.
pub fn error_message(log: &[String]) -> String {
    log.iter()
        .rev()
        .find_map(|line| line.strip_prefix("ERROR: "))
        .map(|line| {
            // Drop Python exception details: "... (caused by TransportError(...))"
            let line = line.split(" (caused by ").next().unwrap_or(line).trim();
            // "[youtube] abc: Video unavailable" → "Video unavailable"
            match line.strip_prefix('[').and_then(|rest| rest.split_once("] ")) {
                Some((_, message)) => message.split_once(": ").map(|(_, m)| m).unwrap_or(message).to_string(),
                None => line.to_string(),
            }
        })
        .unwrap_or_else(|| "yt-dlp stopped unexpectedly".to_string())
}

fn push_log(log: &mut Vec<String>, line: String) {
    if log.len() >= LOG_LIMIT {
        log.remove(0);
    }
    log.push(line);
}

fn command(ytdlp: &Path, args: &[String]) -> tokio::process::Command {
    let mut command = tokio::process::Command::new(ytdlp);
    command
        .args(args)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .env("PYTHONIOENCODING", "utf-8")
        .env("PYTHONUTF8", "1");
    hide_console(&mut command);
    command
}

/// Wrap the command so killing it also kills the FFmpeg children yt-dlp started.
fn wrap(command: tokio::process::Command) -> CommandWrap {
    let mut wrapped = CommandWrap::from(command);
    #[cfg(unix)]
    wrapped.wrap(ProcessGroup::leader());
    #[cfg(windows)]
    {
        wrapped.wrap(CreationFlags(windows::Win32::System::Threading::CREATE_NO_WINDOW));
        wrapped.wrap(JobObject);
    }
    wrapped.wrap(KillOnDrop);
    wrapped
}

/// Run one download to completion, cancellation or failure.
pub async fn run_download(
    job: &Job,
    tools: &Tools,
    mut stop: watch::Receiver<Option<StopReason>>,
    mut on_progress: impl FnMut(Progress),
) -> Result<Finished> {
    let ytdlp = tools.ytdlp.as_ref().ok_or_else(|| Error::msg("yt-dlp is not installed yet"))?;
    std::fs::create_dir_all(&job.output_dir)?;
    let args = download_args(job, tools);
    let mut child = wrap(command(ytdlp, &args)).spawn()?;
    let stdout = child.stdout().take().ok_or_else(|| Error::msg("no stdout"))?;
    let stderr = child.stderr().take().ok_or_else(|| Error::msg("no stderr"))?;
    let mut stdout = BufReader::new(stdout).lines();
    let mut stderr = BufReader::new(stderr).lines();

    let mut log = Vec::new();
    let mut output = None;
    let mut progress = Progress::default();
    let (mut stdout_done, mut stderr_done, mut stop_closed) = (false, false, false);
    let mut stop_reason = *stop.borrow();

    while stop_reason.is_none() && !(stdout_done && stderr_done) {
        // yt-dlp writes download progress to stdout and post-processing/errors to stderr
        let line = tokio::select! {
            line = stdout.next_line(), if !stdout_done => match line? {
                Some(line) => line,
                None => { stdout_done = true; continue; }
            },
            line = stderr.next_line(), if !stderr_done => match line? {
                Some(line) => line,
                None => { stderr_done = true; continue; }
            },
            changed = stop.changed(), if !stop_closed => {
                match changed {
                    Ok(()) => stop_reason = *stop.borrow(),
                    Err(_) => stop_closed = true,
                }
                continue;
            }
        };
        let line = line.trim_end_matches('\r').to_string();
        if let Some(update) = parse_download_line(&line) {
            progress = Progress { postprocessing: false, ..update };
            on_progress(progress.clone());
        } else if let Some(active) = parse_postprocess_line(&line) {
            if active != progress.postprocessing {
                progress.postprocessing = active;
                on_progress(progress.clone());
            }
        } else if let Some(path) = line.strip_prefix(FILE_PREFIX) {
            output = Some(path.trim().to_string());
        } else if !line.trim().is_empty() {
            push_log(&mut log, line);
        }
    }

    if let Some(reason) = stop_reason {
        Box::into_pin(child.kill()).await.ok();
        // Give the OS a moment to release the .part file before the next attempt
        let _ = tokio::time::timeout(Duration::from_secs(5), child.wait()).await;
        return Ok(Finished { outcome: Outcome::Stopped(reason), log });
    }

    let status = child.wait().await?;
    let outcome = if status.success() {
        Outcome::Completed { output }
    } else {
        Outcome::Failed { error: error_message(&log) }
    };
    Ok(Finished { outcome, log })
}

#[derive(Debug, Clone, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    pub title: Option<String>,
    pub uploader: Option<String>,
    pub duration: Option<f64>,
    pub thumbnail: Option<String>,
    pub is_playlist: bool,
    pub entry_count: Option<u32>,
    /// Entry URLs, so a playlist can be expanded into separate items
    #[serde(skip)]
    pub entries: Vec<String>,
}

fn entry_url(entry: &serde_json::Value) -> Option<String> {
    let url = entry.get("url").and_then(|u| u.as_str());
    if let Some(url) = url.filter(|u| u.starts_with("http")) {
        return Some(url.to_string());
    }
    let id = entry.get("id").and_then(|i| i.as_str())?;
    let key = entry.get("ie_key").and_then(|k| k.as_str()).unwrap_or("Youtube");
    (key == "Youtube").then(|| format!("https://www.youtube.com/watch?v={id}"))
}

pub fn parse_metadata(json: &serde_json::Value) -> Metadata {
    let text = |key: &str| json.get(key).and_then(|v| v.as_str()).map(|s| s.replace('\n', " ").trim().to_string());
    let is_playlist = json.get("_type").and_then(|t| t.as_str()) == Some("playlist");
    let entries: Vec<String> = json
        .get("entries")
        .and_then(|e| e.as_array())
        .map(|list| list.iter().filter_map(entry_url).collect())
        .unwrap_or_default();
    let thumbnail = text("thumbnail").or_else(|| {
        json.get("thumbnails")
            .and_then(|t| t.as_array())
            .and_then(|list| list.iter().rev().find_map(|t| t.get("url")?.as_str().map(String::from)))
    });
    Metadata {
        title: text("title"),
        uploader: text("uploader").or_else(|| text("channel")),
        duration: json.get("duration").and_then(|d| d.as_f64()),
        thumbnail,
        is_playlist,
        entry_count: json
            .get("playlist_count")
            .and_then(|c| c.as_u64())
            .map(|c| c as u32)
            .or(is_playlist.then_some(entries.len() as u32)),
        entries,
    }
}

/// Read title, thumbnail and playlist entries without downloading.
pub async fn fetch_metadata(url: &str, tools: &Tools) -> Result<Metadata> {
    let ytdlp = tools.ytdlp.as_ref().ok_or_else(|| Error::msg("yt-dlp is not installed yet"))?;
    let mut args = common_args(tools);
    args.extend(["-J", "--flat-playlist", "--skip-download"].map(String::from));
    args.push(if wants_single_video(url) { "--no-playlist" } else { "--yes-playlist" }.into());
    args.push("--".into());
    args.push(url.to_string());
    let mut command = command(ytdlp, &args);
    command.kill_on_drop(true);
    let output = tokio::time::timeout(Duration::from_secs(90), command.output())
        .await
        .map_err(|_| Error::msg("Timed out reading video information"))??;
    if !output.status.success() {
        let log: Vec<String> = String::from_utf8_lossy(&output.stderr).lines().map(String::from).collect();
        return Err(Error::msg(error_message(&log)));
    }
    let json: serde_json::Value = serde_json::from_slice(&output.stdout)?;
    Ok(parse_metadata(&json))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn tools() -> Tools {
        Tools {
            ytdlp: Some(PathBuf::from("/c/yt-dlp")),
            ffmpeg_dir: Some(PathBuf::from("/c/ffmpeg")),
            deno: Some(PathBuf::from("/c/deno/deno")),
        }
    }

    fn job(format: Format, quality: u32) -> Job {
        Job {
            url: "https://www.youtube.com/watch?v=abc".into(),
            format,
            quality,
            is_playlist: false,
            output_dir: "/out".into(),
            filename_template: "%(title)s.%(ext)s".into(),
        }
    }

    #[test]
    fn audio_args_match_streamq_1x() {
        assert_eq!(
            format_args(Format::Audio, 192),
            ["-f", "bestaudio[abr<=192]/bestaudio/best", "-x", "--audio-format", "mp3", "--audio-quality", "192K"]
        );
    }

    #[test]
    fn video_args_match_streamq_1x() {
        assert_eq!(
            format_args(Format::Video, 2160),
            [
                "-f",
                "bestvideo[height<=2160]+bestaudio/best[height<=2160]/best",
                "-S",
                "res:2160,vcodec:h264,acodec:aac",
                "--merge-output-format",
                "mp4"
            ]
        );
    }

    #[test]
    fn download_args_point_at_tools_and_output() {
        let args = download_args(&job(Format::Video, 1080), &tools());
        let joined = args.join(" ");
        assert!(joined.starts_with("--ignore-config --no-colors --ffmpeg-location /c/ffmpeg --js-runtimes deno:/c/deno/deno"));
        assert!(joined.contains(&format!("-o {}", Path::new("/out").join("%(title)s.%(ext)s").display())));
        assert!(joined.contains("--no-playlist"));
        assert_eq!(args[args.len() - 2..], ["--", "https://www.youtube.com/watch?v=abc"]);
    }

    #[test]
    fn playlist_downloads_go_into_a_folder() {
        let mut playlist = job(Format::Audio, 320);
        playlist.is_playlist = true;
        let args = download_args(&playlist, &tools());
        assert!(args.contains(&"--yes-playlist".to_string()));
        assert!(args.iter().any(|a| a.contains("%(playlist_title,playlist_id)s") && a.contains("%(playlist_index)03d - ")));
    }

    #[test]
    fn single_video_detection() {
        assert!(wants_single_video("https://www.youtube.com/watch?v=abc&list=PL1"));
        assert!(wants_single_video("https://youtu.be/abc"));
        assert!(!wants_single_video("https://www.youtube.com/playlist?list=PL1"));
    }

    #[test]
    fn parses_download_progress() {
        let line = r#"[sq-dl]NA|NA|{"status": "downloading", "downloaded_bytes": 3072, "total_bytes": 600000, "eta": 4, "speed": 2707480.9, "_percent": 0.512}"#;
        let progress = parse_download_line(line).unwrap();
        assert_eq!(progress.downloaded, Some(3072));
        assert_eq!(progress.total, Some(600000));
        assert_eq!(progress.eta, Some(4));
        assert!((progress.percent - 0.512).abs() < 1e-9);
        assert_eq!(progress.entry, None);

        let estimate = r#"[sq-dl]2|12|{"status": "downloading", "downloaded_bytes": 50, "total_bytes_estimate": 200}"#;
        let progress = parse_download_line(estimate).unwrap();
        assert_eq!((progress.entry, progress.entries), (Some(2), Some(12)));
        assert_eq!(progress.percent, 25.0);
        assert!(parse_download_line("[download] 5% of 3MiB").is_none());
    }

    #[test]
    fn parses_postprocess_state() {
        assert_eq!(parse_postprocess_line("[sq-pp]FFmpegExtractAudio|started"), Some(true));
        assert_eq!(parse_postprocess_line("[sq-pp]Merger|finished"), Some(false));
        assert_eq!(parse_postprocess_line("[sq-pp]MoveFiles|started"), Some(false));
    }

    #[test]
    fn extracts_readable_errors() {
        let log = vec![
            "[youtube] abc: Downloading webpage".to_string(),
            "ERROR: [youtube] abc: Video unavailable. This video is not available in your country".to_string(),
        ];
        assert_eq!(error_message(&log), "Video unavailable. This video is not available in your country");
        assert_eq!(error_message(&["ERROR: Unable to connect".to_string()]), "Unable to connect");
        let transport = "ERROR: [generic] x: Unable to download webpage: Connection refused (caused by TransportError(\"...\"))";
        assert_eq!(error_message(&[transport.to_string()]), "Unable to download webpage: Connection refused");
        assert_eq!(error_message(&[]), "yt-dlp stopped unexpectedly");
    }

    #[test]
    fn parses_video_and_playlist_metadata() {
        let video = serde_json::json!({"_type": "video", "title": "A\nB", "duration": 61.0, "uploader": "Chan",
            "thumbnails": [{"url": "https://i/1.jpg"}, {"url": "https://i/2.jpg"}]});
        let meta = parse_metadata(&video);
        assert_eq!(meta.title.as_deref(), Some("A B"));
        assert_eq!(meta.thumbnail.as_deref(), Some("https://i/2.jpg"));
        assert!(!meta.is_playlist);

        let playlist = serde_json::json!({"_type": "playlist", "title": "Mix", "playlist_count": 2,
            "entries": [{"id": "a1", "ie_key": "Youtube", "url": "https://www.youtube.com/watch?v=a1"}, {"id": "b2", "ie_key": "Youtube"}]});
        let meta = parse_metadata(&playlist);
        assert!(meta.is_playlist);
        assert_eq!(meta.entry_count, Some(2));
        assert_eq!(meta.entries, ["https://www.youtube.com/watch?v=a1", "https://www.youtube.com/watch?v=b2"]);
    }
}
