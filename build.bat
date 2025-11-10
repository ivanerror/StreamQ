@echo off
setlocal ENABLEDELAYEDEXPANSION

rem StreamQ Standalone Executable Builder
rem This script creates a standalone .exe file using PyInstaller
rem Usage: Run this batch file from the project root directory

echo ========================================
echo StreamQ Standalone Executable Builder
echo ========================================
echo.

rem Check if Python is installed
python --version >nul 2>&1
if %errorlevel% neq 0 (
    echo ERROR: Python is not installed or not in PATH
    echo Please install Python 3.8+ and try again
    pause
    exit /b 1
)

rem Create virtual environment if it doesn't exist
set "VENV_DIR=.venv"
if not exist "%VENV_DIR%\Scripts\python.exe" (
    echo Creating virtual environment...
    python -m venv "%VENV_DIR%"
    if %errorlevel% neq 0 (
        echo ERROR: Failed to create virtual environment
        pause
        exit /b 1
    )
)

rem Activate virtual environment
echo Activating virtual environment...
call "%VENV_DIR%\Scripts\activate.bat"
if %errorlevel% neq 0 (
    echo ERROR: Failed to activate virtual environment
    pause
    exit /b 1
)

rem Upgrade pip and install requirements
echo Installing/upgrading dependencies...
python -m pip install --upgrade pip
pip install -r requirements.txt
pip install pyinstaller

rem Check if icon file exists
set "ICON=assets\streamq.ico"
set "ICON_ARG="
if exist "%ICON%" (
    set "ICON_ARG=--icon "%ICON%""
    echo Using icon: %ICON%
) else (
    echo Warning: Icon file not found at %ICON%
)

rem Clean previous builds
echo Cleaning previous builds...
if exist "dist" rmdir /s /q "dist"
if exist "build" rmdir /s /q "build"
if exist "StreamQ.spec" del "StreamQ.spec"

echo.
echo Building standalone executable...
echo This may take several minutes...
echo.

rem Build executable using PyInstaller
pyinstaller ^
  --noconfirm ^
  --clean ^
  --onefile ^
  --name "StreamQ" ^
  --windowed ^
  %ICON_ARG% ^
  --add-data "src;src" ^
  --collect-all yt_dlp ^
  --collect-all ttkbootstrap ^
  --collect-submodules pycryptodomex ^
  --hidden-import=tkinter ^
  --hidden-import=tkinter.messagebox ^
  --hidden-import=ttkbootstrap ^
  --hidden-import=yt_dlp ^
  --hidden-import=pycryptodomex ^
  main.py

if %errorlevel% neq 0 (
    echo.
    echo ERROR: Build failed!
    echo Check the output above for error details.
    pause
    exit /b 1
)

echo.
echo ========================================
echo BUILD COMPLETED SUCCESSFULLY!
echo ========================================
echo.
echo Executable location: dist\StreamQ.exe
echo.
echo You can now distribute the StreamQ.exe file
echo It contains all dependencies and runs standalone
echo.

rem Check if executable was created
if exist "dist\StreamQ.exe" (
    echo File size: 
    dir "dist\StreamQ.exe" | findstr StreamQ.exe
    echo.
    echo Do you want to run the executable now? (Y/N)
    set /p "choice=Enter your choice: "
    if /i "!choice!"=="Y" (
        echo Starting StreamQ.exe...
        start "" "dist\StreamQ.exe"
    )
) else (
    echo ERROR: Executable was not created!
)

echo.
pause
endlocal