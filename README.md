# StreamQ

StreamQ is a modern desktop application for queuing and downloading video/audio from YouTube using `yt-dlp`, wrapped in a clean Tkinter-based GUI. The application follows Python best practices with a modular architecture for maintainability and extensibility. The UI uses ttkbootstrap for a polished, modern look.

## Features

- Modern GUI with ttkbootstrap (Bootstrap-like themes)
- Queue multiple YouTube downloads
- Audio downloads (MP3) and Video downloads (MP4)
- Video quality up to 2160p (4K) and 1440p (2K)
- Cancel a running download; retry failed or cancelled items with one click
- Automatic FFmpeg setup on Windows and Linux (uses a system FFmpeg from `PATH` when available)
- Automatic JavaScript runtime (Deno) setup, required by yt-dlp for full YouTube support
- Real-time progress (percentage + speed; ETA hidden for cleaner status)
- Background title fetching for queued items
- Single queue table view: columns Status | Link | Title
- Auto-open download folder on completion
- Windows launcher opens without console by default

## Download (no setup needed)

Grab the latest build from the [Releases page](https://github.com/ivanerror/streamq/releases/latest):

| Platform | File | How to run |
| --- | --- | --- |
| Windows 10/11 (x64) | `StreamQ-windows-x64.exe` | Double-click. If SmartScreen warns, click *More info* → *Run anyway* (the build is unsigned). |
| macOS (Apple Silicon) | `StreamQ-macos-arm64.zip` | Unzip, then right-click `StreamQ.app` → *Open* the first time (unsigned app). Install FFmpeg with `brew install ffmpeg`. |
| Linux (x64) | `StreamQ-linux-x64.tar.gz` | `tar -xzf StreamQ-linux-x64.tar.gz && ./StreamQ` |

On first launch StreamQ downloads FFmpeg (Windows/Linux) and the Deno JavaScript runtime in the background; the *Start Download* button becomes available once they are ready. App data and downloads are stored in `~/StreamQ/` (`%USERPROFILE%\StreamQ\` on Windows).

## Run from Source

### Prerequisites

- Python 3.10+ installed (required by current yt-dlp)
- Windows 10/11 recommended; macOS/Linux supported
- Internet connection (for dependencies and FFmpeg download)

### Quick Start

#### Option 1: Windows Convenience Script

The easiest way to start on Windows:

1. Double-click `run.bat` from File Explorer (opens GUI without console), or run in a terminal:
   ```bat
   run.bat
   ```
2. The script automatically:
   - Creates a virtual environment at `.venv/`
   - Installs dependencies from `requirements.txt` only when it changes (cached via SHA-256)
   - Launches the StreamQ GUI using `pythonw` (no console window)

Flags:
- `--console` run with a terminal window and logs
- `--update` force re-install dependencies (refresh the cache)

#### Option 2: Manual Installation (All Platforms)

```bash
# Create and activate virtual environment
python -m venv .venv

# Windows
.venv\Scripts\activate

# macOS/Linux
source .venv/bin/activate

# Install dependencies
python -m pip install --upgrade pip
pip install -r requirements.txt

# Run the application
python main.py
```

#### Option 3: Install as a Package

```bash
# Install in development mode
pip install -e .

# Or install from source
pip install .

# Run from anywhere (console)
streamq

# Or launch without console (GUI script on Windows)
streamq-gui
```

## Build Executables

All platforms use the same PyInstaller spec:

```bash
pip install -r requirements.txt pyinstaller
pyinstaller --noconfirm --clean StreamQ.spec
```

Output: `dist/StreamQ.exe` (Windows), `dist/StreamQ` (Linux) or `dist/StreamQ.app` (macOS). On Windows you can also run `scripts\build_exe.bat`.

### Publishing a Release

1. Bump `__version__` in `src/streamq/__init__.py` (and `version` in `pyproject.toml`/`setup.py`), update `CHANGELOG.md`
2. Tag and push: `git tag v1.2.0 && git push origin v1.2.0`
3. The **Release** workflow runs the tests, builds Windows/macOS/Linux executables and publishes them with `SHA256SUMS.txt` on the Releases page

The workflow fails if the tag does not match `__version__`. Running it manually (*Actions → Release → Run workflow*) only builds downloadable artifacts.

## Usage

1. Launch StreamQ
2. Paste a YouTube URL and click “Add to Queue”
3. Choose Audio (MP3) or Video (MP4)
4. Select quality
5. Click “Start Download”
6. Click “Cancel” to stop; the current item becomes *Cancelled* and the rest stay *Pending*
7. Click “Retry Failed” to re-queue failed/cancelled items (partial downloads are resumed)

Queue display (single table):
- `Status`: Pending / Downloading / Completed / Failed / Cancelled
- `Link`: The original URL you added
- `Title`: Fetched automatically in the background

### Output Locations

- Audio files: `Output/audio/`
- Video files: `Output/video/`
- FFmpeg binaries (Windows/Linux auto-download): `ffmpeg_support/`
- Deno JavaScript runtime (when not found in `PATH`): `js_runtime/`

## Project Structure

```
src/
  streamq/
    __init__.py
    __main__.py        # Module entry point
    config.py          # Configuration management
    utils/
      __init__.py
      ffmpeg.py        # FFmpeg handling utilities
      jsruntime.py     # JavaScript runtime (Deno) detection/download
    core/
      __init__.py
      app.py           # Main GUI application
      downloader.py    # Download logic & queue management

main.py               # Standalone entry point script
pyproject.toml        # Packaging configuration
setup.py              # Legacy/fallback setup
requirements.txt      # Python dependencies
run.bat               # Windows launcher
run.sh                # Linux/macOS helper
LICENSE
README.md
```

## Platform Notes

### Windows
- FFmpeg is automatically downloaded and configured on first run
- ttkbootstrap theme enabled by default (changeable in `main.py` or `src/streamq/__main__.py`)
- `run.bat` launches without console by default; use `--console` for logs

### All platforms
- Deno is downloaded automatically if no JavaScript runtime (deno, node, quickjs, bun) is in `PATH`

### Linux
- FFmpeg from `PATH` is used if installed (`sudo apt install ffmpeg`); otherwise a static build is downloaded on first run (x64/arm64)

### macOS
- Install FFmpeg with `brew install ffmpeg` (Homebrew locations are detected even when launched from Finder)

## Development

```bash
# Clone the repository
git clone https://github.com/ivanerror/streamq.git
cd streamq

# Install in development mode with optional dev tools
pip install -e .[dev]

# Or manually install dev dependencies
pip install pytest black isort flake8 mypy
```

### Running Tests

```bash
pytest
pytest --cov=streamq
```

### Code Quality

```bash
black src/
isort src/
flake8 src/
mypy src/
```

## Troubleshooting

**No GUI or errors at startup**
- Ensure you are using Python 3.10+
- Activate the virtual environment
- Use `run.bat --console` to see logs

**Dependency installation failed**
- Check internet connection
- `python -m pip install --upgrade pip`
- `run.bat --update`

**FFmpeg missing (macOS/Linux)**
- Install FFmpeg and ensure it is in `PATH`

**Download failed**
- Verify YouTube URL
- Update yt-dlp: `pip install -U "yt-dlp[default]"` (or `run.bat --update`)
- YouTube formats missing / "n challenge" or "JS runtime" warnings: make sure Deno is installed or let StreamQ download it into `js_runtime/`

## License

MIT License. See `LICENSE` for details.

---

StreamQ — Simple, reliable YouTube downloading with a modern Python architecture.
