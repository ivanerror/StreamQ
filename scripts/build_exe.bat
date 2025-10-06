@echo off
setlocal ENABLEDELAYEDEXPANSION

rem Build StreamQ Windows .exe using PyInstaller
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

rem Build
set "ICON=assets\streamq.ico"
set "ICON_ARG="
if exist "%ICON%" set "ICON_ARG=--icon \"%ICON%\""

pyinstaller ^
  --noconfirm ^
  --clean ^
  --name "StreamQ" ^
  --windowed ^
  %ICON_ARG% ^
  --collect-all yt_dlp ^
  --collect-all ttkbootstrap ^
  --collect-submodules pycryptodomex ^
  main.py

echo.
echo Build complete. See dist\StreamQ\StreamQ.exe
popd
endlocal
