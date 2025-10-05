# Changelog

All notable changes to StreamQ will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [1.0.0] - 2025-01-05

### Added
- Complete project restructuring following Python best practices
- Modern `src/` package layout for better organization
- Modular architecture with separated concerns:
  - [`src/streamq/core/app.py`](src/streamq/core/app.py) - Main GUI application
  - [`src/streamq/core/downloader.py`](src/streamq/core/downloader.py) - Download logic and queue management
  - [`src/streamq/utils/ffmpeg.py`](src/streamq/utils/ffmpeg.py) - FFmpeg handling utilities
  - [`src/streamq/config.py`](src/streamq/config.py) - Configuration management
- Proper packaging with [`pyproject.toml`](pyproject.toml) and [`setup.py`](setup.py)
- Multiple entry points for running the application:
  - `python main.py` (legacy compatibility)
  - `python -m streamq` (package module)
  - `streamq` command (after installation)
- Enhanced run scripts:
  - [`run.bat`](run.bat) - Enhanced Windows launcher
  - [`run.sh`](run.sh) - Cross-platform Unix launcher
- Comprehensive documentation updates
- MIT License added
- Development installation support (`pip install -e .`)

### Changed
- Refactored monolithic `main.py` into logical modules
- Improved error handling and user feedback
- Enhanced virtual environment management in run scripts
- Better FFmpeg detection and setup process
- Modernized project structure following Python packaging standards

### Technical Improvements
- Separation of GUI logic from download management
- Centralized configuration system
- Type-safe module organization
- Better import structure and dependency management
- Enhanced cross-platform compatibility
- Improved development workflow support

### Dependencies
- Maintains compatibility with existing `yt-dlp>=2024.8.6` requirement
- Added development dependencies for code quality tools

### Backward Compatibility
- Existing `run.bat` functionality preserved and enhanced
- Original `main.py` entry point maintained for compatibility
- Download directory structure unchanged
- All user-facing features remain identical

## [0.9.0] - Previous Release
### Initial Features
- Basic GUI interface with Tkinter
- YouTube video/audio downloading using yt-dlp
- Queue management system
- FFmpeg integration for Windows
- Progress tracking and status updates
- Automatic download folder opening