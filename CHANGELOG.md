# Changelog

All notable changes to StreamQ will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.0.0] - 2026-10-18

StreamQ rebuilt with Tauri 2 (`desktop/`), following the architecture decision record
reviewed in the StreamQ Modernization design canvas. The Python 1.x app stays in the
repository as the legacy version.

### Added
- New light-only interface (React + TypeScript): sidebar with Queue / Completed / Failed,
  rich queue rows with thumbnails, inline progress, speed and time left, Settings screen
- Format and quality per item (MP4 up to 2160p, MP3 up to 320 kbps)
- 1–3 parallel downloads; Pause all / Resume all, Cancel all, Retry all
- Queue saved to disk and restored on start (interrupted downloads resume)
- Playlists: download whole into a folder, or expand into separate items
- Paste anywhere (Ctrl/⌘+V), drag-and-drop links, optional clipboard suggestions
- "View log" with yt-dlp's full output for failed downloads; readable error messages
- Desktop notification when a download finishes while StreamQ is in the background
- Automatic yt-dlp updates (SHA-256 verified) plus "Check now", and "Check for app update"
- Licenses & notices screen

### Changed
- yt-dlp now runs as its official standalone binary; Python is no longer shipped
- yt-dlp, FFmpeg and Deno are downloaded on first launch (with progress) instead of being
  bundled, so installers stay small; system-installed FFmpeg/Deno are used when present
- Cancel stops yt-dlp together with the processes it started (process group / Job Object)
- Installers: NSIS + MSI (Windows, per-user), universal .app/.dmg (macOS), AppImage + deb (Linux)

## [1.1.0] - 2026-10-18

### Added
- **Ready-to-run downloads**: a Release workflow builds `StreamQ-Setup-windows-x64.exe`
  (installer), `StreamQ-windows-x64-portable.zip`, `StreamQ-macos-arm64.zip` and
  `StreamQ-linux-x64.tar.gz` (plus `SHA256SUMS.txt`) and publishes them as a GitHub
  Release when a `v*` tag is pushed
- FFmpeg is **not** bundled, to keep downloads small: it is downloaded automatically
  on first launch (Windows/Linux) or taken from the system `PATH`/Homebrew
- **Cancel** button to stop the running download; the current item becomes *Cancelled*
  and the remaining items stay *Pending*
- **Retry Failed** button to re-queue failed/cancelled items (partial downloads resume)
- Video quality options **1440p (2K)** and **2160p (4K)**; default video quality is now 1080p
- FFmpeg is downloaded automatically on Linux (x64/arm64) too
- Unit tests and a GitHub Actions test workflow (Ubuntu/Windows, Python 3.10/3.13)

### Fixed
- Frozen builds crashed at startup because `PIL._tkinter_finder` (used by ttkbootstrap) was not bundled
- The window no longer freezes on first launch while FFmpeg/Deno are downloaded; setup runs in the background
- FFmpeg installed with Homebrew is found even when the macOS app is launched from Finder
- Opening the download folder no longer raises an error when `xdg-open` is missing
- The app icon is loaded from the bundle in frozen builds
- YouTube support restored for current yt-dlp: a JavaScript runtime (Deno) is now
  detected or downloaded automatically and `yt-dlp-ejs` is installed via `yt-dlp[default]`
- FFmpeg installed system-wide (`PATH`) is now used on macOS/Linux instead of failing at startup
- Audio downloads no longer fall back to a full video stream when no audio-only format matches the bitrate
- Video downloads prefer H.264/AAC so the merged MP4 plays in common players
- Fetching the title of a playlist URL no longer resolves every entry
- "Paste & Add" is disabled while downloads run, like the other inputs
- PyInstaller builds bundle `yt-dlp-ejs` and use the correct `Cryptodome` module name

### Changed
- Minimum Python version is now 3.10 (required by yt-dlp)
- Dependencies updated to `yt-dlp[default]>=2026.8.19`
- `streamq.StreamQApp` is imported lazily so core modules work without tkinter
- Removed committed `streamq.egg-info` build metadata
- One PyInstaller spec (`StreamQ.spec`) is used by CI and `scripts/build_exe.bat`;
  builds are one-folder (fast start-up, nothing unpacked per launch) and skip UPX
  (fewer antivirus false positives)
- Replaced the Windows-only build workflow with the cross-platform Release workflow

## [1.0.0] - 2025-01-05

### Added
- Complete project restructuring following Python best practices
- Modern `src/` package layout for better organization
- Modular architecture with separated concerns:
  - [`src/streamq/core/app.py`](src/streamq/core/app.py) - Main GUI application
  - [`src/streamq/core/downloader.py`](src/streamq/core/downloader.py) - Download logic and queue management
  - [`src/streamq/utils/ffmpeg.py`](src/streamq/utils/ffmpeg.py) - FFmpeg handling utilities
  - [`src/streamq/config.py`](src/streamq/config.py) - Configuration management
- Proper packaging with [`pyproject.toml`](pyproject.toml) and [`setup.py`](setup.py)
- Multiple entry points for running the application:
  - `python main.py` (legacy compatibility)
  - `python -m streamq` (package module)
  - `streamq` command (after installation)
- Enhanced run scripts:
  - [`run.bat`](run.bat) - Enhanced Windows launcher
  - [`run.sh`](run.sh) - Cross-platform Unix launcher
- Comprehensive documentation updates
- MIT License added
- Development installation support (`pip install -e .`)

### Changed
- Refactored monolithic `main.py` into logical modules
- Improved error handling and user feedback
- Enhanced virtual environment management in run scripts
- Better FFmpeg detection and setup process
- Modernized project structure following Python packaging standards

### Technical Improvements
- Separation of GUI logic from download management
- Centralized configuration system
- Type-safe module organization
- Better import structure and dependency management
- Enhanced cross-platform compatibility
- Improved development workflow support

### Dependencies
- Maintains compatibility with existing `yt-dlp>=2024.8.6` requirement
- Added development dependencies for code quality tools

### Backward Compatibility
- Existing `run.bat` functionality preserved and enhanced
- Original `main.py` entry point maintained for compatibility
- Download directory structure unchanged
- All user-facing features remain identical

## [0.9.0] - Previous Release
### Initial Features
- Basic GUI interface with Tkinter
- YouTube video/audio downloading using yt-dlp
- Queue management system
- FFmpeg integration for Windows
- Progress tracking and status updates
- Automatic download folder opening