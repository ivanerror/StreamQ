@echo off
REM StreamQ
REM Copyright (c) 2025 ivanerror (https://github.com/ivanerror)
REM All rights reserved.
setlocal

:: Pastikan perintah berjalan dari lokasi skrip
pushd "%~dp0" >nul 2>&1
if errorlevel 1 (
    echo Gagal mengakses direktori skrip.
    pause
    exit /b 1
)

:: Cari interpreter Python
set "PYTHON_CMD="
where python >nul 2>&1
if not errorlevel 1 (
    set "PYTHON_CMD=python"
)

if not defined PYTHON_CMD (
    py -3 --version >nul 2>&1
    if not errorlevel 1 (
        set "PYTHON_CMD=py -3"
    )
)

if not defined PYTHON_CMD (
    echo Python 3 tidak ditemukan. Silakan install Python 3.8+ lalu jalankan ulang.
    popd
    pause
    exit /b 1
)

set "VENV_DIR=.venv"
if not exist "%VENV_DIR%\Scripts\python.exe" (
    echo Membuat virtual environment di "%VENV_DIR%"...
    %PYTHON_CMD% -m venv "%VENV_DIR%"
    if errorlevel 1 (
        echo Gagal membuat virtual environment.
        popd
        pause
        exit /b 1
    )
)

call "%VENV_DIR%\Scripts\activate.bat"
if errorlevel 1 (
    echo Gagal mengaktifkan virtual environment.
    popd
    pause
    exit /b 1
)

set "REQUIREMENTS_FILE=requirements.txt"
if exist "%REQUIREMENTS_FILE%" (
    echo Menginstal dependensi...
    python -m pip install --upgrade pip
    if errorlevel 1 (
        echo Gagal memperbarui pip.
        popd
        pause
        exit /b 1
    )
    python -m pip install -r "%REQUIREMENTS_FILE%"
    if errorlevel 1 (
        echo Gagal menginstal dependensi dari "%REQUIREMENTS_FILE%".
        popd
        pause
        exit /b 1
    )
) else (
    echo Melewati instalasi dependensi karena "%REQUIREMENTS_FILE%" tidak ditemukan.
)

echo Menjalankan aplikasi...
python main.py
set "EXIT_CODE=%ERRORLEVEL%"

popd
if not "%EXIT_CODE%"=="0" (
    echo Program selesai dengan kode error %EXIT_CODE%.
)
pause
exit /b %EXIT_CODE%
