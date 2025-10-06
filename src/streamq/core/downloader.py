"""Download functionality for StreamQ using yt-dlp."""

import os
import threading
import yt_dlp
from ..config import config


class DownloadManager:
    """Manages YouTube video/audio downloads using yt-dlp."""
    
    def __init__(self, ffmpeg_dir):
        """
        Initialize the download manager.
        
        Args:
            ffmpeg_dir (str): Path to the FFmpeg bin directory
        """
        self.ffmpeg_dir = ffmpeg_dir
        self.progress_callback = None
        self.status_callback = None
    
    def set_progress_callback(self, callback):
        """Set the progress update callback function."""
        self.progress_callback = callback
    
    def set_status_callback(self, callback):
        """Set the status update callback function."""
        self.status_callback = callback
    
    def fetch_video_title(self, url):
        """
        Fetch the title of a YouTube video without downloading.
        
        Args:
            url (str): YouTube video URL
            
        Returns:
            str: Video title or error message
        """
        options = {
            "quiet": True,
            "skip_download": True,
            "no_warnings": True,
        }
        
        try:
            with yt_dlp.YoutubeDL(options) as ydl:
                info = ydl.extract_info(url, download=False)
            title = info.get("title") or "Unknown title"
            return title.replace("\n", " ").strip() or "Unknown title"
        except Exception:
            return "Title unavailable"
    
    def download_video(self, url, format_type, quality, index=1, total=1):
        """
        Download a single video/audio from YouTube.
        
        Args:
            url (str): YouTube video URL
            format_type (str): 'audio' or 'video'
            quality (str): Quality setting
            index (int): Current download index
            total (int): Total downloads in queue
        """
        download_dir = config.get_download_dir(format_type)
        os.makedirs(download_dir, exist_ok=True)
        
        if format_type == "audio":
            ydl_opts = {
                "format": f"bestaudio[abr<={quality}]/best",
                "postprocessors": [
                    {
                        "key": "FFmpegExtractAudio",
                        "preferredcodec": "mp3",
                        "preferredquality": quality,
                    }
                ],
                "outtmpl": os.path.join(download_dir, "%(title)s.%(ext)s"),
                "ffmpeg_location": self.ffmpeg_dir,
            }
        else:
            ydl_opts = {
                "format": f"bestvideo[height<={quality}]+bestaudio/best[height<={quality}]",
                "outtmpl": os.path.join(download_dir, "%(title)s.%(ext)s"),
                "ffmpeg_location": self.ffmpeg_dir,
                "merge_output_format": "mp4",
            }
        
        def progress_hook(data):
            self._handle_progress(data, url, index, total)
        
        ydl_opts["progress_hooks"] = [progress_hook]
        
        with yt_dlp.YoutubeDL(ydl_opts) as ydl:
            ydl.download([url])
    
    def _handle_progress(self, data, url, index, total):
        """Handle progress updates from yt-dlp."""
        if not self.progress_callback:
            return
            
        status = data.get("status")
        if status == "downloading":
            percent_value, percent_text = self._extract_percent(data)
            speed_text = data.get("_speed_str") or ""
            message_parts = [f"Downloading {index}/{total}", percent_text]
            if speed_text:
                message_parts.append(speed_text)
            message = " | ".join(part for part in message_parts if part)
            self.progress_callback(percent_value, message)
        elif status == "finished":
            message = f"Processing download {index}/{total}..."
            self.progress_callback(100.0, message)
    
    @staticmethod
    def _extract_percent(data):
        """Extract percentage from yt-dlp progress data."""
        percent_text = data.get("_percent_str")
        if percent_text:
            percent_text = percent_text.strip()
            if percent_text.endswith("%"):
                try:
                    return float(percent_text[:-1]), percent_text
                except ValueError:
                    pass
        total_bytes = data.get("total_bytes") or data.get("total_bytes_estimate")
        downloaded_bytes = data.get("downloaded_bytes")
        if total_bytes and downloaded_bytes:
            percent_value = (downloaded_bytes / total_bytes) * 100
            percent_value = max(0.0, min(100.0, percent_value))
            return percent_value, f"{percent_value:.1f}%"
        return 0.0, "0%"


class DownloadQueue:
    """Manages the download queue and processing."""
    
    def __init__(self, download_manager):
        """
        Initialize the download queue.
        
        Args:
            download_manager (DownloadManager): The download manager instance
        """
        self.download_manager = download_manager
        self.queue = []  # list of dicts: url, item_id, status, title
        self.is_downloading = False
        self.status_callback = None
        self.completion_callback = None
    
    def set_status_callback(self, callback):
        """Set the status update callback function."""
        self.status_callback = callback
    
    def set_completion_callback(self, callback):
        """Set the completion callback function."""
        self.completion_callback = callback
    
    def add_to_queue(self, url, item_id):
        """
        Add a URL to the download queue.
        
        Args:
            url (str): YouTube video URL
            item_id: Treeview item ID
            
        Returns:
            dict: The created queue entry
        """
        entry = {
            "url": url,
            "item_id": item_id,
            "status": "Pending",
            "title": None,
        }
        self.queue.append(entry)
        
        # Fetch title in background
        threading.Thread(
            target=self._fetch_title_for_entry,
            args=(entry,),
            daemon=True,
        ).start()
        
        return entry
    
    def _fetch_title_for_entry(self, entry):
        """Fetch title for a queue entry in background."""
        title = self.download_manager.fetch_video_title(entry["url"])
        entry["title"] = title
        
        # Notify that title is available
        if self.status_callback:
            self.status_callback("title_updated", entry)
    
    def process_queue(self, format_type, quality):
        """
        Process all pending items in the queue.
        
        Args:
            format_type (str): 'audio' or 'video'
            quality (str): Quality setting
        """
        if self.is_downloading:
            return
        
        pending_entries = [entry for entry in self.queue if entry["status"] == "Pending"]
        if not pending_entries:
            return
        
        self.is_downloading = True
        
        worker = threading.Thread(
            target=self._process_downloads,
            args=(pending_entries, format_type, quality),
            daemon=True,
        )
        worker.start()
    
    def _process_downloads(self, entries, format_type, quality):
        """Process downloads in background thread."""
        total = len(entries)
        errors = []
        completed = []
        
        for index, entry in enumerate(entries, start=1):
            url = entry["url"]
            
            # Update status to downloading
            entry["status"] = "Downloading"
            if self.status_callback:
                self.status_callback("status_changed", entry)
            
            try:
                self.download_manager.download_video(url, format_type, quality, index, total)
                completed.append(url)
                entry["status"] = "Completed"
            except Exception as error:
                errors.append((url, str(error)))
                entry["status"] = "Failed"
            
            # Notify status change
            if self.status_callback:
                self.status_callback("status_changed", entry)
        
        self.is_downloading = False
        
        # Notify completion
        if self.completion_callback:
            self.completion_callback(format_type, errors, completed)
    
    def get_pending_count(self):
        """Get the number of pending items."""
        return sum(1 for entry in self.queue if entry["status"] == "Pending")
    
    def get_status_counts(self):
        """Get counts for each status."""
        counts = {"Pending": 0, "Downloading": 0, "Completed": 0, "Failed": 0}
        for entry in self.queue:
            status = entry.get("status", "Pending")
            counts[status] += 1
        return counts
