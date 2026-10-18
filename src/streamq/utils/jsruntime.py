"""JavaScript runtime utilities for StreamQ.

yt-dlp needs an external JavaScript runtime (Deno recommended) together with
the ``yt-dlp-ejs`` package for full YouTube support. Without it, many formats
are missing or downloads fail outright.
"""

import os
import platform
import shutil
import stat
import urllib.request
import zipfile
from urllib.error import URLError

DENO_RELEASE_URL = "https://github.com/denoland/deno/releases/latest/download/{}.zip"

# Runtimes supported by yt-dlp, in its order of priority, mapped to executable names
SUPPORTED_RUNTIMES = (
    ("deno", "deno"),
    ("node", "node"),
    ("quickjs", "qjs"),
    ("bun", "bun"),
)


def resolve_deno_asset():
    """Resolve the Deno release asset name for the current platform."""
    system = platform.system()
    machine = platform.machine().lower()

    if machine in {"amd64", "x86_64"}:
        arch = "x86_64"
    elif machine in {"arm64", "aarch64"}:
        arch = "aarch64"
    else:
        return None

    if system == "Windows":
        return f"deno-{arch}-pc-windows-msvc"
    if system == "Linux":
        return f"deno-{arch}-unknown-linux-gnu"
    if system == "Darwin":
        return f"deno-{arch}-apple-darwin"
    return None


def get_runtime_dir():
    """Get the directory where StreamQ keeps its bundled JavaScript runtime."""
    from ..config import get_project_root

    return os.path.join(get_project_root(), "js_runtime")


def find_js_runtimes():
    """
    Find available JavaScript runtimes without downloading anything.

    Returns:
        dict: yt-dlp ``js_runtimes`` option, empty if no runtime was found
    """
    deno_binary = "deno.exe" if os.name == "nt" else "deno"
    bundled_deno = os.path.join(get_runtime_dir(), deno_binary)
    if os.path.isfile(bundled_deno):
        return {"deno": {"path": bundled_deno}}

    for runtime, executable in SUPPORTED_RUNTIMES:
        path = shutil.which(executable)
        if path:
            return {runtime: {"path": path}}
    return {}


def download_deno():
    """
    Download Deno into the StreamQ runtime directory.

    Returns:
        str: Path to the Deno executable

    Raises:
        RuntimeError: If Deno cannot be downloaded for this platform
    """
    asset = resolve_deno_asset()
    if not asset:
        raise RuntimeError(
            "Automatic Deno download is not available for this platform. "
            "Please install Deno (https://deno.com) and add it to PATH."
        )

    runtime_dir = get_runtime_dir()
    os.makedirs(runtime_dir, exist_ok=True)
    deno_binary = "deno.exe" if os.name == "nt" else "deno"
    deno_path = os.path.join(runtime_dir, deno_binary)
    archive_path = os.path.join(runtime_dir, "deno_download.zip")

    try:
        with urllib.request.urlopen(DENO_RELEASE_URL.format(asset)) as response, open(archive_path, "wb") as archive_file:
            shutil.copyfileobj(response, archive_file)
    except URLError as error:
        raise RuntimeError(f"Failed to download Deno: {error}") from error

    try:
        with zipfile.ZipFile(archive_path) as archive:
            member = next(
                (name for name in archive.namelist() if os.path.basename(name) == deno_binary),
                None,
            )
            if not member:
                raise RuntimeError("Deno archive does not contain the deno executable.")
            with archive.open(member) as source, open(deno_path, "wb") as target:
                shutil.copyfileobj(source, target)
    except zipfile.BadZipFile as error:
        raise RuntimeError("Deno archive is corrupted or invalid.") from error
    finally:
        if os.path.exists(archive_path):
            os.remove(archive_path)

    if os.name != "nt":
        mode = os.stat(deno_path).st_mode
        os.chmod(deno_path, mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)
    return deno_path


def ensure_js_runtimes():
    """
    Ensure a JavaScript runtime is available, downloading Deno if needed.

    Returns:
        dict: yt-dlp ``js_runtimes`` option

    Raises:
        RuntimeError: If no runtime is found and Deno cannot be downloaded
    """
    runtimes = find_js_runtimes()
    if runtimes:
        return runtimes
    return {"deno": {"path": download_deno()}}
