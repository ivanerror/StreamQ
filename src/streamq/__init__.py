"""
StreamQ - A desktop app to queue and download video/audio from YouTube.

Copyright (c) 2025 ivanerror (https://github.com/ivanerror)
All rights reserved.
"""

__version__ = "1.1.0"
__author__ = "ivanerror"
__email__ = "https://github.com/ivanerror"
__description__ = "Queue and download video/audio from YouTube using yt-dlp with a clean GUI"


def __getattr__(name):
    # Import the GUI lazily so core modules work without tkinter (e.g. in tests)
    if name == "StreamQApp":
        from .core.app import StreamQApp

        return StreamQApp
    raise AttributeError(f"module {__name__!r} has no attribute {name!r}")


__all__ = ["StreamQApp"]