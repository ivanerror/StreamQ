"""Main StreamQ application with GUI interface."""

import os
import sys
import subprocess
import threading
import tkinter as tk
from tkinter import ttk, messagebox
import tkinter.font as tkfont

from ..config import config
from ..utils.ffmpeg import ensure_ffmpeg
from .downloader import DownloadManager, DownloadQueue


class StreamQApp:
    """Main StreamQ application with Tkinter GUI."""
    
    def __init__(self, master):
        """
        Initialize the StreamQ application.
        
        Args:
            master: The root Tkinter window
        """
        self.master = master
        self.ffmpeg_dir = ensure_ffmpeg()
        
        # Initialize download system
        self.download_manager = DownloadManager(self.ffmpeg_dir)
        self.download_queue = DownloadQueue(self.download_manager)
        
        # Set up callbacks
        self.download_manager.set_progress_callback(self._on_progress_update)
        self.download_queue.set_status_callback(self._on_status_update)
        self.download_queue.set_completion_callback(self._on_download_complete)
        
        # UI state
        self.progress_var = tk.DoubleVar(value=0.0)
        self.status_var = tk.StringVar(value="Ready to download.")
        self.format_var = tk.StringVar(value="audio")
        self.quality_var = tk.StringVar()
        
        # Status tag mapping
        self.status_tags = {
            "Pending": "status-pending",
            "Downloading": "status-downloading", 
            "Completed": "status-completed",
            "Failed": "status-failed",
        }
        
        # Configure and build the UI
        self._configure_window()
        self._build_layout()
        self._update_quality_options()
        
        # Ensure directories exist
        config.ensure_directories()
    
    def _configure_window(self):
        """Configure the main window appearance and styling."""
        self.master.title(config.window_title)
        self.master.geometry(config.window_geometry)
        self.master.minsize(*config.window_min_size)
        
        # Configure theme and styling
        self.style = ttk.Style()
        current_theme = self.style.theme_use()
        for theme in config.preferred_themes:
            if theme in self.style.theme_names():
                self.style.theme_use(theme)
                break
        else:
            self.style.theme_use(current_theme)
        
        # Configure fonts
        default_font = tkfont.nametofont("TkDefaultFont")
        default_font.configure(family=config.default_font_family, size=config.default_font_size)
        self.master.option_add("*Font", default_font)
        
        # Set background color
        background = self.style.lookup("TFrame", "background")
        if not background:
            background = "#f3f3f3"
        self.master.configure(background=background)
        
        # Configure custom styles
        self.style.configure("Header.TLabel", font=(config.default_font_family + " Semibold", 16))
        self.style.configure("Section.TLabelframe", padding=15)
        self.style.configure("Section.TLabelframe.Label", font=(config.default_font_family + " Semibold", 11))
        self.style.configure("Accent.TButton", padding=(12, 6))
    
    def _build_layout(self):
        """Build the main application layout."""
        container = ttk.Frame(self.master, padding=(20, 20, 20, 20))
        container.pack(fill="both", expand=True)
        container.columnconfigure(0, weight=1)
        container.rowconfigure(3, weight=1)
        
        # Header section
        self._build_header_section(container)
        
        # URL input section
        self._build_url_section(container)
        
        # Queue display section
        self._build_queue_section(container)
        
        # Format and quality section
        self._build_format_section(container)
        
        # Progress section
        self._build_progress_section(container)
        
        # Action buttons section
        self._build_actions_section(container)
        
        # Set initial focus
        self.url_entry.focus_set()
    
    def _build_header_section(self, container):
        """Build the header section with title and subtitle."""
        header = ttk.Label(container, text="StreamQ", style="Header.TLabel")
        header.grid(row=0, column=0, sticky="w")
        
        subtitle = ttk.Label(
            container,
            text="Queue and download audio or video from YouTube with a familiar Windows look.",
            wraplength=520,
        )
        subtitle.grid(row=1, column=0, sticky="w", pady=(4, 16))
    
    def _build_url_section(self, container):
        """Build the URL input section."""
        url_section = ttk.LabelFrame(container, text="Source", style="Section.TLabelframe")
        url_section.grid(row=2, column=0, sticky="ew", pady=(0, 16))
        url_section.columnconfigure(0, weight=1)
        
        self.url_entry = ttk.Entry(url_section)
        self.url_entry.grid(row=0, column=0, sticky="ew", pady=(0, 8))
        self.add_button = ttk.Button(url_section, text="Add to Queue", command=self._add_to_queue)
        self.add_button.grid(row=0, column=1, padx=(10, 0))
    
    def _build_queue_section(self, container):
        """Build the download queue display section."""
        queue_section = ttk.LabelFrame(container, text="Download Queue", style="Section.TLabelframe")
        queue_section.grid(row=3, column=0, sticky="nsew", pady=(0, 16))
        queue_section.columnconfigure(0, weight=1)
        queue_section.rowconfigure(0, weight=1)
        
        # Main queue treeview
        self.queue_display = ttk.Treeview(
            queue_section,
            columns=("status", "url", "title"),
            show="headings",
            selectmode="browse",
            height=8,
        )
        self.queue_display.heading("status", text="Status")
        self.queue_display.heading("url", text="Link")
        self.queue_display.heading("title", text="Title")
        self.queue_display.column("status", anchor="center", width=120, stretch=False)
        self.queue_display.column("url", anchor="w", width=360, stretch=True)
        self.queue_display.column("title", anchor="w", width=360, stretch=True)
        
        queue_scroll = ttk.Scrollbar(queue_section, orient="vertical", command=self.queue_display.yview)
        self.queue_display.configure(yscrollcommand=queue_scroll.set)
        self.queue_display.grid(row=0, column=0, sticky="nsew")
        queue_scroll.grid(row=0, column=1, sticky="ns", padx=(8, 0))
        
        # Configure status tag colors
        tag_colors = {
            "status-pending": "#616161",
            "status-downloading": "#005FB8",
            "status-completed": "#107C10",
            "status-failed": "#D13438",
        }
        for tag, color in tag_colors.items():
            self.queue_display.tag_configure(tag, foreground=color)
        
    
    def _build_format_section(self, container):
        """Build the format and quality selection section."""
        format_section = ttk.LabelFrame(container, text="Output Preferences", style="Section.TLabelframe")
        format_section.grid(row=4, column=0, sticky="ew", pady=(0, 16))
        format_section.columnconfigure(1, weight=1)
        
        # Format radio buttons
        self.format_radio_audio = ttk.Radiobutton(
            format_section,
            text="Audio (MP3)",
            variable=self.format_var,
            value="audio",
            command=self._update_quality_options,
        )
        self.format_radio_audio.grid(row=0, column=0, sticky="w")
        
        self.format_radio_video = ttk.Radiobutton(
            format_section,
            text="Video (MP4)",
            variable=self.format_var,
            value="video", 
            command=self._update_quality_options,
        )
        self.format_radio_video.grid(row=0, column=1, sticky="w")
        
        # Quality selection
        quality_label = ttk.Label(format_section, text="Quality")
        quality_label.grid(row=1, column=0, sticky="w", pady=(12, 0))
        
        self.quality_dropdown = ttk.Combobox(format_section, textvariable=self.quality_var, state="readonly", width=12)
        self.quality_dropdown.grid(row=1, column=1, sticky="w", pady=(12, 0))
    
    def _build_progress_section(self, container):
        """Build the progress display section."""
        progress_section = ttk.LabelFrame(container, text="Status", style="Section.TLabelframe")
        progress_section.grid(row=5, column=0, sticky="ew")
        progress_section.columnconfigure(0, weight=1)
        
        self.progress_bar = ttk.Progressbar(progress_section, variable=self.progress_var, maximum=100)
        self.progress_bar.grid(row=0, column=0, sticky="ew")
        
        self.status_label = ttk.Label(progress_section, textvariable=self.status_var)
        self.status_label.grid(row=1, column=0, sticky="w", pady=(8, 0))
    
    def _build_actions_section(self, container):
        """Build the action buttons section."""
        actions_frame = ttk.Frame(container)
        actions_frame.grid(row=6, column=0, sticky="ew", pady=(20, 0))
        actions_frame.columnconfigure(0, weight=1)
        
        self.download_button = ttk.Button(
            actions_frame,
            text="Start Download",
            command=self._start_download,
            style="Accent.TButton",
        )
        self.download_button.grid(row=0, column=0, sticky="e")
    
    def _update_quality_options(self):
        """Update quality options based on selected format."""
        format_type = self.format_var.get()
        if format_type == "audio":
            qualities = config.audio_qualities
        else:
            qualities = config.video_qualities
        self.quality_dropdown["values"] = qualities
        self.quality_var.set(qualities[-1])
    
    def _add_to_queue(self):
        """Add URL to the download queue."""
        url = self.url_entry.get().strip()
        if not url:
            messagebox.showwarning("Warning", "Please enter a URL.")
            return
        
        item_id = self.queue_display.insert(
            "",
            "end", 
            values=("Pending", url, "Fetching title..."),
            tags=(self.status_tags["Pending"],),
        )
        
        entry = self.download_queue.add_to_queue(url, item_id)
        
        pending_total = self.download_queue.get_pending_count()
        self.status_var.set(f"Added to queue. Pending items: {pending_total}.")
        
        self.url_entry.delete(0, tk.END)
    
    def _start_download(self):
        """Start processing the download queue."""
        if self.download_queue.is_downloading:
            messagebox.showinfo("Info", "A download is already in progress.")
            return
        
        pending_count = self.download_queue.get_pending_count()
        if pending_count == 0:
            messagebox.showwarning("Warning", "No pending items in the queue.")
            return
        
        format_type = self.format_var.get()
        quality = self.quality_var.get()
        if not quality:
            messagebox.showwarning("Warning", "Please select a quality option.")
            return
        
        self.progress_var.set(0.0)
        self.status_var.set(f"Preparing {pending_count} download(s)...")
        self._set_controls_state(disabled=True)
        
        self.download_queue.process_queue(format_type, quality)
    
    def _set_controls_state(self, disabled):
        """Enable or disable UI controls."""
        widgets = [
            self.url_entry,
            self.add_button,
            self.download_button,
            self.format_radio_audio,
            self.format_radio_video,
            self.quality_dropdown,
        ]
        for widget in widgets:
            if disabled:
                widget.state(["disabled"])
            else:
                widget.state(["!disabled"])
        if not disabled:
            self.url_entry.focus_set()
    
    def _on_progress_update(self, percent_value, message):
        """Handle progress updates from download manager."""
        self.master.after(0, self._update_progress_ui, percent_value, message)
    
    def _update_progress_ui(self, value, message):
        """Update progress UI elements."""
        self.progress_var.set(max(0.0, min(100.0, value)))
        self.status_var.set(message)
    
    def _on_status_update(self, update_type, entry):
        """Handle status updates from download queue."""
        if update_type == "status_changed":
            self.master.after(0, self._update_queue_status, entry)
        elif update_type == "title_updated":
            self.master.after(0, self._update_entry_title, entry)
    
    def _update_queue_status(self, entry):
        """Update the status display for a queue entry."""
        status = entry["status"]
        item_id = entry["item_id"]
        # Keep existing URL and Title when updating status
        current_values = self.queue_display.item(item_id, "values") or ("", "", "")
        url = entry.get("url") or (current_values[1] if len(current_values) > 1 else "")
        title = entry.get("title") or (current_values[2] if len(current_values) > 2 else "")
        self.queue_display.item(item_id, values=(status, url, title))
        tag = self.status_tags.get(status)
        if tag:
            self.queue_display.item(item_id, tags=(tag,))

    def _update_entry_title(self, entry):
        """Update the title for a queue entry."""
        item_id = entry.get("item_id")
        if not item_id:
            return
        current_values = self.queue_display.item(item_id, "values") or ("", "", "")
        status = entry.get("status") or (current_values[0] if len(current_values) > 0 else "")
        url = entry.get("url") or (current_values[1] if len(current_values) > 1 else "")
        title = entry.get("title") or ""
        self.queue_display.item(item_id, values=(status, url, title))

    
    def _on_download_complete(self, format_type, errors, completed):
        """Handle download completion."""
        self.master.after(0, self._handle_download_complete, format_type, errors, completed)
    
    def _handle_download_complete(self, format_type, errors, completed):
        """Handle download completion in the main thread."""
        self._set_controls_state(disabled=False)
        
        success_count = len(completed)
        failure_count = len(errors)
        counts = self.download_queue.get_status_counts()
        pending_count = counts["Pending"]
        
        if failure_count:
            details = "\n".join(f"- {url}: {error}" for url, error in errors[:3])
            if success_count:
                self.progress_var.set(100.0)
                self.status_var.set(
                    f"Finished with issues | Success {success_count} | Failed {failure_count} | Pending {pending_count}"
                )
                messagebox.showwarning("Download finished with issues", f"Some downloads failed:\n{details}")
            else:
                self.progress_var.set(0.0)
                self.status_var.set(f"Downloads failed | Failed {failure_count} | Pending {pending_count}")
                messagebox.showerror("Downloads failed", f"All downloads failed:\n{details}")
        else:
            if success_count:
                self.progress_var.set(100.0)
                self.status_var.set(f"All downloads completed | Success {success_count} | Pending {pending_count}")
                messagebox.showinfo("Info", "Downloads complete")
                self._open_download_folder(format_type)
            else:
                self.progress_var.set(0.0)
                self.status_var.set("No downloads were processed.")
    
    def _open_download_folder(self, format_type):
        """Open the download folder in the system file manager."""
        download_dir = config.get_download_dir(format_type)
        if not os.path.isdir(download_dir):
            return
        if os.name == "nt":
            os.startfile(download_dir)
        elif sys.platform == "darwin":
            subprocess.call(["open", download_dir])
        else:
            subprocess.call(["xdg-open", download_dir])
