@echo off
REM StreamQ - Enhanced launch script with support for new project structure
REM Copyright (c) 2025 ivanerror (https://github.com/ivanerror)
REM All rights reserved.
setlocal ENABLEDELAYEDEXPANSION

REM Parse arguments
set "GUI_MODE=1"
set "FORCE_UPDATE="
for %%A in (%*) do (
  if /I "%%~A"=="--console" set "GUI_MODE="
  if /I "%%~A"=="/console" set "GUI_MODE="
  if /I "%%~A"=="--update" set "FORCE_UPDATE=1"
)

if not defined GUI_MODE (
  echo ========================================
  echo           StreamQ Launcher
  echo ========================================
  echo.
)

:: Ensure we're running from the script's directory
pushd "%~dp0" >nul 2>&1
if errorlevel 1 (
    echo ERROR: Failed to access script directory.
    pause
    exit /b 1
)

:: Find Python interpreter
set "PYTHON_CMD="
if not defined GUI_MODE echo [1/5] Looking for Python interpreter...

:: Try python command first
where python >nul 2>&1
if not errorlevel 1 (
    python --version 2>&1 | findstr /C:"Python 3" >nul
    if not errorlevel 1 (
        set "PYTHON_CMD=python"
        if not defined GUI_MODE echo       Found: python
    )
)

:: Try py launcher if python not found
if not defined PYTHON_CMD (
    py -3 --version >nul 2>&1
    if not errorlevel 1 (
        set "PYTHON_CMD=py -3"
        if not defined GUI_MODE echo       Found: py -3 launcher
    )
)

:: Error if no Python found
if not defined PYTHON_CMD (
    echo       ERROR: Python 3.8+ not found!
    echo       Please install Python 3.8 or later and ensure it's in your PATH.
    echo       Download from: https://www.python.org/downloads/
    popd
    pause
    exit /b 1
)

:: Check Python version
%PYTHON_CMD% -c "import sys; exit(0 if sys.version_info >= (3, 8) else 1)" 2>nul
if errorlevel 1 (
    echo       ERROR: Python 3.8 or later is required.
    %PYTHON_CMD% --version
    popd
    pause
    exit /b 1
)

:: Setup virtual environment
set "VENV_DIR=.venv"
if not defined GUI_MODE echo [2/5] Setting up virtual environment...

if not exist "%VENV_DIR%\Scripts\python.exe" (
    if not defined GUI_MODE echo       Creating virtual environment...
    %PYTHON_CMD% -m venv "%VENV_DIR%"
    if errorlevel 1 (
        echo       ERROR: Failed to create virtual environment.
        popd
        pause
        exit /b 1
    )
    if not defined GUI_MODE echo       Virtual environment created successfully.
) else (
    if not defined GUI_MODE echo       Virtual environment already exists.
)

:: Activate virtual environment
if not defined GUI_MODE echo [3/5] Activating virtual environment...
call "%VENV_DIR%\Scripts\activate.bat"
if errorlevel 1 (
    echo       ERROR: Failed to activate virtual environment.
    popd
    pause
    exit /b 1
)
if not defined GUI_MODE echo       Virtual environment activated.

:: Install/update dependencies
if not defined GUI_MODE echo [4/5] Installing dependencies...
set "REQUIREMENTS_FILE=requirements.txt"
set "REQ_HASH_FILE=%VENV_DIR%\.req.sha256"
if exist "%REQUIREMENTS_FILE%" (
    set "NEED_INSTALL="
    if defined FORCE_UPDATE (
        set "NEED_INSTALL=1"
    ) else (
        rem Compute current requirements hash (SHA256)
        set "CUR_HASH="
        for /f "tokens=* delims=" %%H in ('certutil -hashfile "%REQUIREMENTS_FILE%" SHA256 ^| findstr /R "^[0-9A-F]"') do (
            if not defined CUR_HASH set "CUR_HASH=%%H"
        )
        if defined CUR_HASH set "CUR_HASH=!CUR_HASH: =!"
        set "OLD_HASH="
        if exist "%REQ_HASH_FILE%" (
            set /p OLD_HASH=<"%REQ_HASH_FILE%"
        )
        if not defined CUR_HASH (
            set "NEED_INSTALL=1"
        ) else if not defined OLD_HASH (
            set "NEED_INSTALL=1"
        ) else if /I not "!CUR_HASH!"=="!OLD_HASH!" (
            set "NEED_INSTALL=1"
        )
    )

    if defined NEED_INSTALL (
        if not defined GUI_MODE echo       Upgrading pip...
        python -m pip install --upgrade pip --quiet
        if errorlevel 1 (
            if not defined GUI_MODE echo       WARNING: Failed to upgrade pip.
        )
        if not defined GUI_MODE echo       Installing packages from %REQUIREMENTS_FILE%...
        python -m pip install -r "%REQUIREMENTS_FILE%" --quiet
        if errorlevel 1 (
            echo       ERROR: Failed to install dependencies.
            echo       Try running: pip install -r "%REQUIREMENTS_FILE%" manually
            popd
            if not defined GUI_MODE pause
            exit /b 1
        )
        if defined CUR_HASH (
            > "%REQ_HASH_FILE%" echo(!CUR_HASH!
        )
        if not defined GUI_MODE echo       Dependencies installed/updated.

        rem Optional: install package in development mode after deps update
        if exist "pyproject.toml" (
            if not defined GUI_MODE echo       Installing StreamQ in development mode...
            python -m pip install -e . --quiet 2>nul
            if not errorlevel 1 (
                if not defined GUI_MODE echo       Development installation completed.
            ) else (
                if not defined GUI_MODE echo       Note: Development mode installation failed, using fallback mode.
            )
        )
    ) else (
        if not defined GUI_MODE echo       Dependencies up-to-date. Skipping installation.
    )
) else (
    if not defined GUI_MODE echo       WARNING: %REQUIREMENTS_FILE% not found, skipping dependency installation.
)

:: Launch the application
if not defined GUI_MODE (
  echo [5/5] Launching StreamQ...
  echo.
  echo Starting StreamQ GUI...
)

REM Choose interpreter for GUI launch
set "PYW=%VENV_DIR%\Scripts\pythonw.exe"
if not exist "%PYW%" set "PYW=pythonw"

REM Try multiple launch methods in order of preference
if defined GUI_MODE (
  start "" "%PYW%" -m streamq
  if not errorlevel 1 goto :success
  start "" "%PYW%" -m src.streamq
  if not errorlevel 1 goto :success
  start "" "%PYW%" main.py
  if not errorlevel 1 goto :success
) else (
  python -m streamq 2>nul
  if not errorlevel 1 goto :success
  python -m src.streamq 2>nul
  if not errorlevel 1 goto :success
  python main.py
  if not errorlevel 1 goto :success
)

:: If all methods fail
echo ERROR: Failed to start StreamQ application.
echo Please check the installation and try again.
set "EXIT_CODE=1"
goto :cleanup

:success
set "EXIT_CODE=0"

:cleanup
popd

if not defined GUI_MODE (
  if not "%EXIT_CODE%"=="0" (
      echo.
      echo ========================================
      echo Program exited with error code %EXIT_CODE%
      echo ========================================
  ) else (
      echo.
      echo ========================================
      echo       StreamQ session completed
      echo ========================================
  )
  echo Press any key to exit...
  pause >nul
)
exit /b %EXIT_CODE%
