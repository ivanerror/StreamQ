import os
import sys
import platform
import subprocess
import urllib.request
import zipfile
import shutil
import threading
from urllib.error import URLError

import tkinter as tk
from tkinter import ttk, messagebox
import tkinter.font as tkfont

import yt_dlp


def resolve_ffmpeg_url():
    system = platform.system()
    machine = platform.machine().lower()

    if system == "Windows":
        if machine in {"amd64", "x86_64"}:
            return "https://github.com/yt-dlp/FFmpeg-Builds/releases/latest/download/ffmpeg-master-latest-win64-gpl.zip"
        if machine in {"arm64", "aarch64"}:
            return "https://github.com/yt-dlp/FFmpeg-Builds/releases/latest/download/ffmpeg-master-latest-winarm64-gpl.zip"
        if machine in {"x86", "i386", "i686"}:
            return "https://github.com/yt-dlp/FFmpeg-Builds/releases/latest/download/ffmpeg-master-latest-win32-gpl.zip"
    return None


def prepend_to_path(path):
    current = os.environ.get("PATH", "")
    entries = [entry for entry in current.split(os.pathsep) if entry]
    if path not in entries:
        os.environ["PATH"] = os.pathsep.join([path] + entries)


def ensure_ffmpeg():
    ffmpeg_root = os.path.join(os.path.dirname(__file__), "ffmpeg_support")
    bin_dir = os.path.join(ffmpeg_root, "bin")
    ffmpeg_binary = "ffmpeg.exe" if os.name == "nt" else "ffmpeg"
    ffprobe_binary = "ffprobe.exe" if os.name == "nt" else "ffprobe"
    ffmpeg_path = os.path.join(bin_dir, ffmpeg_binary)
    ffprobe_path = os.path.join(bin_dir, ffprobe_binary)

    if os.path.isfile(ffmpeg_path) and os.path.isfile(ffprobe_path):
        prepend_to_path(bin_dir)
        return bin_dir

    download_url = resolve_ffmpeg_url()
    if not download_url:
        raise RuntimeError(
            "FFmpeg tidak ditemukan dan unduhan otomatis tidak tersedia untuk platform ini. "
            "Silakan instal FFmpeg secara manual dan tambahkan ke PATH."
        )

    os.makedirs(bin_dir, exist_ok=True)
    archive_path = os.path.join(ffmpeg_root, "ffmpeg_download.zip")

    try:
        with urllib.request.urlopen(download_url) as response, open(archive_path, "wb") as archive_file:
            shutil.copyfileobj(response, archive_file)
    except URLError as error:
        raise RuntimeError(f"Gagal mengunduh FFmpeg: {error}") from error

    try:
        required = {ffmpeg_binary, ffprobe_binary}
        with zipfile.ZipFile(archive_path) as archive:
            available = {os.path.basename(name) for name in archive.namelist() if os.path.basename(name) in required}
            if not required.issubset(available):
                raise RuntimeError("Arsip FFmpeg tidak berisi ffmpeg dan ffprobe.")
            for member in archive.namelist():
                basename = os.path.basename(member)
                if basename in required:
                    target_path = os.path.join(bin_dir, basename)
                    with archive.open(member) as source, open(target_path, "wb") as target:
                        shutil.copyfileobj(source, target)
    except zipfile.BadZipFile as error:
        raise RuntimeError("Arsip FFmpeg rusak atau tidak valid.") from error
    finally:
        if os.path.exists(archive_path):
            os.remove(archive_path)

    prepend_to_path(bin_dir)
    return bin_dir


class StreamQApp:
    def __init__(self, master):
        self.master = master
        self.ffmpeg_dir = ensure_ffmpeg()

        self.download_queue = []  # list of dicts: url, item_id, listbox_index, display_index, status
        self.downloading = False
        self.progress_var = tk.DoubleVar(value=0.0)
        self.status_var = tk.StringVar(value="Ready to download.")
        self.status_tags = {
            "Pending": "status-pending",
            "Downloading": "status-downloading",
            "Completed": "status-completed",
            "Failed": "status-failed",
        }

        self._configure_window()
        self._build_layout()
        self.update_quality_options()

    def _configure_window(self):
        self.master.title("StreamQ")
        self.master.geometry("580x620")
        self.master.minsize(560, 600)

        self.style = ttk.Style()
        preferred_themes = ("vista", "xpnative", "clam")
        current_theme = self.style.theme_use()
        for theme in preferred_themes:
            if theme in self.style.theme_names():
                self.style.theme_use(theme)
                break
        else:
            self.style.theme_use(current_theme)

        default_font = tkfont.nametofont("TkDefaultFont")
        default_font.configure(family="Segoe UI", size=10)
        self.master.option_add("*Font", default_font)
        background = self.style.lookup("TFrame", "background")
        if not background:
            background = "#f3f3f3"
        self.master.configure(background=background)

        self.style.configure("Header.TLabel", font=("Segoe UI Semibold", 16))
        self.style.configure("Section.TLabelframe", padding=15)
        self.style.configure("Section.TLabelframe.Label", font=("Segoe UI Semibold", 11))
        self.style.configure("Accent.TButton", padding=(12, 6))

    def _build_layout(self):
        container = ttk.Frame(self.master, padding=(20, 20, 20, 20))
        container.pack(fill="both", expand=True)
        container.columnconfigure(0, weight=1)
        container.rowconfigure(3, weight=1)

        header = ttk.Label(container, text="StreamQ", style="Header.TLabel")
        header.grid(row=0, column=0, sticky="w")

        subtitle = ttk.Label(
            container,
            text="Queue and download audio or video from YouTube with a familiar Windows look.",
            wraplength=520,
        )
        subtitle.grid(row=1, column=0, sticky="w", pady=(4, 16))

        url_section = ttk.LabelFrame(container, text="Source", style="Section.TLabelframe")
        url_section.grid(row=2, column=0, sticky="ew", pady=(0, 16))
        url_section.columnconfigure(0, weight=1)

        self.url_entry = ttk.Entry(url_section)
        self.url_entry.grid(row=0, column=0, sticky="ew", pady=(0, 8))
        self.add_button = ttk.Button(url_section, text="Add to Queue", command=self.add_to_queue)
        self.add_button.grid(row=0, column=1, padx=(10, 0))

        queue_section = ttk.LabelFrame(container, text="Download Queue", style="Section.TLabelframe")
        queue_section.grid(row=3, column=0, sticky="nsew", pady=(0, 16))
        queue_section.columnconfigure(0, weight=1)
        queue_section.rowconfigure(0, weight=1)
        queue_section.rowconfigure(1, weight=0)

        self.queue_display = ttk.Treeview(
            queue_section,
            columns=("status", "url"),
            show="headings",
            selectmode="browse",
            height=8,
        )
        self.queue_display.heading("status", text="Status")
        self.queue_display.heading("url", text="URL")
        self.queue_display.column("status", anchor="center", width=120, stretch=False)
        self.queue_display.column("url", anchor="w", width=420, stretch=True)

        queue_scroll = ttk.Scrollbar(queue_section, orient="vertical", command=self.queue_display.yview)
        self.queue_display.configure(yscrollcommand=queue_scroll.set)
        self.queue_display.grid(row=0, column=0, sticky="nsew")
        queue_scroll.grid(row=0, column=1, sticky="ns", padx=(8, 0))

        tag_colors = {
            "status-pending": "#616161",
            "status-downloading": "#005FB8",
            "status-completed": "#107C10",
            "status-failed": "#D13438",
        }
        for tag, color in tag_colors.items():
            self.queue_display.tag_configure(tag, foreground=color)

        summary_frame = ttk.Frame(queue_section)
        summary_frame.grid(row=1, column=0, columnspan=2, sticky="ew", pady=(12, 0))
        summary_frame.columnconfigure(0, weight=1)

        summary_header = ttk.Label(summary_frame, text="Queued URLs (review before downloading):")
        summary_header.grid(row=0, column=0, sticky="w")

        self.queue_listbox = tk.Listbox(
            summary_frame,
            height=5,
            activestyle="none",
            exportselection=False,
            font=("Consolas", 9),
        )
        self.queue_listbox.grid(row=1, column=0, sticky="nsew", pady=(6, 0))
        summary_scroll = ttk.Scrollbar(summary_frame, orient="vertical", command=self.queue_listbox.yview)
        self.queue_listbox.configure(yscrollcommand=summary_scroll.set)
        summary_scroll.grid(row=1, column=1, sticky="ns", padx=(8, 0))

        format_section = ttk.LabelFrame(container, text="Output Preferences", style="Section.TLabelframe")
        format_section.grid(row=4, column=0, sticky="ew", pady=(0, 16))
        format_section.columnconfigure(1, weight=1)

        self.format_var = tk.StringVar(value="audio")
        self.format_radio_audio = ttk.Radiobutton(
            format_section,
            text="Audio (MP3)",
            variable=self.format_var,
            value="audio",
            command=self.update_quality_options,
        )
        self.format_radio_audio.grid(row=0, column=0, sticky="w")

        self.format_radio_video = ttk.Radiobutton(
            format_section,
            text="Video (MP4)",
            variable=self.format_var,
            value="video",
            command=self.update_quality_options,
        )
        self.format_radio_video.grid(row=0, column=1, sticky="w")

        quality_label = ttk.Label(format_section, text="Quality")
        quality_label.grid(row=1, column=0, sticky="w", pady=(12, 0))

        self.quality_var = tk.StringVar()
        self.quality_dropdown = ttk.Combobox(format_section, textvariable=self.quality_var, state="readonly", width=12)
        self.quality_dropdown.grid(row=1, column=1, sticky="w", pady=(12, 0))

        progress_section = ttk.LabelFrame(container, text="Status", style="Section.TLabelframe")
        progress_section.grid(row=5, column=0, sticky="ew")
        progress_section.columnconfigure(0, weight=1)

        self.progress_bar = ttk.Progressbar(progress_section, variable=self.progress_var, maximum=100)
        self.progress_bar.grid(row=0, column=0, sticky="ew")

        self.status_label = ttk.Label(progress_section, textvariable=self.status_var)
        self.status_label.grid(row=1, column=0, sticky="w", pady=(8, 0))

        actions_frame = ttk.Frame(container)
        actions_frame.grid(row=6, column=0, sticky="ew", pady=(20, 0))
        actions_frame.columnconfigure(0, weight=1)

        self.download_button = ttk.Button(
            actions_frame,
            text="Start Download",
            command=self.start_download,
            style="Accent.TButton",
        )
        self.download_button.grid(row=0, column=0, sticky="e")

        self.url_entry.focus_set()

    def update_quality_options(self):
        format_type = self.format_var.get()
        if format_type == "audio":
            qualities = ["64", "128", "192", "256", "320"]
        else:
            qualities = ["144", "240", "360", "480", "720", "1080"]
        self.quality_dropdown["values"] = qualities
        self.quality_var.set(qualities[-1])

    def add_to_queue(self):
        url = self.url_entry.get().strip()
        if not url:
            messagebox.showwarning("Warning", "Please enter a URL.")
            return

        item_id = self.queue_display.insert(
            "",
            "end",
            values=("Pending", url),
            tags=(self.status_tags["Pending"],),
        )
        listbox_index = self.queue_listbox.size()
        display_index = listbox_index + 1
        entry = {
            "url": url,
            "item_id": item_id,
            "listbox_index": listbox_index,
            "display_index": display_index,
            "status": "Pending",
            "title": None,
        }
        self.download_queue.append(entry)

        self.queue_listbox.insert(tk.END, self.format_queue_listbox_entry(entry))

        threading.Thread(
            target=self.fetch_title_for_entry,
            args=(entry,),
            daemon=True,
        ).start()

        pending_total = sum(1 for item in self.download_queue if item["status"] == "Pending")
        self.status_var.set(f"Added to queue. Pending items: {pending_total}.")

        self.url_entry.delete(0, tk.END)

    def fetch_title_for_entry(self, entry):
        url = entry.get("url")
        if not url:
            return

        options = {
            "quiet": True,
            "skip_download": True,
            "no_warnings": True,
        }

        try:
            with yt_dlp.YoutubeDL(options) as ydl:
                info = ydl.extract_info(url, download=False)
            title = info.get("title") or "Unknown title"
        except Exception:
            title = "Title unavailable"

        self.master.after(0, lambda: self.apply_entry_title(entry, title))

    def apply_entry_title(self, entry, title):
        if entry not in self.download_queue:
            return

        entry["title"] = (title or "").replace("\n", " ").strip() or "Unknown title"

        index = entry.get("listbox_index")
        if index is not None and 0 <= index < self.queue_listbox.size():
            self.queue_listbox.delete(index)
            self.queue_listbox.insert(index, self.format_queue_listbox_entry(entry))
            entry["listbox_index"] = index

    def start_download(self):
        if self.downloading:
            messagebox.showinfo("Info", "A download is already in progress.")
            return

        pending_entries = [entry for entry in self.download_queue if entry["status"] == "Pending"]
        if not pending_entries:
            messagebox.showwarning("Warning", "No pending items in the queue.")
            return

        format_type = self.format_var.get()
        quality = self.quality_var.get()
        if not quality:
            messagebox.showwarning("Warning", "Please select a quality option.")
            return

        self.downloading = True
        self.progress_var.set(0.0)
        self.status_var.set(f"Preparing {len(pending_entries)} download(s)...")
        self.set_controls_state(disabled=True)

        worker = threading.Thread(
            target=self.process_download_queue,
            args=(pending_entries, format_type, quality),
            daemon=True,
        )
        worker.start()

    def set_controls_state(self, disabled):
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

    def process_download_queue(self, entries, format_type, quality):
        total = len(entries)
        errors = []
        completed = []

        for index, entry in enumerate(entries, start=1):
            url = entry["url"]
            self.master.after(0, self.mark_queue_status, entry, "Downloading")
            self.master.after(0, self.prepare_download_ui, index, total, url)
            try:
                self.download_youtube(url, format_type, quality, index, total)
            except Exception as error:
                errors.append((url, str(error)))
                self.master.after(0, self.mark_queue_status, entry, "Failed")
            else:
                completed.append(url)
                self.master.after(0, self.mark_queue_status, entry, "Completed")

        self.master.after(0, lambda: self.on_download_complete(format_type, errors, completed))

    def mark_queue_status(self, entry, status):
        entry["status"] = status
        item_id = entry["item_id"]
        self.queue_display.item(item_id, values=(status, entry["url"]))
        tag = self.status_tags.get(status)
        if tag:
            self.queue_display.item(item_id, tags=(tag,))

        index = entry.get("listbox_index")
        if index is not None and 0 <= index < self.queue_listbox.size():
            self.queue_listbox.delete(index)
            self.queue_listbox.insert(index, self.format_queue_listbox_entry(entry))

    def format_queue_listbox_entry(self, entry):
        status = entry.get("status", "Pending")
        display_index = entry.get("display_index", 0)
        url = entry.get("url", "")
        title = entry.get("title")
        title_text = (title or "Fetching title...").replace("\n", " ").strip() or "Unknown title"
        if len(title_text) > 80:
            title_text = f"{title_text[:77]}..."
        return f"{display_index:02d}. [{status}] {url} | {title_text}"


    def prepare_download_ui(self, index, total, url):
        self.progress_var.set(0.0)
        display_url = url if len(url) <= 60 else f"{url[:57]}..."
        self.status_var.set(f"Starting {index}/{total}: {display_url}")

    def download_youtube(self, url, format_type, quality, index, total):
        if format_type == "audio":
            download_dir = os.path.join(os.path.dirname(__file__), "py_downloader", "audio")
        else:
            download_dir = os.path.join(os.path.dirname(__file__), "py_downloader", "video")
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
            self.on_progress(data, url, index, total)

        ydl_opts["progress_hooks"] = [progress_hook]

        with yt_dlp.YoutubeDL(ydl_opts) as ydl:
            ydl.download([url])

    def on_progress(self, data, url, index, total):
        status = data.get("status")
        if status == "downloading":
            percent_value, percent_text = self._extract_percent(data)
            speed_text = data.get("_speed_str") or ""
            eta_text = data.get("_eta_str") or ""
            message_parts = [f"Downloading {index}/{total}", percent_text]
            if speed_text:
                message_parts.append(speed_text)
            if eta_text:
                message_parts.append(f"ETA {eta_text}")
            message = " | ".join(part for part in message_parts if part)
            self.master.after(0, self.update_progress_ui, percent_value, message)
        elif status == "finished":
            message = f"Processing download {index}/{total}..."
            self.master.after(0, self.update_progress_ui, 100.0, message)

    def update_progress_ui(self, value, message):
        self.progress_var.set(max(0.0, min(100.0, value)))
        self.status_var.set(message)

    @staticmethod
    def _extract_percent(data):
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

    def on_download_complete(self, format_type, errors, completed):
        self.downloading = False
        self.set_controls_state(disabled=False)

        success_count = len(completed)
        failure_count = len(errors)
        pending_count = sum(1 for entry in self.download_queue if entry["status"] == "Pending")

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
                self.open_download_folder(format_type)
            else:
                self.progress_var.set(0.0)
                self.status_var.set("No downloads were processed.")

    def open_download_folder(self, format_type):
        download_dir = os.path.join(os.path.dirname(__file__), "py_downloader", format_type)
        if not os.path.isdir(download_dir):
            return
        if os.name == "nt":
            os.startfile(download_dir)
        elif sys.platform == "darwin":
            subprocess.call(["open", download_dir])
        else:
            subprocess.call(["xdg-open", download_dir])


if __name__ == "__main__":
    root = tk.Tk()
    try:
        app = StreamQApp(root)
    except Exception as error:
        messagebox.showerror("Error", str(error))
        root.destroy()
        sys.exit(1)
    root.mainloop()
