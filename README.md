# StreamQ

StreamQ is a modern desktop application for queuing and downloading video/audio from YouTube using `yt-dlp`, wrapped in a clean Tkinter-based GUI. The application follows Python best practices with a modular architecture for maintainability and extensibility. The UI uses ttkbootstrap for a polished, modern look.

## Features

- Modern GUI with ttkbootstrap (Bootstrap-like themes)
- Queue multiple YouTube downloads
- Audio downloads (MP3) and Video downloads (MP4)
- Automatic FFmpeg setup on Windows
- Real-time progress (percentage + speed; ETA hidden for cleaner status)
- Background title fetching for queued items
- Single queue table view: columns Status | Link | Title
- Auto-open download folder on completion
- Windows launcher opens without console by default

## Prerequisites

- Python 3.8+ installed
- Windows 10/11 recommended; macOS/Linux supported
- Internet connection (for dependencies and FFmpeg download)

## Quick Start

### Option 1: Windows Convenience Script

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

### Option 2: Manual Installation (All Platforms)

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

### Option 3: Install as a Package

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

## Usage

1. Launch StreamQ
2. Paste a YouTube URL and click “Add to Queue”
3. Choose Audio (MP3) or Video (MP4)
4. Select quality
5. Click “Start Download”

Queue display (single table):
- `Status`: Pending / Downloading / Completed / Failed
- `Link`: The original URL you added
- `Title`: Fetched automatically in the background

### Output Locations

- Audio files: `py_downloader/audio/`
- Video files: `py_downloader/video/`
- FFmpeg binaries (Windows): `ffmpeg_support/`

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

### macOS/Linux
- Install FFmpeg manually:
  - macOS: `brew install ffmpeg`
  - Ubuntu/Debian: `sudo apt install ffmpeg`
- Ensure `ffmpeg` and `ffprobe` are available in `PATH`

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
- Ensure you are using Python 3.8+
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
- Update yt-dlp: `pip install -U yt-dlp`

## License

MIT License. See `LICENSE` for details.

---

StreamQ — Simple, reliable YouTube downloading with a modern Python architecture.

