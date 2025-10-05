#!/bin/bash
# StreamQ - Cross-platform launch script for Unix-like systems
# Copyright (c) 2025 ivanerror (https://github.com/ivanerror)
# All rights reserved.

set -e  # Exit on any error

echo "========================================"
echo "         StreamQ Launcher"
echo "========================================"
echo

# Get script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$SCRIPT_DIR"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

print_step() {
    echo -e "${BLUE}[$1/5]${NC} $2"
}

print_success() {
    echo -e "      ${GREEN}$1${NC}"
}

print_warning() {
    echo -e "      ${YELLOW}WARNING: $1${NC}"
}

print_error() {
    echo -e "      ${RED}ERROR: $1${NC}"
}

# Step 1: Find Python interpreter
print_step 1 "Looking for Python interpreter..."

PYTHON_CMD=""

# Try python3 first (preferred on Unix)
if command -v python3 >/dev/null 2>&1; then
    if python3 -c "import sys; exit(0 if sys.version_info >= (3, 8) else 1)" 2>/dev/null; then
        PYTHON_CMD="python3"
        print_success "Found: python3"
    fi
fi

# Try python if python3 not found
if [ -z "$PYTHON_CMD" ] && command -v python >/dev/null 2>&1; then
    if python -c "import sys; exit(0 if sys.version_info >= (3, 8) else 1)" 2>/dev/null; then
        PYTHON_CMD="python"
        print_success "Found: python"
    fi
fi

# Error if no suitable Python found
if [ -z "$PYTHON_CMD" ]; then
    print_error "Python 3.8+ not found!"
    echo "      Please install Python 3.8 or later:"
    echo "      - macOS: brew install python"
    echo "      - Ubuntu/Debian: sudo apt install python3 python3-venv python3-pip"
    echo "      - Fedora: sudo dnf install python3 python3-pip"
    exit 1
fi

# Step 2: Setup virtual environment
VENV_DIR=".venv"
print_step 2 "Setting up virtual environment..."

if [ ! -f "$VENV_DIR/bin/python" ]; then
    print_success "Creating virtual environment..."
    $PYTHON_CMD -m venv "$VENV_DIR"
    print_success "Virtual environment created successfully."
else
    print_success "Virtual environment already exists."
fi

# Step 3: Activate virtual environment
print_step 3 "Activating virtual environment..."
source "$VENV_DIR/bin/activate"
print_success "Virtual environment activated."

# Step 4: Install/update dependencies
print_step 4 "Installing dependencies..."

if [ -f "requirements.txt" ]; then
    print_success "Upgrading pip..."
    python -m pip install --upgrade pip --quiet || print_warning "Failed to upgrade pip."
    
    print_success "Installing packages from requirements.txt..."
    python -m pip install -r requirements.txt --quiet || {
        print_error "Failed to install dependencies."
        echo "      Try running: pip install -r requirements.txt manually"
        exit 1
    }
    print_success "Dependencies installed successfully."
else
    print_warning "requirements.txt not found, skipping dependency installation."
fi

# Install package in development mode if possible
if [ -f "pyproject.toml" ]; then
    print_success "Installing StreamQ in development mode..."
    if python -m pip install -e . --quiet 2>/dev/null; then
        print_success "Development installation completed."
    else
        print_warning "Development mode installation failed, using fallback mode."
    fi
fi

# Check FFmpeg availability
print_step 5 "Checking FFmpeg availability..."
if command -v ffmpeg >/dev/null 2>&1 && command -v ffprobe >/dev/null 2>&1; then
    print_success "FFmpeg is available in PATH."
else
    print_warning "FFmpeg not found in PATH!"
    echo "      Please install FFmpeg:"
    echo "      - macOS: brew install ffmpeg"
    echo "      - Ubuntu/Debian: sudo apt install ffmpeg"
    echo "      - Fedora: sudo dnf install ffmpeg"
    echo "      - Arch: sudo pacman -S ffmpeg"
    echo
    echo "      StreamQ will attempt to continue anyway..."
fi

# Launch the application
echo
echo "Starting StreamQ GUI..."

# Try multiple launch methods
if python -m streamq 2>/dev/null; then
    EXIT_CODE=0
elif python -m src.streamq 2>/dev/null; then
    EXIT_CODE=0
elif python main.py; then
    EXIT_CODE=0
else
    print_error "Failed to start StreamQ application."
    echo "      Please check the installation and try again."
    EXIT_CODE=1
fi

echo
if [ $EXIT_CODE -eq 0 ]; then
    echo "========================================"
    echo "      StreamQ session completed"
    echo "========================================"
else
    echo "========================================"
    echo "   Program exited with error code $EXIT_CODE"
    echo "========================================"
fi

exit $EXIT_CODE