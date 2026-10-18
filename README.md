# StreamQ

StreamQ is a desktop app for queuing and downloading YouTube video (MP4) and audio (MP3). Version 2 is built with [Tauri 2](https://tauri.app): a Rust core drives the official [yt-dlp](https://github.com/yt-dlp/yt-dlp) binary, and a React interface gives it a clean, light design.

## Features

- Queue many links at once: paste into the field, press Ctrl/⌘+V anywhere, or drop a link onto the window
- MP4 video up to 2160p (4K) or MP3 audio up to 320 kbps, chosen per item
- 1–3 parallel downloads with live progress, speed and time left
- Pause, resume, cancel and retry; cancelled downloads keep their partial file
- Playlists can be downloaded whole or expanded into separate items
- The queue is saved, so it survives restarting the app
- Failed downloads show a readable reason plus the full yt-dlp log
- yt-dlp updates itself automatically, so YouTube changes are fixed without reinstalling StreamQ
- Native installers and a desktop notification when downloads finish

## Download

Get the latest version from the [Releases page](https://github.com/ivanerror/streamq/releases/latest):

| Platform | File | Notes |
| --- | --- | --- |
| Windows 10/11 | `StreamQ_<version>_x64-setup.exe` (or `.msi`) | Installs for the current user, no admin rights needed. The build is unsigned: if SmartScreen warns, choose *More info* → *Run anyway*. |
| macOS 11+ (Intel and Apple Silicon) | `StreamQ_<version>_universal.dmg` | Unsigned: right-click `StreamQ.app` → *Open* the first time. |
| Linux | `StreamQ_<version>_amd64.AppImage` or `.deb` | AppImage: `chmod +x` and run. Debian/Ubuntu: `sudo apt install ./StreamQ_<version>_amd64.deb`. |

Installers stay small (the Linux `.deb` is about 4 MB) because they contain only StreamQ itself. On first launch StreamQ downloads the tools it needs and shows their progress:

| Tool | Purpose | Source | Approx. size |
| --- | --- | --- | --- |
| yt-dlp | Download engine | [yt-dlp releases](https://github.com/yt-dlp/yt-dlp/releases), SHA-256 verified | 18–40 MB |
| FFmpeg + FFprobe | Merge video/audio, convert to MP3 | [yt-dlp/FFmpeg-Builds](https://github.com/yt-dlp/FFmpeg-Builds) (Windows, Linux, SHA-256 verified), [martin-riedl.de](https://ffmpeg.martin-riedl.de) (macOS) | 90–160 MB |
| Deno | Solves YouTube's JavaScript checks | [Deno releases](https://github.com/denoland/deno/releases), SHA-256 verified | ~45 MB |

FFmpeg or Deno already installed on the system (for example with `apt`, `brew` or `winget`) is used instead of downloading. You can add links while setup runs; downloads start once it finishes.

### Where files go

- Downloads: your `Downloads/StreamQ` folder (change it in *Settings*)
- Queue and settings: the app data folder (`%APPDATA%\io.github.ivanerror.streamq` on Windows, `~/Library/Application Support/io.github.ivanerror.streamq` on macOS, `~/.local/share/io.github.ivanerror.streamq` on Linux)
- Downloaded tools: the `components` folder in the app's local data folder

## Development

The app lives in [`desktop/`](desktop):

```
desktop/
  src/                 React + TypeScript UI (Vite)
  src-tauri/
    src/components.rs  First-launch download, verification and updates of yt-dlp, FFmpeg, Deno
    src/engine.rs      yt-dlp command line, JSON progress parsing, process-tree cancel
    src/queue.rs       Queue items, state transitions, persistence
    src/app.rs         Parallel scheduler, events to the UI, notifications
    src/settings.rs    Settings and their validation
    src/lib.rs         Tauri commands and startup
```

Prerequisites: Node.js 22+, Rust (stable) and the [Tauri system dependencies](https://tauri.app/start/prerequisites/) for your OS (on Linux: `libwebkit2gtk-4.1-dev`, `librsvg2-dev`, `libayatana-appindicator3-dev`, `libxdo-dev`).

```bash
cd desktop
npm install
npm run tauri dev        # run with hot reload
npm run tauri build      # build installers for this OS into src-tauri/target/release/bundle
```

### Tests

```bash
cd desktop
npm run typecheck && npm test           # UI types and helpers
cd src-tauri
cargo test                              # Rust unit tests
cargo test -- --ignored installs_all_components   # real first-launch downloads (~200 MB)
STREAMQ_YTDLP=/path/to/yt-dlp cargo test integration_tests -- --ignored --skip installs_all_components
```

The ignored tests drive the real yt-dlp binary against a local HTTP server (progress, cancel, errors) and install every component exactly like a first launch. The **Desktop (Tauri)** workflow runs all of them on Windows, macOS and Linux.

### Releasing

1. Update `version` in `desktop/src-tauri/tauri.conf.json`, `desktop/src-tauri/Cargo.toml` and `desktop/package.json`, and add a `CHANGELOG.md` entry
2. Start the **Release (2.x desktop)** workflow, either way:
   - *Actions → Release (2.x desktop) → Run workflow* on `main` (the tag `v<version>` is created when the release is published), or
   - push a tag: `git tag v2.0.0 && git push origin v2.0.0`
3. The workflow builds the Windows, macOS and Linux installers, then publishes the release with `SHA256SUMS.txt` and the `version.json` that *Check for app update* reads

A pushed tag must match the version in `tauri.conf.json`, and an already published version is never overwritten. Don't create the release by hand on GitHub; the workflow creates it (and reuses its own draft if a run is restarted).

## StreamQ 1.x (Python, legacy)

The original Tkinter app is still in this repository (`src/streamq`, `main.py`) and keeps working until 2.x has replaced it everywhere. It needs Python 3.10+:

```bash
python -m venv .venv && source .venv/bin/activate   # Windows: .venv\Scripts\activate
pip install -r requirements.txt
python main.py
```

On Windows, `run.bat` creates the virtual environment and starts the app without a console (`--console` shows logs, `--update` reinstalls dependencies). Run its tests with `pytest`.

1.x release builds use PyInstaller (`pyinstaller --noconfirm --clean StreamQ.spec`, or `scripts\build_exe.bat` plus `iscc /DAppVersion=1.1.0 installer\streamq.iss` for the Windows installer). Pushing a `v1.*` tag runs the **Release (1.x Python)** workflow. FFmpeg is not bundled in 1.x either: it is downloaded on first launch on Windows and Linux, and taken from Homebrew on macOS (`brew install ffmpeg`).

## Troubleshooting

**Setup could not finish** — check the internet connection and click *Try again*. Corporate networks may block GitHub downloads; installing FFmpeg and Deno system-wide lets StreamQ skip them.

**A YouTube download fails** — open *Settings → Download engine → Check now* to get the newest yt-dlp, then retry. *View log* on the failed item shows yt-dlp's full output.

**macOS says the app is damaged or from an unidentified developer** — the build is not notarized; right-click the app and choose *Open*, or run `xattr -dr com.apple.quarantine /Applications/StreamQ.app`.

## License

StreamQ is MIT licensed (see `LICENSE`). yt-dlp (Unlicense), FFmpeg (GPL) and Deno (MIT) are separate programs downloaded from their publishers; see *Settings → Licenses & notices*.
