# -*- mode: python ; coding: utf-8 -*-
# Build with: pyinstaller --noconfirm --clean StreamQ.spec
# Windows/Linux: single-file executable in dist/; macOS: dist/StreamQ.app
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

icon = [os.path.join(root, 'assets', 'streamq.ico')]
is_mac = sys.platform == 'darwin'

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

if is_mac:
    exe = EXE(
        pyz,
        a.scripts,
        [],
        exclude_binaries=True,
        name='StreamQ',
        console=False,
        upx=False,
        argv_emulation=False,
        icon=icon,
    )
    coll = COLLECT(exe, a.binaries, a.datas, name='StreamQ', upx=False)
    app = BUNDLE(
        coll,
        name='StreamQ.app',
        icon=icon,
        bundle_identifier='io.github.ivanerror.streamq',
        info_plist={'NSHighResolutionCapable': True},
    )
else:
    exe = EXE(
        pyz,
        a.scripts,
        a.binaries,
        a.datas,
        [],
        name='StreamQ',
        debug=False,
        bootloader_ignore_signals=False,
        strip=False,
        # UPX-packed executables trigger more antivirus false positives
        upx=False,
        runtime_tmpdir=None,
        console=False,
        disable_windowed_traceback=False,
        argv_emulation=False,
        target_arch=None,
        codesign_identity=None,
        entitlements_file=None,
        icon=icon,
    )
