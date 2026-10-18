"""Tests for FFmpeg and JavaScript runtime detection."""

from unittest import mock

from streamq.utils import ffmpeg, jsruntime


def test_resolve_deno_asset():
    cases = {
        ("Windows", "AMD64"): "deno-x86_64-pc-windows-msvc",
        ("Linux", "aarch64"): "deno-aarch64-unknown-linux-gnu",
        ("Darwin", "arm64"): "deno-aarch64-apple-darwin",
        ("Linux", "armv7l"): None,
    }
    for (system, machine), expected in cases.items():
        with mock.patch("platform.system", return_value=system), \
                mock.patch("platform.machine", return_value=machine):
            assert jsruntime.resolve_deno_asset() == expected


def test_find_js_runtimes_prefers_bundled_deno(tmp_path):
    deno = tmp_path / ("deno.exe" if jsruntime.os.name == "nt" else "deno")
    deno.write_text("")
    with mock.patch.object(jsruntime, "get_runtime_dir", return_value=str(tmp_path)):
        assert jsruntime.find_js_runtimes() == {"deno": {"path": str(deno)}}


def test_find_js_runtimes_falls_back_to_path(tmp_path):
    def which(name):
        return "/usr/bin/node" if name == "node" else None

    with mock.patch.object(jsruntime, "get_runtime_dir", return_value=str(tmp_path)), \
            mock.patch("shutil.which", side_effect=which):
        assert jsruntime.find_js_runtimes() == {"node": {"path": "/usr/bin/node"}}


def test_ensure_js_runtimes_downloads_when_missing():
    with mock.patch.object(jsruntime, "find_js_runtimes", return_value={}), \
            mock.patch.object(jsruntime, "download_deno", return_value="/rt/deno"):
        assert jsruntime.ensure_js_runtimes() == {"deno": {"path": "/rt/deno"}}


def test_ensure_ffmpeg_uses_system_install(tmp_path):
    with mock.patch("streamq.config.get_project_root", return_value=str(tmp_path)), \
            mock.patch("shutil.which", side_effect=lambda name, path=None: f"/usr/bin/{name}"):
        assert ffmpeg.ensure_ffmpeg() == "/usr/bin"


def test_extract_binaries_from_tar_xz(tmp_path):
    import io
    import os
    import tarfile

    archive = tmp_path / "ffmpeg.tar.xz"
    with tarfile.open(archive, "w:xz") as tar:
        for name in ("ffmpeg-master/bin/ffmpeg", "ffmpeg-master/bin/ffprobe", "ffmpeg-master/LICENSE"):
            info = tarfile.TarInfo(name)
            info.size = 4
            tar.addfile(info, io.BytesIO(b"data"))

    bin_dir = tmp_path / "bin"
    bin_dir.mkdir()
    ffmpeg.extract_binaries(str(archive), {"ffmpeg", "ffprobe"}, str(bin_dir))
    assert sorted(os.listdir(bin_dir)) == ["ffmpeg", "ffprobe"]
    if os.name != "nt":
        assert os.access(bin_dir / "ffmpeg", os.X_OK)


def test_ensure_ffmpeg_prefers_bundled_build(tmp_path):
    import os

    exe = ".exe" if os.name == "nt" else ""
    bundled = tmp_path / "bundle" / "ffmpeg"
    bundled.mkdir(parents=True)
    for name in ("ffmpeg", "ffprobe"):
        (bundled / (name + exe)).write_text("")
    with mock.patch("sys._MEIPASS", str(tmp_path / "bundle"), create=True), \
            mock.patch("streamq.config.get_project_root", return_value=str(tmp_path)), \
            mock.patch("shutil.which", side_effect=lambda name, path=None: f"/usr/bin/{name}"):
        assert ffmpeg.ensure_ffmpeg() == str(bundled)
