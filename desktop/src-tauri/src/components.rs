//! Downloads, verifies and locates the external tools StreamQ drives:
//! yt-dlp (download engine), FFmpeg/FFprobe (merge + MP3) and Deno (YouTube JS challenges).
//!
//! Nothing is bundled with the installer; everything is fetched on first launch
//! into the app's local data directory, or taken from the system when present.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{Error, Result};

const YTDLP_RELEASE: &str = "https://github.com/yt-dlp/yt-dlp/releases/latest/download/";
const FFMPEG_RELEASE: &str = "https://github.com/yt-dlp/FFmpeg-Builds/releases/latest/download/";
const DENO_RELEASE: &str = "https://github.com/denoland/deno/releases/latest/download/";
const MACOS_FFMPEG: &str = "https://ffmpeg.martin-riedl.de/redirect/latest/macos";

// Apps launched from Finder or desktop menus don't inherit the shell PATH (e.g. Homebrew)
const EXTRA_SEARCH_DIRS: &[&str] = &["/opt/homebrew/bin", "/usr/local/bin", "/usr/bin"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Kind {
    YtDlp,
    Ffmpeg,
    Deno,
}

impl Kind {
    pub const ALL: [Kind; 3] = [Kind::YtDlp, Kind::Ffmpeg, Kind::Deno];

    pub fn label(self) -> &'static str {
        match self {
            Kind::YtDlp => "yt-dlp",
            Kind::Ffmpeg => "FFmpeg",
            Kind::Deno => "Deno",
        }
    }

    fn dir_name(self) -> &'static str {
        match self {
            Kind::YtDlp => "yt-dlp",
            Kind::Ffmpeg => "ffmpeg",
            Kind::Deno => "deno",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum State {
    Missing,
    Downloading,
    Installing,
    Ready,
    Failed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Source {
    Managed,
    System,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub kind: Kind,
    pub label: &'static str,
    pub state: State,
    pub source: Option<Source>,
    pub version: Option<String>,
    pub downloaded: u64,
    pub total: Option<u64>,
    pub error: Option<String>,
}

impl Status {
    pub fn new(kind: Kind) -> Self {
        Status {
            kind,
            label: kind.label(),
            state: State::Missing,
            source: None,
            version: None,
            downloaded: 0,
            total: None,
            error: None,
        }
    }
}

/// Resolved locations of the tools, passed to the download engine.
#[derive(Debug, Clone, Default)]
pub struct Tools {
    pub ytdlp: Option<PathBuf>,
    pub ffmpeg_dir: Option<PathBuf>,
    pub deno: Option<PathBuf>,
}

/// Where an archive's files come from and how to unpack them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Asset {
    pub url: String,
    /// URL of a checksum file listing this asset, if the publisher provides one
    pub checksum_url: Option<String>,
    /// File name looked up in the checksum file
    pub checksum_name: String,
    pub format: ArchiveFormat,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArchiveFormat {
    /// The download is the executable itself
    Binary,
    Zip,
    TarXz,
}

fn exe(name: &str) -> String {
    if cfg!(windows) {
        format!("{name}.exe")
    } else {
        name.to_string()
    }
}

/// yt-dlp release asset for an OS/arch pair (`std::env::consts` values).
pub fn ytdlp_asset_name(os: &str, arch: &str) -> Option<&'static str> {
    Some(match (os, arch) {
        ("windows", "x86_64") => "yt-dlp.exe",
        ("windows", "aarch64") => "yt-dlp_arm64.exe",
        ("windows", "x86") => "yt-dlp_x86.exe",
        // Universal binary for Intel and Apple Silicon
        ("macos", _) => "yt-dlp_macos",
        ("linux", "x86_64") => "yt-dlp_linux",
        ("linux", "aarch64") => "yt-dlp_linux_aarch64",
        _ => return None,
    })
}

pub fn ytdlp_asset(os: &str, arch: &str) -> Option<Asset> {
    let name = ytdlp_asset_name(os, arch)?;
    Some(Asset {
        url: format!("{YTDLP_RELEASE}{name}"),
        checksum_url: Some(format!("{YTDLP_RELEASE}SHA2-256SUMS")),
        checksum_name: name.to_string(),
        format: ArchiveFormat::Binary,
    })
}

/// FFmpeg assets: one archive with both tools, except on macOS (one zip per tool).
pub fn ffmpeg_assets(os: &str, arch: &str) -> Option<Vec<Asset>> {
    let yt_dlp_build = |name: &str, format| Asset {
        url: format!("{FFMPEG_RELEASE}{name}"),
        checksum_url: Some(format!("{FFMPEG_RELEASE}checksums.sha256")),
        checksum_name: name.to_string(),
        format,
    };
    Some(match (os, arch) {
        // The "shared" build keeps the download small by sharing DLLs between both tools
        ("windows", "x86_64") => vec![yt_dlp_build("ffmpeg-master-latest-win64-gpl-shared.zip", ArchiveFormat::Zip)],
        ("windows", "aarch64") => vec![yt_dlp_build("ffmpeg-master-latest-winarm64-gpl-shared.zip", ArchiveFormat::Zip)],
        ("linux", "x86_64") => vec![yt_dlp_build("ffmpeg-master-latest-linux64-gpl.tar.xz", ArchiveFormat::TarXz)],
        ("linux", "aarch64") => vec![yt_dlp_build("ffmpeg-master-latest-linuxarm64-gpl.tar.xz", ArchiveFormat::TarXz)],
        ("macos", arch) => {
            let arch = if arch == "aarch64" { "arm64" } else { "amd64" };
            ["ffmpeg", "ffprobe"]
                .iter()
                .map(|tool| Asset {
                    url: format!("{MACOS_FFMPEG}/{arch}/release/{tool}.zip"),
                    checksum_url: None,
                    checksum_name: format!("{tool}.zip"),
                    format: ArchiveFormat::Zip,
                })
                .collect()
        }
        _ => return None,
    })
}

pub fn deno_asset(os: &str, arch: &str) -> Option<Asset> {
    let arch = match arch {
        "x86_64" => "x86_64",
        "aarch64" => "aarch64",
        _ => return None,
    };
    let triple = match os {
        "windows" => "pc-windows-msvc",
        "macos" => "apple-darwin",
        "linux" => "unknown-linux-gnu",
        _ => return None,
    };
    let name = format!("deno-{arch}-{triple}.zip");
    Some(Asset {
        url: format!("{DENO_RELEASE}{name}"),
        checksum_url: Some(format!("{DENO_RELEASE}{name}.sha256sum")),
        checksum_name: name,
        format: ArchiveFormat::Zip,
    })
}

/// Find a checksum for `name` in a checksum file.
///
/// Handles `sha256sum` output (`<hash>  <name>`, one per line) and the single-hash
/// PowerShell `Get-FileHash` format Deno publishes for Windows.
pub fn parse_checksum(text: &str, name: &str) -> Option<String> {
    let is_hash = |token: &str| token.len() == 64 && token.chars().all(|c| c.is_ascii_hexdigit());
    for line in text.lines() {
        let mut parts = line.split_whitespace();
        if let (Some(hash), Some(file)) = (parts.next(), parts.next()) {
            if is_hash(hash) && file.trim_start_matches('*') == name {
                return Some(hash.to_ascii_lowercase());
            }
        }
    }
    let hashes: Vec<&str> = text.split_whitespace().filter(|t| is_hash(t)).collect();
    if hashes.len() == 1 && text.contains(name) {
        return Some(hashes[0].to_ascii_lowercase());
    }
    None
}

pub fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

pub fn sha256_file(path: &Path) -> Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 1 << 16];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(to_hex(&hasher.finalize()))
}

pub fn http_client() -> Result<reqwest::Client> {
    // Ignore the error when a provider was already installed (e.g. by an earlier client)
    let _ = rustls::crypto::ring::default_provider().install_default();
    Ok(reqwest::Client::builder()
        .user_agent(concat!("StreamQ/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(30))
        .read_timeout(Duration::from_secs(120))
        .build()?)
}

pub async fn fetch_text(client: &reqwest::Client, url: &str) -> Result<String> {
    let response = client.get(url).send().await?.error_for_status()?;
    Ok(response.text().await?)
}

/// Stream `url` into `dest`, returning the SHA-256 of the bytes written.
pub async fn download_to(
    client: &reqwest::Client,
    url: &str,
    dest: &Path,
    mut on_progress: impl FnMut(u64, Option<u64>),
) -> Result<String> {
    let response = client.get(url).send().await?.error_for_status()?;
    let total = response.content_length();
    let mut file = std::fs::File::create(dest)?;
    let mut hasher = Sha256::new();
    let mut downloaded = 0u64;
    let mut stream = response.bytes_stream();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk)?;
        hasher.update(&chunk);
        downloaded += chunk.len() as u64;
        on_progress(downloaded, total);
    }
    file.flush()?;
    Ok(to_hex(&hasher.finalize()))
}

#[cfg(unix)]
fn make_executable(path: &Path) -> Result<()> {
    use std::os::unix::fs::PermissionsExt;
    let mut permissions = std::fs::metadata(path)?.permissions();
    permissions.set_mode(permissions.mode() | 0o755);
    std::fs::set_permissions(path, permissions)?;
    Ok(())
}

#[cfg(not(unix))]
fn make_executable(_path: &Path) -> Result<()> {
    Ok(())
}

/// Copy the files StreamQ needs out of an FFmpeg/Deno archive into `dest_dir`.
///
/// `wanted` decides by file name; directory structure is flattened.
pub fn extract(archive: &Path, format: ArchiveFormat, dest_dir: &Path, wanted: &dyn Fn(&str, &str) -> bool) -> Result<Vec<String>> {
    let mut extracted = Vec::new();
    match format {
        ArchiveFormat::Binary => unreachable!("binaries are not archives"),
        ArchiveFormat::Zip => {
            let mut zip = zip::ZipArchive::new(std::fs::File::open(archive)?)?;
            for index in 0..zip.len() {
                let mut entry = zip.by_index(index)?;
                if entry.is_dir() {
                    continue;
                }
                let Some(path) = entry.enclosed_name() else { continue };
                let full = path.to_string_lossy().replace('\\', "/");
                let Some(name) = path.file_name().map(|n| n.to_string_lossy().to_string()) else { continue };
                if !wanted(&full, &name) {
                    continue;
                }
                let target = dest_dir.join(&name);
                let mut out = std::fs::File::create(&target)?;
                std::io::copy(&mut entry, &mut out)?;
                make_executable(&target)?;
                extracted.push(name);
            }
        }
        ArchiveFormat::TarXz => {
            let decoder = xz2::read::XzDecoder::new(std::fs::File::open(archive)?);
            let mut tar = tar::Archive::new(decoder);
            for entry in tar.entries()? {
                let mut entry = entry?;
                if !entry.header().entry_type().is_file() {
                    continue;
                }
                let path = entry.path()?.to_path_buf();
                let full = path.to_string_lossy().replace('\\', "/");
                let Some(name) = path.file_name().map(|n| n.to_string_lossy().to_string()) else { continue };
                if !wanted(&full, &name) {
                    continue;
                }
                let target = dest_dir.join(&name);
                let mut out = std::fs::File::create(&target)?;
                std::io::copy(&mut entry, &mut out)?;
                make_executable(&target)?;
                extracted.push(name);
            }
        }
    }
    Ok(extracted)
}

/// Run `<binary> <arg>` and return the first line of its output, or an error if it fails.
pub async fn probe_version(binary: &Path, arg: &str) -> Result<String> {
    let mut command = tokio::process::Command::new(binary);
    command.arg(arg).kill_on_drop(true);
    crate::engine::hide_console(&mut command);
    let output = tokio::time::timeout(Duration::from_secs(60), command.output())
        .await
        .map_err(|_| Error::msg(format!("{} did not respond", binary.display())))??;
    if !output.status.success() {
        return Err(Error::msg(format!("{} failed to run", binary.display())));
    }
    let text = String::from_utf8_lossy(&output.stdout);
    Ok(text.lines().next().unwrap_or_default().trim().to_string())
}

/// Shorten tool version banners (`ffmpeg version N-1234-g… Copyright…`, `deno 2.9.7 (stable…)`).
pub fn short_version(kind: Kind, line: &str) -> String {
    match kind {
        Kind::YtDlp => line.trim().to_string(),
        Kind::Ffmpeg => line
            .strip_prefix("ffmpeg version ")
            .and_then(|rest| rest.split_whitespace().next())
            .unwrap_or(line)
            .to_string(),
        Kind::Deno => line.split_whitespace().nth(1).unwrap_or(line).to_string(),
    }
}

/// Look for an executable on PATH plus the common package-manager locations.
pub fn find_on_path(name: &str) -> Option<PathBuf> {
    let file = exe(name);
    let path_var = std::env::var_os("PATH").unwrap_or_default();
    std::env::split_paths(&path_var)
        .chain(EXTRA_SEARCH_DIRS.iter().map(PathBuf::from))
        .map(|dir| dir.join(&file))
        .find(|candidate| candidate.is_file())
}

/// The managed install layout under `<app local data>/components`.
#[derive(Debug, Clone)]
pub struct Layout {
    pub root: PathBuf,
}

impl Layout {
    pub fn dir(&self, kind: Kind) -> PathBuf {
        self.root.join(kind.dir_name())
    }

    pub fn ytdlp(&self) -> PathBuf {
        self.dir(Kind::YtDlp).join(exe("yt-dlp"))
    }

    pub fn ffmpeg(&self) -> PathBuf {
        self.dir(Kind::Ffmpeg).join(exe("ffmpeg"))
    }

    pub fn ffprobe(&self) -> PathBuf {
        self.dir(Kind::Ffmpeg).join(exe("ffprobe"))
    }

    pub fn deno(&self) -> PathBuf {
        self.dir(Kind::Deno).join(exe("deno"))
    }

    /// Path of a tool already usable without downloading, and where it came from.
    pub fn existing(&self, kind: Kind) -> Option<(PathBuf, Source)> {
        match kind {
            Kind::YtDlp => self.ytdlp().is_file().then(|| (self.ytdlp(), Source::Managed)),
            Kind::Ffmpeg => {
                if self.ffmpeg().is_file() && self.ffprobe().is_file() {
                    return Some((self.dir(Kind::Ffmpeg), Source::Managed));
                }
                let ffmpeg = find_on_path("ffmpeg")?;
                let dir = ffmpeg.parent()?.to_path_buf();
                dir.join(exe("ffprobe")).is_file().then_some((dir, Source::System))
            }
            Kind::Deno => {
                if self.deno().is_file() {
                    return Some((self.deno(), Source::Managed));
                }
                find_on_path("deno").map(|path| (path, Source::System))
            }
        }
    }
}

/// Binary used to check a tool's version: the directory for FFmpeg holds `ffmpeg`.
pub fn version_binary(kind: Kind, path: &Path) -> (PathBuf, &'static str) {
    match kind {
        Kind::YtDlp => (path.to_path_buf(), "--version"),
        Kind::Ffmpeg => (path.join(exe("ffmpeg")), "-version"),
        Kind::Deno => (path.to_path_buf(), "--version"),
    }
}

/// Download, verify and install one tool into a fresh directory, then swap it in.
///
/// Returns the installed path (directory for FFmpeg) once its version check passed.
pub async fn install(
    client: &reqwest::Client,
    layout: &Layout,
    kind: Kind,
    mut on_progress: impl FnMut(u64, Option<u64>),
) -> Result<PathBuf> {
    let (os, arch) = (std::env::consts::OS, std::env::consts::ARCH);
    let unsupported = || Error::msg(format!("No {} download is available for {os}/{arch}", kind.label()));
    let assets = match kind {
        Kind::YtDlp => vec![ytdlp_asset(os, arch).ok_or_else(unsupported)?],
        Kind::Ffmpeg => ffmpeg_assets(os, arch).ok_or_else(unsupported)?,
        Kind::Deno => vec![deno_asset(os, arch).ok_or_else(unsupported)?],
    };

    std::fs::create_dir_all(&layout.root)?;
    let staging = layout.root.join(format!(".{}-{}", kind.dir_name(), uuid::Uuid::new_v4().simple()));
    std::fs::create_dir_all(&staging)?;
    let result = install_into(client, &staging, kind, &assets, &mut on_progress).await;
    let installed = match result {
        Ok(()) => swap_in(layout, kind, &staging),
        Err(error) => Err(error),
    };
    if staging.exists() {
        let _ = std::fs::remove_dir_all(&staging);
    }
    installed?;
    let path = match kind {
        Kind::YtDlp => layout.ytdlp(),
        Kind::Ffmpeg => layout.dir(Kind::Ffmpeg),
        Kind::Deno => layout.deno(),
    };
    Ok(path)
}

async fn install_into(
    client: &reqwest::Client,
    staging: &Path,
    kind: Kind,
    assets: &[Asset],
    on_progress: &mut impl FnMut(u64, Option<u64>),
) -> Result<()> {
    let mut done_bytes = 0u64;
    for asset in assets {
        let expected = match &asset.checksum_url {
            Some(url) => {
                let text = fetch_text(client, url).await?;
                Some(parse_checksum(&text, &asset.checksum_name).ok_or_else(|| {
                    Error::msg(format!("No checksum published for {}", asset.checksum_name))
                })?)
            }
            None => None,
        };
        let download = staging.join(format!("download-{}", asset.checksum_name));
        let base = done_bytes;
        let actual = download_to(client, &asset.url, &download, |done, total| {
            on_progress(base + done, total.map(|t| base + t));
        })
        .await?;
        done_bytes += std::fs::metadata(&download)?.len();
        if let Some(expected) = expected {
            if expected != actual {
                return Err(Error::msg(format!("Checksum mismatch for {}", asset.checksum_name)));
            }
        }

        match asset.format {
            ArchiveFormat::Binary => {
                std::fs::rename(&download, staging.join(exe("yt-dlp")))?;
                make_executable(&staging.join(exe("yt-dlp")))?;
            }
            format => {
                let wanted: Box<dyn Fn(&str, &str) -> bool> = match kind {
                    // Windows shared builds keep ffmpeg.exe, ffprobe.exe and their DLLs in bin/
                    Kind::Ffmpeg => Box::new(|full: &str, name: &str| {
                        let in_bin = full.contains("/bin/") || !full.contains('/');
                        in_bin && (name == exe("ffmpeg") || name == exe("ffprobe") || name.ends_with(".dll"))
                            || name == "LICENSE.txt"
                    }),
                    Kind::Deno => Box::new(|_: &str, name: &str| name == exe("deno")),
                    Kind::YtDlp => unreachable!(),
                };
                extract(&download, format, staging, wanted.as_ref())?;
                std::fs::remove_file(&download)?;
            }
        }
    }

    let probe_path = match kind {
        Kind::YtDlp => staging.join(exe("yt-dlp")),
        Kind::Ffmpeg => staging.to_path_buf(),
        Kind::Deno => staging.join(exe("deno")),
    };
    let (binary, arg) = version_binary(kind, &probe_path);
    if !binary.is_file() {
        return Err(Error::msg(format!("{} archive did not contain {}", kind.label(), binary.display())));
    }
    if kind == Kind::Ffmpeg && !staging.join(exe("ffprobe")).is_file() {
        return Err(Error::msg("FFmpeg archive did not contain ffprobe"));
    }
    probe_version(&binary, arg).await?;
    Ok(())
}

/// Replace the installed directory with the staged one.
///
/// The old directory is renamed aside first; on Windows a running yt-dlp.exe can be
/// renamed but not deleted, so leftovers are cleaned up on the next start.
fn swap_in(layout: &Layout, kind: Kind, staging: &Path) -> Result<()> {
    let target = layout.dir(kind);
    if target.exists() {
        let old = layout.root.join(format!(".old-{}-{}", kind.dir_name(), uuid::Uuid::new_v4().simple()));
        std::fs::rename(&target, &old)?;
        let _ = std::fs::remove_dir_all(&old);
    }
    std::fs::rename(staging, &target)?;
    Ok(())
}

/// Remove staging and replaced directories left behind by an earlier run.
pub fn clean_leftovers(layout: &Layout) {
    let Ok(entries) = std::fs::read_dir(&layout.root) else { return };
    for entry in entries.flatten() {
        if entry.file_name().to_string_lossy().starts_with('.') {
            let _ = std::fs::remove_dir_all(entry.path());
        }
    }
}

/// Whether the latest yt-dlp release differs from the installed binary.
pub async fn ytdlp_update_available(client: &reqwest::Client, layout: &Layout) -> Result<bool> {
    let asset = ytdlp_asset(std::env::consts::OS, std::env::consts::ARCH)
        .ok_or_else(|| Error::msg("yt-dlp updates are not available on this platform"))?;
    let sums = fetch_text(client, asset.checksum_url.as_deref().unwrap_or_default()).await?;
    let latest = parse_checksum(&sums, &asset.checksum_name)
        .ok_or_else(|| Error::msg("The latest yt-dlp release has no checksum for this platform"))?;
    let installed = sha256_file(&layout.ytdlp())?;
    Ok(installed != latest)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ytdlp_assets_cover_supported_platforms() {
        assert_eq!(ytdlp_asset_name("windows", "x86_64"), Some("yt-dlp.exe"));
        assert_eq!(ytdlp_asset_name("macos", "aarch64"), Some("yt-dlp_macos"));
        assert_eq!(ytdlp_asset_name("linux", "aarch64"), Some("yt-dlp_linux_aarch64"));
        assert_eq!(ytdlp_asset_name("freebsd", "x86_64"), None);
        let asset = ytdlp_asset("linux", "x86_64").unwrap();
        assert!(asset.url.ends_with("/yt-dlp_linux"));
        assert!(asset.checksum_url.unwrap().ends_with("/SHA2-256SUMS"));
    }

    #[test]
    fn ffmpeg_assets_per_platform() {
        let windows = ffmpeg_assets("windows", "x86_64").unwrap();
        assert_eq!(windows.len(), 1);
        assert!(windows[0].url.ends_with("win64-gpl-shared.zip"));
        let linux = ffmpeg_assets("linux", "aarch64").unwrap();
        assert_eq!(linux[0].format, ArchiveFormat::TarXz);
        let mac = ffmpeg_assets("macos", "aarch64").unwrap();
        assert_eq!(mac.len(), 2);
        assert!(mac[1].url.ends_with("/arm64/release/ffprobe.zip"));
        assert!(ffmpeg_assets("linux", "riscv64").is_none());
    }

    #[test]
    fn deno_assets_per_platform() {
        assert!(deno_asset("windows", "x86_64").unwrap().url.ends_with("deno-x86_64-pc-windows-msvc.zip"));
        assert!(deno_asset("macos", "aarch64").unwrap().url.ends_with("deno-aarch64-apple-darwin.zip"));
        assert!(deno_asset("linux", "x86").is_none());
    }

    #[test]
    fn parses_sha256sum_and_powershell_formats() {
        let sums = "1fa6733c37ea6fb51c99ad8fe785e7b7e5f3246c9b980230329d4fb72ed8d4d6  yt-dlp\n\
                    66674953fe251b89f4d08c5f0e35e0728679bd67ab3d7d05c0562af101dd3e7a  yt-dlp.exe\n";
        assert_eq!(
            parse_checksum(sums, "yt-dlp.exe").as_deref(),
            Some("66674953fe251b89f4d08c5f0e35e0728679bd67ab3d7d05c0562af101dd3e7a")
        );
        assert_eq!(parse_checksum(sums, "yt-dlp_macos"), None);

        let powershell = "\r\nAlgorithm : SHA256\r\nHash      : A0C3101B4158D1DFB7D6A78A7BF0F3DE80C96BB423C152BEEC8BEB22786F2238\r\n\
                          Path      : C:\\a\\deno\\deno\\target\\release\\deno-x86_64-pc-windows-msvc.zip\r\n";
        assert_eq!(
            parse_checksum(powershell, "deno-x86_64-pc-windows-msvc.zip").as_deref(),
            Some("a0c3101b4158d1dfb7d6a78a7bf0f3de80c96bb423c152beec8beb22786f2238")
        );
    }

    #[test]
    fn shortens_versions() {
        assert_eq!(short_version(Kind::Ffmpeg, "ffmpeg version N-127617-g94cba3150c-20261017 Copyright (c)"), "N-127617-g94cba3150c-20261017");
        assert_eq!(short_version(Kind::Deno, "deno 2.9.7 (stable, release, x86_64-unknown-linux-gnu)"), "2.9.7");
        assert_eq!(short_version(Kind::YtDlp, "2026.08.19\n"), "2026.08.19");
    }

    #[test]
    fn extracts_wanted_files_from_zip() {
        let dir = tempfile::tempdir().unwrap();
        let archive = dir.path().join("a.zip");
        {
            let mut writer = zip::ZipWriter::new(std::fs::File::create(&archive).unwrap());
            let options = zip::write::SimpleFileOptions::default();
            for name in ["ffmpeg-x/bin/ffmpeg", "ffmpeg-x/bin/ffprobe", "ffmpeg-x/doc/readme.txt"] {
                writer.start_file(name, options).unwrap();
                writer.write_all(b"data").unwrap();
            }
            writer.finish().unwrap();
        }
        let out = dir.path().join("out");
        std::fs::create_dir(&out).unwrap();
        let mut names = extract(&archive, ArchiveFormat::Zip, &out, &|full, _| full.contains("/bin/")).unwrap();
        names.sort();
        assert_eq!(names, ["ffmpeg", "ffprobe"]);
    }
}
