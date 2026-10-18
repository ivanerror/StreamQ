//! Tests against real binaries and the network. They are `#[ignore]`d by default;
//! CI runs them with `cargo test -- --ignored` on Windows, macOS and Linux.
//!
//! - `installs_all_components` downloads yt-dlp, FFmpeg and Deno into a temp dir
//!   (proving every download URL and checksum for the runner's platform).
//! - The download tests need `STREAMQ_YTDLP` (path to a yt-dlp binary); they serve
//!   a slow file from a local HTTP server, so no external site is involved.

use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::PathBuf;
use std::time::{Duration, Instant};

use tokio::sync::watch;

use crate::components::{self, Kind, Layout, Tools};
use crate::engine::{self, Format, Job, Outcome, StopReason};

/// Serve `size` bytes per request, slowly, so a download can be interrupted mid-way.
fn slow_server(size: usize, chunk_delay: Duration) -> String {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            std::thread::spawn(move || {
                let mut request = [0u8; 4096];
                let _ = stream.read(&mut request);
                let head = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: audio/mpeg\r\nContent-Length: {size}\r\nConnection: close\r\n\r\n"
                );
                if stream.write_all(head.as_bytes()).is_err() {
                    return;
                }
                let chunk = vec![0u8; 20_000];
                let mut sent = 0;
                while sent < size {
                    let n = chunk.len().min(size - sent);
                    if stream.write_all(&chunk[..n]).is_err() {
                        return;
                    }
                    sent += n;
                    std::thread::sleep(chunk_delay);
                }
            });
        }
    });
    format!("http://{address}/sample.mp3")
}

fn ytdlp() -> Option<PathBuf> {
    std::env::var_os("STREAMQ_YTDLP").map(PathBuf::from)
}

fn job(url: String, output_dir: &std::path::Path) -> Job {
    // "Video" with a direct file: yt-dlp falls back to `best`, so no FFmpeg is needed
    Job {
        url,
        format: Format::Video,
        quality: 720,
        is_playlist: false,
        output_dir: output_dir.to_string_lossy().into_owned(),
        filename_template: "%(title)s.%(ext)s".into(),
    }
}

#[tokio::test]
#[ignore = "needs STREAMQ_YTDLP"]
async fn downloads_with_progress() {
    let Some(ytdlp) = ytdlp() else { return };
    let dir = tempfile::tempdir().unwrap();
    let url = slow_server(400_000, Duration::from_millis(5));
    let tools = Tools { ytdlp: Some(ytdlp), ..Default::default() };
    let (_stop_tx, stop_rx) = watch::channel(None);
    let mut updates = Vec::new();
    let finished = engine::run_download(&job(url, dir.path()), &tools, stop_rx, |p| updates.push(p)).await.unwrap();

    let Outcome::Completed { output } = finished.outcome else { panic!("not completed: {:?}", finished) };
    let output = PathBuf::from(output.expect("yt-dlp printed the final path"));
    assert_eq!(std::fs::metadata(&output).unwrap().len(), 400_000);
    assert!(updates.len() >= 2, "expected progress updates, got {}", updates.len());
    assert_eq!(updates.last().unwrap().percent, 100.0);
    assert_eq!(updates.last().unwrap().total, Some(400_000));
}

#[tokio::test]
#[ignore = "needs STREAMQ_YTDLP"]
async fn cancel_stops_the_process_quickly() {
    let Some(ytdlp) = ytdlp() else { return };
    let dir = tempfile::tempdir().unwrap();
    // 2 MB at ~400 KB/s: runs for several seconds unless cancelled
    let url = slow_server(2_000_000, Duration::from_millis(50));
    let tools = Tools { ytdlp: Some(ytdlp), ..Default::default() };
    let (stop_tx, stop_rx) = watch::channel(None);
    let started = std::sync::Arc::new(std::sync::Mutex::new(None::<Instant>));
    let flag = started.clone();
    let cancel = tokio::spawn(async move {
        loop {
            if flag.lock().unwrap().is_some() {
                break;
            }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
        stop_tx.send(Some(StopReason::Cancel)).unwrap();
        Instant::now()
    });
    let finished = engine::run_download(&job(url, dir.path()), &tools, stop_rx, |_| {
        started.lock().unwrap().get_or_insert_with(Instant::now);
    })
    .await
    .unwrap();
    let cancelled_at = cancel.await.unwrap();

    assert!(matches!(finished.outcome, Outcome::Stopped(StopReason::Cancel)), "{:?}", finished.outcome);
    assert!(cancelled_at.elapsed() < Duration::from_secs(5), "cancel took {:?}", cancelled_at.elapsed());
    // The partial file stays so a retry can resume it
    let part = std::fs::read_dir(dir.path()).unwrap().flatten().any(|e| e.file_name().to_string_lossy().ends_with(".part"));
    assert!(part, "expected a .part file to be kept");
}

#[tokio::test]
#[ignore = "needs STREAMQ_YTDLP"]
async fn reports_errors() {
    let Some(ytdlp) = ytdlp() else { return };
    let dir = tempfile::tempdir().unwrap();
    let tools = Tools { ytdlp: Some(ytdlp), ..Default::default() };
    let (_stop_tx, stop_rx) = watch::channel(None);
    // Port 9 (discard) refuses connections on CI runners
    let finished = engine::run_download(&job("http://127.0.0.1:9/missing".into(), dir.path()), &tools, stop_rx, |_| {})
        .await
        .unwrap();
    let Outcome::Failed { error } = finished.outcome else { panic!("expected failure") };
    assert!(!error.is_empty());
    assert!(finished.log.iter().any(|l| l.starts_with("ERROR:")));
}

#[tokio::test]
#[ignore = "downloads ~200 MB from GitHub and the FFmpeg/Deno release sites"]
async fn installs_all_components() {
    let dir = tempfile::tempdir().unwrap();
    let layout = Layout { root: dir.path().join("components") };
    let client = components::http_client().unwrap();
    for kind in Kind::ALL {
        let mut last = 0;
        let path = components::install(&client, &layout, kind, |done, _| last = done).await
            .unwrap_or_else(|e| panic!("installing {} failed: {e}", kind.label()));
        assert!(last > 0, "{} reported no progress", kind.label());
        let (binary, arg) = components::version_binary(kind, &path);
        let line = components::probe_version(&binary, arg).await.unwrap();
        println!("{}: {} ({})", kind.label(), components::short_version(kind, &line), path.display());
        assert!(matches!(layout.existing(kind), Some((_, components::Source::Managed))));
    }
    assert!(!components::ytdlp_update_available(&client, &layout).await.unwrap());

    // The installed tools work together: metadata through yt-dlp with Deno and FFmpeg configured
    let tools = Tools {
        ytdlp: Some(layout.ytdlp()),
        ffmpeg_dir: Some(layout.dir(Kind::Ffmpeg)),
        deno: Some(layout.deno()),
    };
    let url = slow_server(100_000, Duration::from_millis(1));
    let metadata = engine::fetch_metadata(&url, &tools).await.unwrap();
    assert_eq!(metadata.title.as_deref(), Some("sample"));
}
