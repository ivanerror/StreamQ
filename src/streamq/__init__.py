"""
StreamQ - A desktop app to queue and download video/audio from YouTube.

Copyright (c) 2025 ivanerror (https://github.com/ivanerror)
All rights reserved.
"""

__version__ = "1.0.0"
__author__ = "ivanerror"
__email__ = "https://github.com/ivanerror"
__description__ = "Queue and download video/audio from YouTube using yt-dlp with a clean GUI"

from .core.app import StreamQApp

__all__ = ["StreamQApp"]