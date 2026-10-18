# -*- mode: python ; coding: utf-8 -*-
# Build with: pyinstaller --noconfirm --clean StreamQ.spec
# Output: dist/StreamQ/ (Windows/Linux folder) or dist/StreamQ.app (macOS).
# Run scripts/fetch_ffmpeg.py first to bundle FFmpeg; without it the app
# falls back to a system FFmpeg or downloads one on first run.
import os
import sys
from PyInstaller.utils.hooks import collect_all, collect_submodules

root = SPECPATH
datas = [(os.path.join(root, 'assets'), 'assets')]
binaries = []
# ttkbootstrap renders images through PIL.ImageTk, which imports this lazily
hiddenimports = ['tkinter', 'tkinter.messagebox', 'PIL._tkinter_finder'] + collect_submodules('Cryptodome')
# yt-dlp loads extractors lazily and yt-dlp-ejs ships JavaScript files as package data
for package in ('yt_dlp', 'yt_dlp_ejs', 'ttkbootstrap'):
    pkg_datas, pkg_binaries, pkg_hiddenimports = collect_all(package)
    datas += pkg_datas
    binaries += pkg_binaries
    hiddenimports += pkg_hiddenimports

# Bundled FFmpeg lands in <bundle>/ffmpeg, where streamq.utils.ffmpeg looks first
ffmpeg_dir = os.path.join(root, 'vendor', 'ffmpeg')
if os.path.isdir(ffmpeg_dir):
    for name in sorted(os.listdir(ffmpeg_dir)):
        path = os.path.join(ffmpeg_dir, name)
        if name.endswith('.txt'):
            datas.append((path, 'ffmpeg'))
        else:
            binaries.append((path, 'ffmpeg'))
else:
    print('WARNING: vendor/ffmpeg not found; building without bundled FFmpeg')

icon = [os.path.join(root, 'assets', 'streamq.ico')]

a = Analysis(
    [os.path.join(root, 'scripts', 'pyinstaller_entry.py')],
    pathex=[os.path.join(root, 'src')],
    binaries=binaries,
    datas=datas,
    hiddenimports=hiddenimports,
    hookspath=[],
    hooksconfig={},
    runtime_hooks=[],
    excludes=[],
    noarchive=False,
    optimize=0,
)
pyz = PYZ(a.pure)

# One-folder build: starts fast because nothing (FFmpeg included) is unpacked on launch
exe = EXE(
    pyz,
    a.scripts,
    [],
    exclude_binaries=True,
    name='StreamQ',
    debug=False,
    bootloader_ignore_signals=False,
    strip=False,
    # UPX-packed executables trigger more antivirus false positives
    upx=False,
    console=False,
    disable_windowed_traceback=False,
    argv_emulation=False,
    target_arch=None,
    codesign_identity=None,
    entitlements_file=None,
    icon=icon,
)
coll = COLLECT(exe, a.binaries, a.datas, strip=False, upx=False, name='StreamQ')

if sys.platform == 'darwin':
    app = BUNDLE(
        coll,
        name='StreamQ.app',
        icon=icon,
        bundle_identifier='io.github.ivanerror.streamq',
        info_plist={'NSHighResolutionCapable': True},
    )
