//! The download queue as plain data: items, their state transitions and persistence.
//! Scheduling and process supervision live in `app.rs`.

use std::collections::HashMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};

use crate::engine::{Format, Metadata, Progress};
use crate::error::Result;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Status {
    Waiting,
    Downloading,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum InfoState {
    Pending,
    Ready,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Item {
    pub id: String,
    pub url: String,
    pub title: Option<String>,
    pub uploader: Option<String>,
    pub duration: Option<f64>,
    pub thumbnail: Option<String>,
    pub is_playlist: bool,
    pub entry_count: Option<u32>,
    pub info: InfoState,
    pub format: Format,
    pub quality: u32,
    pub status: Status,
    pub progress: Option<Progress>,
    pub error: Option<String>,
    pub output: Option<String>,
    pub added_at: u64,
    pub finished_at: Option<u64>,
}

pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or_default()
}

/// Accept only http(s) links with a host, like StreamQ 1.x.
pub fn valid_url(url: &str) -> bool {
    match tauri::Url::parse(url.trim()) {
        Ok(parsed) => matches!(parsed.scheme(), "http" | "https") && parsed.host_str().is_some_and(|h| !h.is_empty()),
        Err(_) => false,
    }
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Queue {
    pub items: Vec<Item>,
    /// yt-dlp output of failed downloads, shown in "View log"
    pub logs: HashMap<String, Vec<String>>,
    pub paused: bool,
}

impl Queue {
    /// Load the saved queue; downloads interrupted by quitting go back to Waiting.
    pub fn load(path: &Path) -> Queue {
        let mut queue: Queue = std::fs::read(path)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        for item in &mut queue.items {
            if item.status == Status::Downloading {
                item.status = Status::Waiting;
            }
            if item.status != Status::Completed {
                item.progress = None;
            }
            if item.info == InfoState::Pending && item.title.is_some() {
                item.info = InfoState::Ready;
            }
        }
        queue.logs.retain(|id, _| queue.items.iter().any(|item| &item.id == id));
        queue
    }

    /// Write atomically so a crash never leaves a truncated queue file.
    pub fn save(&self, path: &Path) -> Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let tmp = path.with_extension("json.tmp");
        std::fs::write(&tmp, serde_json::to_vec_pretty(self)?)?;
        std::fs::rename(&tmp, path)?;
        Ok(())
    }

    pub fn add(&mut self, url: &str, format: Format, quality: u32) -> &Item {
        self.items.push(Item {
            id: uuid::Uuid::new_v4().to_string(),
            url: url.trim().to_string(),
            title: None,
            uploader: None,
            duration: None,
            thumbnail: None,
            is_playlist: false,
            entry_count: None,
            info: InfoState::Pending,
            format,
            quality,
            status: Status::Waiting,
            progress: None,
            error: None,
            output: None,
            added_at: now(),
            finished_at: None,
        });
        self.items.last().expect("just pushed")
    }

    pub fn get(&self, id: &str) -> Option<&Item> {
        self.items.iter().find(|item| item.id == id)
    }

    pub fn get_mut(&mut self, id: &str) -> Option<&mut Item> {
        self.items.iter_mut().find(|item| item.id == id)
    }

    pub fn apply_metadata(&mut self, id: &str, metadata: std::result::Result<&Metadata, String>) {
        let Some(item) = self.get_mut(id) else { return };
        match metadata {
            Ok(meta) => {
                item.title = meta.title.clone().or(item.title.take());
                item.uploader = meta.uploader.clone();
                item.duration = meta.duration;
                item.thumbnail = meta.thumbnail.clone();
                item.is_playlist = meta.is_playlist;
                item.entry_count = meta.entry_count;
                item.info = InfoState::Ready;
            }
            Err(_) => item.info = InfoState::Failed,
        }
    }

    /// Next item to start, skipping ones already running.
    pub fn next_waiting(&self, running: &dyn Fn(&str) -> bool) -> Option<String> {
        self.items
            .iter()
            .find(|item| item.status == Status::Waiting && !running(&item.id))
            .map(|item| item.id.clone())
    }

    pub fn set_status(&mut self, id: &str, status: Status) {
        if let Some(item) = self.get_mut(id) {
            item.status = status;
            match status {
                Status::Downloading => {
                    item.error = None;
                    item.progress = Some(Progress::default());
                }
                Status::Waiting | Status::Cancelled => item.progress = None,
                Status::Completed | Status::Failed => item.finished_at = Some(now()),
            }
        }
    }

    /// Put failed and cancelled items (all, or the given ones) back in line.
    pub fn retry(&mut self, ids: Option<&[String]>) -> usize {
        let mut count = 0;
        for item in &mut self.items {
            let selected = ids.is_none_or(|ids| ids.contains(&item.id));
            if selected && matches!(item.status, Status::Failed | Status::Cancelled) {
                item.status = Status::Waiting;
                item.error = None;
                item.progress = None;
                item.finished_at = None;
                count += 1;
            }
        }
        for id in self.items.iter().filter(|i| i.status == Status::Waiting).map(|i| i.id.clone()).collect::<Vec<_>>() {
            self.logs.remove(&id);
        }
        count
    }

    pub fn remove(&mut self, id: &str) -> Option<Item> {
        let index = self.items.iter().position(|item| item.id == id)?;
        self.logs.remove(id);
        Some(self.items.remove(index))
    }

    pub fn clear_completed(&mut self) -> usize {
        let before = self.items.len();
        self.items.retain(|item| item.status != Status::Completed);
        before - self.items.len()
    }

    /// Replace a playlist item by one item per entry, at the same position.
    pub fn expand(&mut self, id: &str, entries: &[String]) -> usize {
        let Some(index) = self.items.iter().position(|item| item.id == id) else { return 0 };
        let playlist = self.items.remove(index);
        let added: Vec<Item> = entries
            .iter()
            .map(|url| Item {
                id: uuid::Uuid::new_v4().to_string(),
                url: url.clone(),
                title: None,
                uploader: None,
                duration: None,
                thumbnail: None,
                is_playlist: false,
                entry_count: None,
                info: InfoState::Pending,
                format: playlist.format,
                quality: playlist.quality,
                status: Status::Waiting,
                progress: None,
                error: None,
                output: None,
                added_at: now(),
                finished_at: None,
            })
            .collect();
        let count = added.len();
        self.items.splice(index..index, added);
        count
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_urls() {
        assert!(valid_url("https://www.youtube.com/watch?v=abc"));
        assert!(valid_url(" http://youtu.be/abc "));
        assert!(!valid_url("ftp://example.com/file"));
        assert!(!valid_url("youtube.com/watch?v=abc"));
        assert!(!valid_url("https://"));
    }

    #[test]
    fn retry_resets_failed_and_cancelled_only() {
        let mut queue = Queue::default();
        let ids: Vec<String> = (0..3).map(|i| queue.add(&format!("https://x/{i}"), Format::Audio, 320).id.clone()).collect();
        queue.set_status(&ids[0], Status::Failed);
        queue.get_mut(&ids[0]).unwrap().error = Some("boom".into());
        queue.logs.insert(ids[0].clone(), vec!["ERROR: boom".into()]);
        queue.set_status(&ids[1], Status::Cancelled);
        queue.set_status(&ids[2], Status::Completed);

        assert_eq!(queue.retry(None), 2);
        let statuses: Vec<Status> = queue.items.iter().map(|i| i.status).collect();
        assert_eq!(statuses, [Status::Waiting, Status::Waiting, Status::Completed]);
        assert!(queue.items[0].error.is_none());
        assert!(queue.logs.is_empty());
    }

    #[test]
    fn retry_selected_items() {
        let mut queue = Queue::default();
        let a = queue.add("https://x/a", Format::Video, 1080).id.clone();
        let b = queue.add("https://x/b", Format::Video, 1080).id.clone();
        queue.set_status(&a, Status::Failed);
        queue.set_status(&b, Status::Failed);
        assert_eq!(queue.retry(Some(&[b.clone()])), 1);
        assert_eq!(queue.get(&a).unwrap().status, Status::Failed);
        assert_eq!(queue.get(&b).unwrap().status, Status::Waiting);
    }

    #[test]
    fn next_waiting_skips_running_items() {
        let mut queue = Queue::default();
        let a = queue.add("https://x/a", Format::Video, 1080).id.clone();
        let b = queue.add("https://x/b", Format::Video, 1080).id.clone();
        assert_eq!(queue.next_waiting(&|_| false), Some(a.clone()));
        assert_eq!(queue.next_waiting(&|id| id == a), Some(b));
    }

    #[test]
    fn load_resumes_interrupted_downloads() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("queue.json");
        let mut queue = Queue::default();
        let id = queue.add("https://x/a", Format::Video, 1080).id.clone();
        queue.set_status(&id, Status::Downloading);
        queue.save(&path).unwrap();

        let loaded = Queue::load(&path);
        assert_eq!(loaded.items[0].status, Status::Waiting);
        assert!(loaded.items[0].progress.is_none());
        assert!(Queue::load(&dir.path().join("missing.json")).items.is_empty());
    }

    #[test]
    fn expands_playlists_in_place() {
        let mut queue = Queue::default();
        let first = queue.add("https://x/first", Format::Audio, 256).id.clone();
        let playlist = queue.add("https://x/playlist?list=1", Format::Audio, 256).id.clone();
        let last = queue.add("https://x/last", Format::Audio, 256).id.clone();
        assert_eq!(queue.expand(&playlist, &["https://x/1".into(), "https://x/2".into()]), 2);
        let urls: Vec<&str> = queue.items.iter().map(|i| i.url.as_str()).collect();
        assert_eq!(urls, ["https://x/first", "https://x/1", "https://x/2", "https://x/last"]);
        assert_eq!(queue.items[1].quality, 256);
        assert!(queue.get(&first).is_some() && queue.get(&last).is_some());
    }
}
