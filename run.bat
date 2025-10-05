@echo off
REM StreamQ - Enhanced launch script with support for new project structure
REM Copyright (c) 2025 ivanerror (https://github.com/ivanerror)
REM All rights reserved.
setlocal

echo ========================================
echo           StreamQ Launcher
echo ========================================
echo.

:: Ensure we're running from the script's directory
pushd "%~dp0" >nul 2>&1
if errorlevel 1 (
    echo ERROR: Failed to access script directory.
    pause
    exit /b 1
)

:: Find Python interpreter
set "PYTHON_CMD="
echo [1/5] Looking for Python interpreter...

:: Try python command first
where python >nul 2>&1
if not errorlevel 1 (
    python --version 2>&1 | findstr /C:"Python 3" >nul
    if not errorlevel 1 (
        set "PYTHON_CMD=python"
        echo       Found: python
    )
)

:: Try py launcher if python not found
if not defined PYTHON_CMD (
    py -3 --version >nul 2>&1
    if not errorlevel 1 (
        set "PYTHON_CMD=py -3"
        echo       Found: py -3 launcher
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
echo [2/5] Setting up virtual environment...

if not exist "%VENV_DIR%\Scripts\python.exe" (
    echo       Creating virtual environment...
    %PYTHON_CMD% -m venv "%VENV_DIR%"
    if errorlevel 1 (
        echo       ERROR: Failed to create virtual environment.
        popd
        pause
        exit /b 1
    )
    echo       Virtual environment created successfully.
) else (
    echo       Virtual environment already exists.
)

:: Activate virtual environment
echo [3/5] Activating virtual environment...
call "%VENV_DIR%\Scripts\activate.bat"
if errorlevel 1 (
    echo       ERROR: Failed to activate virtual environment.
    popd
    pause
    exit /b 1
)
echo       Virtual environment activated.

:: Install/update dependencies
echo [4/5] Installing dependencies...
set "REQUIREMENTS_FILE=requirements.txt"
if exist "%REQUIREMENTS_FILE%" (
    echo       Upgrading pip...
    python -m pip install --upgrade pip --quiet
    if errorlevel 1 (
        echo       WARNING: Failed to upgrade pip.
    )
    
    echo       Installing packages from %REQUIREMENTS_FILE%...
    python -m pip install -r "%REQUIREMENTS_FILE%" --quiet
    if errorlevel 1 (
        echo       ERROR: Failed to install dependencies.
        echo       Try running: pip install -r "%REQUIREMENTS_FILE%" manually
        popd
        pause
        exit /b 1
    )
    echo       Dependencies installed successfully.
) else (
    echo       WARNING: %REQUIREMENTS_FILE% not found, skipping dependency installation.
)

:: Install package in development mode if possible
if exist "pyproject.toml" (
    echo       Installing StreamQ in development mode...
    python -m pip install -e . --quiet 2>nul
    if not errorlevel 1 (
        echo       Development installation completed.
    ) else (
        echo       Note: Development mode installation failed, using fallback mode.
    )
)

:: Launch the application
echo [5/5] Launching StreamQ...
echo.

:: Try multiple launch methods in order of preference
echo Starting StreamQ GUI...

:: Method 1: Try installed package entry point
python -m streamq 2>nul
if not errorlevel 1 goto :success

:: Method 2: Try new module structure
python -m src.streamq 2>nul  
if not errorlevel 1 goto :success

:: Method 3: Fallback to main.py
python main.py
if not errorlevel 1 goto :success

:: If all methods fail
echo ERROR: Failed to start StreamQ application.
echo Please check the installation and try again.
set "EXIT_CODE=1"
goto :cleanup

:success
set "EXIT_CODE=0"

:cleanup
popd

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
exit /b %EXIT_CODE%
