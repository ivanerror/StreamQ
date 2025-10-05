# YouTube Downloader (GUI)

Aplikasi desktop sederhana untuk mengunduh video/audio dari YouTube menggunakan `yt-dlp` dengan antarmuka GUI (Tkinter). Di Windows, aplikasi akan otomatis menyiapkan FFmpeg sehingga Anda bisa langsung pakai.

## Prasyarat
- Windows 10/11 (direkomendasikan). macOS/Linux juga bisa, lihat catatan di bawah.
- Python 3.8 atau lebih baru terpasang di sistem.
- Koneksi internet (untuk instalasi dependensi dan unduhan FFmpeg di Windows).
- Git (opsional, untuk kloning/push ke GitHub).

## Instalasi & Menjalankan (Windows)

Cara termudah adalah melalui skrip `run.bat` yang sudah disediakan:

1. Klik dua kali `run.bat` dari File Explorer, atau jalankan via Terminal di folder proyek:
   ```bat
   run.bat
   ```
2. Skrip akan:
   - Membuat virtual environment di `.venv/` (jika belum ada)
   - Menginstal dependensi dari `requirements.txt`
   - Menjalankan aplikasi GUI (`python main.py`)

Setelah jendela aplikasi terbuka, tempel URL YouTube, tambahkan ke antrean, lalu mulai unduh. Hasil audio/video secara default disimpan di dalam folder `py_downloader/` (lihat subfolder `audio` dan `video`).

## Menjalankan Secara Manual (Windows/macOS/Linux)

Jika tidak ingin memakai `run.bat`, Anda bisa menjalankan manual dengan langkah standar Python:

```bash
python -m venv .venv
# Windows
.venv\\Scripts\\activate
# macOS/Linux
# source .venv/bin/activate

python -m pip install --upgrade pip
pip install -r requirements.txt
python main.py
```

Catatan FFmpeg:
- Windows: aplikasi akan otomatis mengunduh dan menyiapkan FFmpeg di folder `ffmpeg_support/` saat pertama kali dijalankan.
- macOS/Linux: unduhan otomatis FFmpeg tidak tersedia. Silakan instal FFmpeg secara manual dan pastikan perintah `ffmpeg` serta `ffprobe` ada di `PATH` (contoh: `brew install ffmpeg` di macOS atau `sudo apt install ffmpeg` di Ubuntu).

## Struktur Proyek (ringkas)
- `main.py` — kode utama aplikasi GUI.
- `run.bat` — skrip bantu untuk membuat venv, instal dependensi, dan menjalankan app.
- `requirements.txt` — daftar dependensi Python.
- `py_downloader/` — lokasi hasil unduhan (audio/video).
- `ffmpeg_support/` — lokasi FFmpeg (Windows, otomatis).
- `.venv/` — virtual environment lokal (diabaikan Git).

## Inisialisasi Git & Push ke GitHub

Jika Anda belum menginisialisasi repo Git dan ingin mendorong ke GitHub:

```bash
# Inisialisasi repo lokal (gunakan main sebagai default branch)
git init
git branch -M main

# Tambahkan semua file (file besar output sudah di-ignore via .gitignore)
git add .
git commit -m "Initial commit"

# Buat repo baru di GitHub terlebih dahulu (via web), lalu hubungkan:
git remote add origin https://github.com/<username>/<repo>.git
git push -u origin main
```

Alternatif: Jika Anda memakai GitHub CLI (`gh`), Anda bisa membuat repo langsung:

```bash
gh repo create <username>/<repo> --source . --public --push
```

## Troubleshooting
- Python tidak ditemukan: pastikan `python --version` menampilkan versi >= 3.8.
- Gagal instal dependensi: pastikan koneksi internet lancar, lalu jalankan kembali `run.bat`/perintah pip.
- FFmpeg tidak tersedia (macOS/Linux): instal FFmpeg dan pastikan ada di `PATH`.
- Unduhan gagal: coba ulangi, periksa URL, atau perbarui `yt-dlp` (`pip install -U yt-dlp`).

