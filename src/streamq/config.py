"""Configuration management for StreamQ."""

import os
from pathlib import Path


def get_project_root():
    """
    Get the project root directory.
    
    Returns:
        str: Path to the project root directory
    """
    # This will work whether we're running from the old structure or new structure
    current_file = Path(__file__).resolve()
    
    # From src/streamq/config.py, go up to project root
    if "src" in current_file.parts:
        # New structure: src/streamq/config.py -> project root
        return str(current_file.parent.parent.parent)
    else:
        # Fallback: assume we're in the project root
        return str(current_file.parent)


class Config:
    """Configuration settings for StreamQ."""
    
    def __init__(self):
        self.project_root = get_project_root()
        
        # Directory configurations
        self.download_dir = os.path.join(self.project_root, "py_downloader")
        self.audio_dir = os.path.join(self.download_dir, "audio")
        self.video_dir = os.path.join(self.download_dir, "video")
        self.ffmpeg_dir = os.path.join(self.project_root, "ffmpeg_support")
        
        # Quality options
        self.audio_qualities = ["64", "128", "192", "256", "320"]
        self.video_qualities = ["144", "240", "360", "480", "720", "1080"]
        
        # UI settings
        self.window_title = "StreamQ"
        self.window_geometry = "900x760"
        self.window_min_size = (800, 700)
        
        # Styling
        self.preferred_themes = ("vista", "xpnative", "clam")
        # ttkbootstrap theme name (used when ttkbootstrap is available)
        self.bootstrap_theme = "flatly"
        self.default_font_family = "Segoe UI"
        self.default_font_size = 10
        
    def get_download_dir(self, format_type):
        """
        Get the download directory for a specific format type.
        
        Args:
            format_type (str): Either 'audio' or 'video'
            
        Returns:
            str: Path to the download directory
        """
        if format_type == "audio":
            return self.audio_dir
        elif format_type == "video":
            return self.video_dir
        else:
            return self.download_dir
    
    def ensure_directories(self):
        """Ensure all required directories exist."""
        os.makedirs(self.audio_dir, exist_ok=True)
        os.makedirs(self.video_dir, exist_ok=True)
        os.makedirs(self.ffmpeg_dir, exist_ok=True)


# Global configuration instance
config = Config()
