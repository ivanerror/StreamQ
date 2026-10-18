"""Tests for queue cancel/retry behaviour."""

import threading
from unittest import mock

from yt_dlp.utils import DownloadCancelled

from streamq.core.downloader import DownloadQueue


def make_queue(download_side_effect):
    manager = mock.Mock()
    manager.fetch_video_title.return_value = "title"
    manager.download_video.side_effect = download_side_effect
    queue = DownloadQueue(manager)
    done = threading.Event()
    result = {}

    def on_complete(format_type, errors, completed, cancelled):
        result.update(errors=errors, completed=completed, cancelled=cancelled)
        done.set()

    queue.set_completion_callback(on_complete)
    return queue, done, result


def test_cancel_marks_current_item_and_keeps_rest_pending():
    def download(url, *args, cancel_event=None):
        cancel_event.set()  # user pressed Cancel during the first download
        raise DownloadCancelled()

    queue, done, result = make_queue(download)
    queue.add_to_queue("https://a", "1")
    queue.add_to_queue("https://b", "2")
    queue.process_queue("audio", "320")
    assert done.wait(5)

    assert [e["status"] for e in queue.queue] == ["Cancelled", "Pending"]
    assert result == {"errors": [], "completed": [], "cancelled": True}


def test_retry_resets_failed_and_cancelled():
    queue, done, result = make_queue(Exception("network down"))
    queue.add_to_queue("https://a", "1")
    queue.process_queue("video", "2160")
    assert done.wait(5)
    assert queue.queue[0]["status"] == "Failed"
    assert result["cancelled"] is False

    queue.queue.append({"url": "https://b", "item_id": "2", "status": "Cancelled", "title": None})
    queue.queue.append({"url": "https://c", "item_id": "3", "status": "Completed", "title": None})
    assert queue.retry_failed() == 2
    assert [e["status"] for e in queue.queue] == ["Pending", "Pending", "Completed"]


def test_new_run_clears_previous_cancel():
    queue, done, result = make_queue(None)
    queue.cancel_event.set()
    queue.add_to_queue("https://a", "1")
    queue.process_queue("audio", "128")
    assert done.wait(5)
    assert queue.queue[0]["status"] == "Completed"
    assert result["cancelled"] is False
