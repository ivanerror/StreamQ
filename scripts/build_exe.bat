@echo off
setlocal ENABLEDELAYEDEXPANSION

rem Build StreamQ Windows .exe using PyInstaller (same spec as the Release workflow)
rem Usage: double-click or run in terminal

pushd "%~dp0\.." >nul 2>&1

set "VENV_DIR=.venv"
if not exist "%VENV_DIR%\Scripts\python.exe" (
  py -3 -m venv "%VENV_DIR%"
)
call "%VENV_DIR%\Scripts\activate.bat"

python -m pip install --upgrade pip
pip install -r requirements.txt
pip install pyinstaller

rem Bundle FFmpeg so the build works without any setup
python scripts\fetch_ffmpeg.py
pyinstaller --noconfirm --clean StreamQ.spec

echo.
echo Build complete. See dist\StreamQ\StreamQ.exe
echo To build the installer: iscc /DAppVersion=x.y.z installer\streamq.iss
popd
endlocal
