"""FFmpeg utilities for StreamQ."""

import os
import platform
import urllib.request
import zipfile
import shutil
from urllib.error import URLError


def resolve_ffmpeg_url():
    """Resolve the appropriate FFmpeg download URL based on the current platform."""
    system = platform.system()
    machine = platform.machine().lower()

    if system == "Windows":
        if machine in {"amd64", "x86_64"}:
            return "https://github.com/yt-dlp/FFmpeg-Builds/releases/latest/download/ffmpeg-master-latest-win64-gpl.zip"
        if machine in {"arm64", "aarch64"}:
            return "https://github.com/yt-dlp/FFmpeg-Builds/releases/latest/download/ffmpeg-master-latest-winarm64-gpl.zip"
        if machine in {"x86", "i386", "i686"}:
            return "https://github.com/yt-dlp/FFmpeg-Builds/releases/latest/download/ffmpeg-master-latest-win32-gpl.zip"
    return None


def prepend_to_path(path):
    """Add a path to the beginning of the system PATH environment variable."""
    current = os.environ.get("PATH", "")
    entries = [entry for entry in current.split(os.pathsep) if entry]
    if path not in entries:
        os.environ["PATH"] = os.pathsep.join([path] + entries)


def ensure_ffmpeg():
    """
    Ensure FFmpeg is available, downloading it automatically on Windows if needed.
    
    Returns:
        str: Path to the FFmpeg bin directory
        
    Raises:
        RuntimeError: If FFmpeg cannot be found or downloaded
    """
    # Use the project root directory for FFmpeg support
    from ..config import get_project_root
    
    ffmpeg_root = os.path.join(get_project_root(), "ffmpeg_support")
    bin_dir = os.path.join(ffmpeg_root, "bin")
    ffmpeg_binary = "ffmpeg.exe" if os.name == "nt" else "ffmpeg"
    ffprobe_binary = "ffprobe.exe" if os.name == "nt" else "ffprobe"
    ffmpeg_path = os.path.join(bin_dir, ffmpeg_binary)
    ffprobe_path = os.path.join(bin_dir, ffprobe_binary)

    if os.path.isfile(ffmpeg_path) and os.path.isfile(ffprobe_path):
        prepend_to_path(bin_dir)
        return bin_dir

    download_url = resolve_ffmpeg_url()
    if not download_url:
        raise RuntimeError(
            "FFmpeg tidak ditemukan dan unduhan otomatis tidak tersedia untuk platform ini. "
            "Silakan instal FFmpeg secara manual dan tambahkan ke PATH."
        )

    os.makedirs(bin_dir, exist_ok=True)
    archive_path = os.path.join(ffmpeg_root, "ffmpeg_download.zip")

    try:
        with urllib.request.urlopen(download_url) as response, open(archive_path, "wb") as archive_file:
            shutil.copyfileobj(response, archive_file)
    except URLError as error:
        raise RuntimeError(f"Gagal mengunduh FFmpeg: {error}") from error

    try:
        required = {ffmpeg_binary, ffprobe_binary}
        with zipfile.ZipFile(archive_path) as archive:
            available = {os.path.basename(name) for name in archive.namelist() if os.path.basename(name) in required}
            if not required.issubset(available):
                raise RuntimeError("Arsip FFmpeg tidak berisi ffmpeg dan ffprobe.")
            for member in archive.namelist():
                basename = os.path.basename(member)
                if basename in required:
                    target_path = os.path.join(bin_dir, basename)
                    with archive.open(member) as source, open(target_path, "wb") as target:
                        shutil.copyfileobj(source, target)
    except zipfile.BadZipFile as error:
        raise RuntimeError("Arsip FFmpeg rusak atau tidak valid.") from error
    finally:
        if os.path.exists(archive_path):
            os.remove(archive_path)

    prepend_to_path(bin_dir)
    return bin_dir