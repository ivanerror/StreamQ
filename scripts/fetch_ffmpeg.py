"""Download FFmpeg/FFprobe into vendor/ffmpeg/ so PyInstaller can bundle them.

Used by the Release workflow; run it before ``pyinstaller StreamQ.spec`` to build
a package that works without any FFmpeg setup on the user's machine.
"""

import io
import os
import platform
import shutil
import stat
import sys
import tarfile
import urllib.request
import zipfile

ROOT = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))
TARGET_DIR = os.path.join(ROOT, "vendor", "ffmpeg")
YT_DLP_BUILDS = "https://github.com/yt-dlp/FFmpeg-Builds/releases/latest/download/"
MACOS_BUILDS = "https://ffmpeg.martin-riedl.de/redirect/latest/macos/{arch}/release/{name}.zip"

SOURCE_NOTE = """FFmpeg and FFprobe are bundled unmodified as separate programs.
They are licensed under the GNU GPL; see LICENSE.txt and https://ffmpeg.org/legal.html.
Source code: https://github.com/FFmpeg/FFmpeg
Binary builds: {origin}
"""


def download(url):
    print(f"Downloading {url}")
    with urllib.request.urlopen(url) as response:
        return response.read()


def make_executable(path):
    mode = os.stat(path).st_mode
    os.chmod(path, mode | stat.S_IXUSR | stat.S_IXGRP | stat.S_IXOTH)


def fetch_windows(machine):
    # The "shared" build keeps ffmpeg.exe/ffprobe.exe small by sharing the DLLs
    arch = "winarm64" if machine in {"arm64", "aarch64"} else "win64"
    url = f"{YT_DLP_BUILDS}ffmpeg-master-latest-{arch}-gpl-shared.zip"
    with zipfile.ZipFile(io.BytesIO(download(url))) as archive:
        for name in archive.namelist():
            parts = name.split("/")
            if (len(parts) >= 3 and parts[-2] == "bin" and parts[-1]) or parts[-1] == "LICENSE.txt":
                with archive.open(name) as source, open(os.path.join(TARGET_DIR, parts[-1]), "wb") as target:
                    shutil.copyfileobj(source, target)
    return "https://github.com/yt-dlp/FFmpeg-Builds"


def fetch_linux(machine):
    arch = "linuxarm64" if machine in {"arm64", "aarch64"} else "linux64"
    url = f"{YT_DLP_BUILDS}ffmpeg-master-latest-{arch}-gpl.tar.xz"
    wanted = {"ffmpeg", "ffprobe", "LICENSE.txt"}
    with tarfile.open(fileobj=io.BytesIO(download(url)), mode="r:xz") as archive:
        for member in archive.getmembers():
            basename = os.path.basename(member.name)
            if member.isfile() and basename in wanted:
                target_path = os.path.join(TARGET_DIR, basename)
                with archive.extractfile(member) as source, open(target_path, "wb") as target:
                    shutil.copyfileobj(source, target)
                if basename != "LICENSE.txt":
                    make_executable(target_path)
    return "https://github.com/yt-dlp/FFmpeg-Builds"


def fetch_macos(machine):
    arch = "arm64" if machine in {"arm64", "aarch64"} else "amd64"
    for name in ("ffmpeg", "ffprobe"):
        with zipfile.ZipFile(io.BytesIO(download(MACOS_BUILDS.format(arch=arch, name=name)))) as archive:
            member = next(n for n in archive.namelist() if os.path.basename(n) == name)
            target_path = os.path.join(TARGET_DIR, name)
            with archive.open(member) as source, open(target_path, "wb") as target:
                shutil.copyfileobj(source, target)
            make_executable(target_path)
    return "https://ffmpeg.martin-riedl.de"


def main():
    machine = platform.machine().lower()
    shutil.rmtree(TARGET_DIR, ignore_errors=True)
    os.makedirs(TARGET_DIR)

    if sys.platform == "win32":
        origin = fetch_windows(machine)
    elif sys.platform == "darwin":
        origin = fetch_macos(machine)
    elif sys.platform.startswith("linux"):
        origin = fetch_linux(machine)
    else:
        sys.exit(f"Unsupported platform: {sys.platform}")

    with open(os.path.join(TARGET_DIR, "FFMPEG-SOURCE.txt"), "w", encoding="utf-8") as note:
        note.write(SOURCE_NOTE.format(origin=origin))

    exe = ".exe" if sys.platform == "win32" else ""
    for name in ("ffmpeg", "ffprobe"):
        if not os.path.isfile(os.path.join(TARGET_DIR, name + exe)):
            sys.exit(f"{name}{exe} missing from the downloaded FFmpeg build")
    print(f"FFmpeg ready in {TARGET_DIR}: {sorted(os.listdir(TARGET_DIR))}")


if __name__ == "__main__":
    main()
