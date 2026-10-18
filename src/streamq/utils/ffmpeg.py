"""FFmpeg utilities for StreamQ."""

import os
import platform
import urllib.request
import zipfile
import shutil
import stat
import tarfile
from urllib.error import URLError

FFMPEG_RELEASE_URL = "https://github.com/yt-dlp/FFmpeg-Builds/releases/latest/download/{}"

# Apps launched from Finder/desktop menus don't inherit the shell PATH (e.g. Homebrew)
EXTRA_SEARCH_DIRS = ("/opt/homebrew/bin", "/usr/local/bin", "/usr/bin")


def resolve_ffmpeg_url():
    """Resolve the appropriate FFmpeg download URL based on the current platform."""
    system = platform.system()
    machine = platform.machine().lower()

    if system == "Windows":
        if machine in {"amd64", "x86_64"}:
            return FFMPEG_RELEASE_URL.format("ffmpeg-master-latest-win64-gpl.zip")
        if machine in {"arm64", "aarch64"}:
            return FFMPEG_RELEASE_URL.format("ffmpeg-master-latest-winarm64-gpl.zip")
        if machine in {"x86", "i386", "i686"}:
            return FFMPEG_RELEASE_URL.format("ffmpeg-master-latest-win32-gpl.zip")
    if system == "Linux":
        if machine in {"amd64", "x86_64"}:
            return FFMPEG_RELEASE_URL.format("ffmpeg-master-latest-linux64-gpl.tar.xz")
        if machine in {"arm64", "aarch64"}:
            return FFMPEG_RELEASE_URL.format("ffmpeg-master-latest-linuxarm64-gpl.tar.xz")
    return None


def extract_binaries(archive_path, required, bin_dir):
    """Extract the required binaries from a .zip or .tar.xz FFmpeg archive into bin_dir."""
    if zipfile.is_zipfile(archive_path):
        with zipfile.ZipFile(archive_path) as archive:
            members = {
                os.path.basename(name): name
                for name in archive.namelist()
                if os.path.basename(name) in required
            }
            if not required.issubset(members):
                raise RuntimeError("Arsip FFmpeg tidak berisi ffmpeg dan ffprobe.")
            for basename, member in members.items():
                with archive.open(member) as source, open(os.path.join(bin_dir, basename), "wb") as target:
                    shutil.copyfileobj(source, target)
        return

    try:
        with tarfile.open(archive_path) as archive:
            members = {
                os.path.basename(member.name): member
                for member in archive.getmembers()
                if member.isfile() and os.path.basename(member.name) in required
            }
            if not required.issubset(members):
                raise RuntimeError("Arsip FFmpeg tidak berisi ffmpeg dan ffprobe.")
            for basename, member in members.items():
                target_path = os.path.join(bin_dir, basename)
                with archive.extractfile(member) as source, open(target_path, "wb") as target:
                    shutil.copyfileobj(source, target)
                mode = os.stat(target_path).st_mode
                os.chmod(target_path, mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    except tarfile.TarError as error:
        raise RuntimeError("Arsip FFmpeg rusak atau tidak valid.") from error


def prepend_to_path(path):
    """Add a path to the beginning of the system PATH environment variable."""
    current = os.environ.get("PATH", "")
    entries = [entry for entry in current.split(os.pathsep) if entry]
    if path not in entries:
        os.environ["PATH"] = os.pathsep.join([path] + entries)


def ensure_ffmpeg():
    """
    Ensure FFmpeg is available, downloading it automatically on Windows/Linux if needed.
    
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

    # Fall back to a system-wide FFmpeg installation (e.g. apt, brew, winget)
    search_path = os.pathsep.join([os.environ.get("PATH", ""), *EXTRA_SEARCH_DIRS])
    system_ffmpeg = shutil.which("ffmpeg", path=search_path)
    if system_ffmpeg and shutil.which("ffprobe", path=search_path):
        return os.path.dirname(system_ffmpeg)

    download_url = resolve_ffmpeg_url()
    if not download_url:
        raise RuntimeError(
            "FFmpeg tidak ditemukan dan unduhan otomatis tidak tersedia untuk platform ini. "
            "Silakan instal FFmpeg secara manual dan tambahkan ke PATH "
            "(macOS: brew install ffmpeg)."
        )

    os.makedirs(bin_dir, exist_ok=True)
    archive_path = os.path.join(ffmpeg_root, "ffmpeg_download")

    try:
        with urllib.request.urlopen(download_url) as response, open(archive_path, "wb") as archive_file:
            shutil.copyfileobj(response, archive_file)
    except URLError as error:
        raise RuntimeError(f"Gagal mengunduh FFmpeg: {error}") from error

    try:
        extract_binaries(archive_path, {ffmpeg_binary, ffprobe_binary}, bin_dir)
    except zipfile.BadZipFile as error:
        raise RuntimeError("Arsip FFmpeg rusak atau tidak valid.") from error
    finally:
        if os.path.exists(archive_path):
            os.remove(archive_path)

    prepend_to_path(bin_dir)
    return bin_dir