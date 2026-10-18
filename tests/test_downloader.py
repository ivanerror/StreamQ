"""Tests for StreamQ download option building and progress parsing."""

from unittest import mock

from streamq.core.downloader import DownloadManager


def capture_options(manager, format_type, quality):
    """Run download_video with yt-dlp mocked and return the options passed to it."""
    with mock.patch("streamq.core.downloader.yt_dlp.YoutubeDL") as ydl_cls, \
            mock.patch("streamq.core.downloader.os.makedirs"):
        manager.download_video("https://youtu.be/x", format_type, quality)
    return ydl_cls.call_args[0][0]


def test_audio_options_fall_back_to_best_audio():
    options = capture_options(DownloadManager("/ffmpeg"), "audio", "192")
    assert options["format"] == "bestaudio[abr<=192]/bestaudio/best"
    assert options["postprocessors"][0]["preferredquality"] == "192"
    assert options["ffmpeg_location"] == "/ffmpeg"


def test_video_options_prefer_mp4_compatible_codecs():
    options = capture_options(DownloadManager("/ffmpeg"), "video", "720")
    assert "height<=720" in options["format"]
    assert options["format_sort"][0] == "res:720"
    assert options["merge_output_format"] == "mp4"


def test_js_runtimes_are_passed_to_yt_dlp():
    manager = DownloadManager("/ffmpeg")
    assert "js_runtimes" not in capture_options(manager, "audio", "128")

    manager.set_js_runtimes({"deno": {"path": "/bin/deno"}})
    assert capture_options(manager, "video", "1080")["js_runtimes"] == {"deno": {"path": "/bin/deno"}}


def test_fetch_title_uses_flat_extraction():
    manager = DownloadManager("/ffmpeg")
    with mock.patch("streamq.core.downloader.yt_dlp.YoutubeDL") as ydl_cls:
        ydl_cls.return_value.__enter__.return_value.extract_info.return_value = {"title": " A\nB "}
        assert manager.fetch_video_title("https://youtu.be/x") == "A B"
    assert ydl_cls.call_args[0][0]["extract_flat"] == "in_playlist"


def test_fetch_title_handles_errors():
    with mock.patch("streamq.core.downloader.yt_dlp.YoutubeDL", side_effect=Exception("boom")):
        assert DownloadManager("/ffmpeg").fetch_video_title("https://youtu.be/x") == "Title unavailable"


def test_extract_percent_from_string_and_bytes():
    assert DownloadManager._extract_percent({"_percent_str": " 42.5%"}) == (42.5, "42.5%")
    assert DownloadManager._extract_percent({"total_bytes": 200, "downloaded_bytes": 50}) == (25.0, "25.0%")
    assert DownloadManager._extract_percent({}) == (0.0, "0%")
