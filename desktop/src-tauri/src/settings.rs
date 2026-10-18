use serde::{Deserialize, Serialize};

use crate::engine::Format;

pub const AUDIO_QUALITIES: [u32; 5] = [64, 128, 192, 256, 320];
// 1440p and 2160p are only available as VP9/AV1 on YouTube
pub const VIDEO_QUALITIES: [u32; 8] = [144, 240, 360, 480, 720, 1080, 1440, 2160];

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub output_dir: String,
    pub default_format: Format,
    pub default_video_quality: u32,
    pub default_audio_quality: u32,
    pub parallel_downloads: u8,
    pub filename_template: String,
    pub auto_update_engine: bool,
    pub notify_on_finish: bool,
    pub suggest_clipboard_links: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            output_dir: String::new(),
            default_format: Format::Video,
            default_video_quality: 1080,
            default_audio_quality: 320,
            parallel_downloads: 2,
            filename_template: "%(title)s.%(ext)s".into(),
            auto_update_engine: true,
            notify_on_finish: true,
            suggest_clipboard_links: false,
        }
    }
}

impl Settings {
    /// Clamp values a hand-edited settings file or the UI could get wrong.
    pub fn normalized(mut self, default_output_dir: &str) -> Self {
        if self.output_dir.trim().is_empty() {
            self.output_dir = default_output_dir.to_string();
        }
        self.parallel_downloads = self.parallel_downloads.clamp(1, 3);
        if !VIDEO_QUALITIES.contains(&self.default_video_quality) {
            self.default_video_quality = 1080;
        }
        if !AUDIO_QUALITIES.contains(&self.default_audio_quality) {
            self.default_audio_quality = 320;
        }
        let template = self.filename_template.trim();
        // yt-dlp needs the extension placeholder, and the template must stay inside the output folder
        if !template.contains("%(ext)s") || template.contains("..") || template.starts_with('/') || template.starts_with('\\') {
            self.filename_template = Settings::default().filename_template;
        }
        self
    }

    pub fn quality_for(&self, format: Format) -> u32 {
        match format {
            Format::Audio => self.default_audio_quality,
            Format::Video => self.default_video_quality,
        }
    }
}

pub fn valid_quality(format: Format, quality: u32) -> bool {
    match format {
        Format::Audio => AUDIO_QUALITIES.contains(&quality),
        Format::Video => VIDEO_QUALITIES.contains(&quality),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_bad_values() {
        let settings = Settings {
            output_dir: " ".into(),
            parallel_downloads: 9,
            default_video_quality: 999,
            filename_template: "../%(title)s".into(),
            ..Settings::default()
        }
        .normalized("/home/me/Downloads/StreamQ");
        assert_eq!(settings.output_dir, "/home/me/Downloads/StreamQ");
        assert_eq!(settings.parallel_downloads, 3);
        assert_eq!(settings.default_video_quality, 1080);
        assert_eq!(settings.filename_template, "%(title)s.%(ext)s");
    }

    #[test]
    fn missing_fields_take_defaults() {
        let settings: Settings = serde_json::from_str(r#"{"parallelDownloads": 1}"#).unwrap();
        assert_eq!(settings.parallel_downloads, 1);
        assert_eq!(settings.default_format, Format::Video);
        assert!(settings.auto_update_engine);
    }

    #[test]
    fn quality_lists() {
        assert!(valid_quality(Format::Video, 2160));
        assert!(!valid_quality(Format::Audio, 2160));
    }
}
