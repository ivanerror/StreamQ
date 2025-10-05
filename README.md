# StreamQ

StreamQ is a simple desktop app to queue and download video/audio from YouTube using `yt-dlp`, wrapped in a clean Tkinter-based GUI. On Windows, the app automatically prepares FFmpeg so you can use it right away.

## Prerequisites
- Windows 10/11 recommended; macOS/Linux also supported (see notes below).
- Python 3.8 or newer installed on your system.
- Internet connection (for dependencies and FFmpeg download on Windows).
- Git (optional, for cloning/pushing to GitHub).

## Install & Run (Windows)

The easiest way is to use the included `run.bat` script:

1. Double-click `run.bat` from File Explorer, or run it from a terminal in the project folder:
   ```bat
   run.bat
   ```
2. The script will:
   - Create a virtual environment at `.venv/` (if it doesn’t exist)
   - Install dependencies from `requirements.txt`
   - Launch the GUI app (`python main.py`)

When the StreamQ window opens, paste a YouTube URL, add it to the queue, then start downloading. By default, audio/video outputs are saved under `py_downloader/` (see the `audio` and `video` subfolders).

## Run Manually (Windows/macOS/Linux)

If you prefer not to use `run.bat`, you can run it manually with standard Python steps:

```bash
python -m venv .venv
# Windows
.venv\\Scripts\\activate
# macOS/Linux
# source .venv/bin/activate

python -m pip install --upgrade pip
pip install -r requirements.txt
python main.py
```

FFmpeg notes:
- Windows: the app automatically downloads and sets up FFmpeg into `ffmpeg_support/` on first run.
- macOS/Linux: automatic FFmpeg download is not available. Please install FFmpeg manually and ensure `ffmpeg` and `ffprobe` are in your `PATH` (e.g., `brew install ffmpeg` on macOS or `sudo apt install ffmpeg` on Ubuntu).

## Project Structure (short)
- `main.py` — main StreamQ GUI application code.
- `run.bat` — helper script to create venv, install dependencies, and run the app.
- `requirements.txt` — Python dependencies.
- `py_downloader/` — download outputs (audio/video).
- `ffmpeg_support/` — FFmpeg location (Windows, automatic).
- `.venv/` — local virtual environment (ignored by Git).

## Git Init & Push to GitHub

If you haven’t initialized Git yet and want to push to GitHub:

```bash
# Initialize a local repo (use main as default branch)
git init
git branch -M main

# Add files (large outputs are ignored via .gitignore)
git add .
git commit -m "Initial commit"

# Create a new GitHub repo (via web), then connect it:
git remote add origin https://github.com/<username>/<repo>.git
git push -u origin main
```

Alternative: If you use GitHub CLI (`gh`), you can create and push in one go:

```bash
gh repo create <username>/<repo> --source . --public --push
```

## Troubleshooting
- Python not found: ensure `python --version` shows >= 3.8.
- Dependency install failed: check your internet connection, then rerun `run.bat`/pip commands.
- FFmpeg missing (macOS/Linux): install FFmpeg and ensure it’s on `PATH`.
- Download failed: retry, verify the URL, or update `yt-dlp` (`pip install -U yt-dlp`).
