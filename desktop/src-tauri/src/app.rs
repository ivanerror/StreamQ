//! Application state shared by commands and background tasks: the queue, the
//! parallel job scheduler, component setup, and events pushed to the UI.

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, Instant};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_notification::NotificationExt;
use tokio::sync::{watch, Semaphore};

use crate::components::{self, Kind, Layout, Source, State, Status as ComponentStatus, Tools};
use crate::engine::{self, Format, Job, Outcome, Progress, StopReason};
use crate::error::{Error, Result};
use crate::queue::{self, InfoState, Item, Queue, Status};
use crate::settings::{self, Settings};

pub const EVENT_QUEUE: &str = "queue-changed";
pub const EVENT_PROGRESS: &str = "item-progress";
pub const EVENT_COMPONENTS: &str = "components-changed";

const PROGRESS_INTERVAL: Duration = Duration::from_millis(250);
const UPDATE_CHECK_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);

pub struct Paths {
    pub queue_file: PathBuf,
    pub settings_file: PathBuf,
    pub update_check_file: PathBuf,
    pub default_output: String,
    pub layout: Layout,
}

struct Components {
    statuses: Vec<ComponentStatus>,
    tools: Tools,
}

impl Components {
    fn status_mut(&mut self, kind: Kind) -> &mut ComponentStatus {
        self.statuses.iter_mut().find(|s| s.kind == kind).expect("every kind has a status")
    }

    fn ready(&self) -> bool {
        self.statuses.iter().all(|s| s.state == State::Ready)
    }
}

pub struct App {
    handle: AppHandle,
    pub paths: Paths,
    client: reqwest::Client,
    queue: Mutex<Queue>,
    settings: RwLock<Settings>,
    components: Mutex<Components>,
    running: Mutex<HashMap<String, watch::Sender<Option<StopReason>>>>,
    removing: Mutex<HashSet<String>>,
    last_progress: Mutex<HashMap<String, Instant>>,
    metadata_slots: Semaphore,
    setup_lock: tokio::sync::Mutex<()>,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct QueueSnapshot {
    pub items: Vec<Item>,
    pub paused: bool,
}

#[derive(Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ProgressEvent<'a> {
    id: &'a str,
    progress: &'a Progress,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Snapshot {
    pub queue: QueueSnapshot,
    pub settings: Settings,
    pub components: Vec<ComponentStatus>,
    pub audio_qualities: Vec<u32>,
    pub video_qualities: Vec<u32>,
    pub version: &'static str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AddResult {
    pub added: usize,
    pub invalid: Vec<String>,
}

impl App {
    pub fn new(handle: AppHandle, paths: Paths) -> Result<Arc<App>> {
        let settings: Settings = std::fs::read(&paths.settings_file)
            .ok()
            .and_then(|bytes| serde_json::from_slice(&bytes).ok())
            .unwrap_or_default();
        let settings = settings.normalized(&paths.default_output);
        let queue = Queue::load(&paths.queue_file);
        Ok(Arc::new(App {
            handle,
            client: components::http_client()?,
            queue: Mutex::new(queue),
            settings: RwLock::new(settings),
            components: Mutex::new(Components {
                statuses: Kind::ALL.iter().map(|k| ComponentStatus::new(*k)).collect(),
                tools: Tools::default(),
            }),
            running: Mutex::new(HashMap::new()),
            removing: Mutex::new(HashSet::new()),
            last_progress: Mutex::new(HashMap::new()),
            metadata_slots: Semaphore::new(3),
            setup_lock: tokio::sync::Mutex::new(()),
            paths,
        }))
    }

    // ----- snapshots and events -----

    pub fn snapshot(&self) -> Snapshot {
        Snapshot {
            queue: self.queue_snapshot(),
            settings: self.settings(),
            components: self.components.lock().unwrap().statuses.clone(),
            audio_qualities: settings::AUDIO_QUALITIES.to_vec(),
            video_qualities: settings::VIDEO_QUALITIES.to_vec(),
            version: env!("CARGO_PKG_VERSION"),
        }
    }

    fn queue_snapshot(&self) -> QueueSnapshot {
        let queue = self.queue.lock().unwrap();
        QueueSnapshot { items: queue.items.clone(), paused: queue.paused }
    }

    pub fn settings(&self) -> Settings {
        self.settings.read().unwrap().clone()
    }

    fn tools(&self) -> Tools {
        self.components.lock().unwrap().tools.clone()
    }

    fn components_ready(&self) -> bool {
        self.components.lock().unwrap().ready()
    }

    /// Save the queue and tell the UI; called after every structural change.
    fn queue_changed(&self) {
        let snapshot = {
            let queue = self.queue.lock().unwrap();
            if let Err(error) = queue.save(&self.paths.queue_file) {
                log::error!("could not save queue: {error}");
            }
            QueueSnapshot { items: queue.items.clone(), paused: queue.paused }
        };
        let _ = self.handle.emit(EVENT_QUEUE, snapshot);
    }

    fn components_changed(&self) {
        let statuses = self.components.lock().unwrap().statuses.clone();
        let _ = self.handle.emit(EVENT_COMPONENTS, statuses);
    }

    fn update_component(&self, kind: Kind, update: impl FnOnce(&mut ComponentStatus)) {
        update(self.components.lock().unwrap().status_mut(kind));
        self.components_changed();
    }

    // ----- queue commands -----

    pub fn add_urls(self: &Arc<Self>, urls: Vec<String>, format: Option<Format>, quality: Option<u32>) -> AddResult {
        let settings = self.settings();
        let format = format.unwrap_or(settings.default_format);
        let quality = quality
            .filter(|q| settings::valid_quality(format, *q))
            .unwrap_or_else(|| settings.quality_for(format));
        let mut invalid = Vec::new();
        let mut added = Vec::new();
        {
            let mut queue = self.queue.lock().unwrap();
            for url in urls.iter().map(|u| u.trim()).filter(|u| !u.is_empty()) {
                if queue::valid_url(url) {
                    added.push(queue.add(url, format, quality).id.clone());
                } else {
                    invalid.push(url.to_string());
                }
            }
        }
        if !added.is_empty() {
            self.queue_changed();
            for id in &added {
                self.spawn_metadata(id.clone());
            }
            self.schedule();
        }
        AddResult { added: added.len(), invalid }
    }

    pub fn set_item_options(self: &Arc<Self>, id: &str, format: Format, quality: u32) -> Result<()> {
        if !settings::valid_quality(format, quality) {
            return Err(Error::msg("Unsupported quality for this format"));
        }
        {
            let mut queue = self.queue.lock().unwrap();
            let item = queue.get_mut(id).ok_or_else(|| Error::msg("Item not found"))?;
            if item.status == Status::Downloading {
                return Err(Error::msg("Cannot change a download that is running"));
            }
            item.format = format;
            item.quality = quality;
        }
        self.queue_changed();
        Ok(())
    }

    pub fn retry(self: &Arc<Self>, ids: Option<Vec<String>>) -> usize {
        let count = self.queue.lock().unwrap().retry(ids.as_deref());
        if count > 0 {
            self.queue_changed();
            self.schedule();
        }
        count
    }

    fn stop(&self, id: &str, reason: StopReason) -> bool {
        match self.running.lock().unwrap().get(id) {
            Some(sender) => {
                let _ = sender.send(Some(reason));
                true
            }
            None => false,
        }
    }

    pub fn cancel(self: &Arc<Self>, id: &str) {
        if !self.stop(id, StopReason::Cancel) {
            let mut queue = self.queue.lock().unwrap();
            if queue.get(id).is_some_and(|i| i.status == Status::Waiting) {
                queue.set_status(id, Status::Cancelled);
            }
        }
        self.queue_changed();
    }

    pub fn cancel_all(self: &Arc<Self>) {
        let ids: Vec<String> = self.running.lock().unwrap().keys().cloned().collect();
        for id in &ids {
            self.stop(id, StopReason::Cancel);
        }
        {
            let mut queue = self.queue.lock().unwrap();
            let waiting: Vec<String> = queue.items.iter().filter(|i| i.status == Status::Waiting).map(|i| i.id.clone()).collect();
            for id in waiting {
                queue.set_status(&id, Status::Cancelled);
            }
        }
        self.queue_changed();
    }

    /// Pausing stops running downloads and returns them to Waiting; yt-dlp resumes the .part file later.
    pub fn set_paused(self: &Arc<Self>, paused: bool) {
        self.queue.lock().unwrap().paused = paused;
        if paused {
            let ids: Vec<String> = self.running.lock().unwrap().keys().cloned().collect();
            for id in &ids {
                self.stop(id, StopReason::Pause);
            }
        }
        self.queue_changed();
        if !paused {
            self.schedule();
        }
    }

    pub fn remove(self: &Arc<Self>, id: &str) {
        if self.stop(id, StopReason::Cancel) {
            // Removed once the process has exited, in finish_job
            self.removing.lock().unwrap().insert(id.to_string());
        } else {
            self.queue.lock().unwrap().remove(id);
        }
        self.queue_changed();
    }

    pub fn clear_completed(self: &Arc<Self>) -> usize {
        let count = self.queue.lock().unwrap().clear_completed();
        self.queue_changed();
        count
    }

    pub fn log(&self, id: &str) -> Vec<String> {
        self.queue.lock().unwrap().logs.get(id).cloned().unwrap_or_default()
    }

    pub fn output_path(&self, id: &str) -> Option<String> {
        self.queue.lock().unwrap().get(id).and_then(|i| i.output.clone())
    }

    pub fn save_settings(self: &Arc<Self>, settings: Settings) -> Result<Settings> {
        let settings = settings.normalized(&self.paths.default_output);
        if let Some(parent) = self.paths.settings_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(&self.paths.settings_file, serde_json::to_vec_pretty(&settings)?)?;
        *self.settings.write().unwrap() = settings.clone();
        // More parallel slots may have opened up
        self.schedule();
        Ok(settings)
    }

    pub async fn expand_playlist(self: &Arc<Self>, id: &str) -> Result<usize> {
        let url = {
            let queue = self.queue.lock().unwrap();
            let item = queue.get(id).ok_or_else(|| Error::msg("Item not found"))?;
            if item.status == Status::Downloading {
                return Err(Error::msg("Stop the download before expanding the playlist"));
            }
            item.url.clone()
        };
        let metadata = engine::fetch_metadata(&url, &self.tools()).await?;
        if metadata.entries.is_empty() {
            return Err(Error::msg("This playlist has no videos StreamQ can list"));
        }
        let added = {
            let mut queue = self.queue.lock().unwrap();
            queue.expand(id, &metadata.entries)
        };
        self.queue_changed();
        self.fetch_pending_metadata();
        self.schedule();
        Ok(added)
    }

    // ----- metadata -----

    fn spawn_metadata(self: &Arc<Self>, id: String) {
        if self.tools().ytdlp.is_none() {
            // Fetched once setup has installed yt-dlp
            return;
        }
        let app = self.clone();
        tauri::async_runtime::spawn(async move {
            let _slot = app.metadata_slots.acquire().await;
            let Some(url) = app.queue.lock().unwrap().get(&id).map(|i| i.url.clone()) else { return };
            let result = engine::fetch_metadata(&url, &app.tools()).await;
            app.queue
                .lock()
                .unwrap()
                .apply_metadata(&id, result.as_ref().map_err(|e| e.to_string()));
            app.queue_changed();
        });
    }

    fn fetch_pending_metadata(self: &Arc<Self>) {
        let pending: Vec<String> = self
            .queue
            .lock()
            .unwrap()
            .items
            .iter()
            .filter(|i| i.info == InfoState::Pending)
            .map(|i| i.id.clone())
            .collect();
        for id in pending {
            self.spawn_metadata(id);
        }
    }

    // ----- scheduling -----

    /// Start waiting items until all parallel slots are busy.
    pub fn schedule(self: &Arc<Self>) {
        if !self.components_ready() {
            return;
        }
        let parallel = self.settings().parallel_downloads as usize;
        loop {
            let (id, job) = {
                let mut queue = self.queue.lock().unwrap();
                if queue.paused {
                    return;
                }
                let running = self.running.lock().unwrap();
                if running.len() >= parallel {
                    return;
                }
                let Some(id) = queue.next_waiting(&|id| running.contains_key(id)) else { return };
                drop(running);
                queue.set_status(&id, Status::Downloading);
                let item = queue.get(&id).expect("next_waiting returned an existing item");
                let settings = self.settings();
                let job = Job {
                    url: item.url.clone(),
                    format: item.format,
                    quality: item.quality,
                    is_playlist: item.is_playlist && !engine::wants_single_video(&item.url),
                    output_dir: settings.output_dir.clone(),
                    filename_template: settings.filename_template.clone(),
                };
                (id, job)
            };
            let (sender, receiver) = watch::channel(None);
            self.running.lock().unwrap().insert(id.clone(), sender);
            self.queue_changed();
            let app = self.clone();
            tauri::async_runtime::spawn(async move { app.run_job(id, job, receiver).await });
        }
    }

    async fn run_job(self: Arc<Self>, id: String, job: Job, stop: watch::Receiver<Option<StopReason>>) {
        let tools = self.tools();
        let app = self.clone();
        let progress_id = id.clone();
        let result = engine::run_download(&job, &tools, stop, move |progress| app.on_progress(&progress_id, progress)).await;
        self.finish_job(&id, result);
    }

    fn on_progress(&self, id: &str, progress: Progress) {
        if let Some(item) = self.queue.lock().unwrap().get_mut(id) {
            item.progress = Some(progress.clone());
        }
        // Throttle UI updates; always pass on post-processing changes
        let mut last = self.last_progress.lock().unwrap();
        let due = last.get(id).is_none_or(|t| t.elapsed() >= PROGRESS_INTERVAL) || progress.postprocessing;
        if due {
            last.insert(id.to_string(), Instant::now());
            let _ = self.handle.emit(EVENT_PROGRESS, ProgressEvent { id, progress: &progress });
        }
    }

    fn finish_job(self: &Arc<Self>, id: &str, result: Result<engine::Finished>) {
        self.running.lock().unwrap().remove(id);
        self.last_progress.lock().unwrap().remove(id);
        let removed = self.removing.lock().unwrap().remove(id);
        let mut completed_title = None;
        {
            let mut queue = self.queue.lock().unwrap();
            if removed {
                queue.remove(id);
            } else {
                let (status, error, output, log) = match result {
                    Ok(finished) => match finished.outcome {
                        Outcome::Completed { output } => (Status::Completed, None, output, None),
                        Outcome::Stopped(StopReason::Cancel) => (Status::Cancelled, None, None, None),
                        Outcome::Stopped(StopReason::Pause) => (Status::Waiting, None, None, None),
                        Outcome::Failed { error } => (Status::Failed, Some(error), None, Some(finished.log)),
                    },
                    Err(error) => (Status::Failed, Some(error.to_string()), None, None),
                };
                queue.set_status(id, status);
                if let Some(item) = queue.get_mut(id) {
                    item.error = error;
                    if status == Status::Completed {
                        item.output = output;
                        item.progress = Some(Progress { percent: 100.0, ..Default::default() });
                        completed_title = Some(item.title.clone().unwrap_or_else(|| item.url.clone()));
                    }
                }
                if let Some(log) = log {
                    queue.logs.insert(id.to_string(), log);
                }
            }
        }
        self.queue_changed();
        if let Some(title) = completed_title {
            self.notify_completed(&title);
        }
        self.schedule();
    }

    fn notify_completed(&self, title: &str) {
        if !self.settings().notify_on_finish {
            return;
        }
        let focused = self
            .handle
            .get_webview_window("main")
            .and_then(|w| w.is_focused().ok())
            .unwrap_or(false);
        if focused {
            return;
        }
        let _ = self.handle.notification().builder().title("Download complete").body(title).show();
    }

    // ----- components -----

    /// Find or install yt-dlp, FFmpeg and Deno, then start the queue.
    pub async fn setup_components(self: Arc<Self>) {
        let _guard = self.setup_lock.lock().await;
        let layout = self.paths.layout.clone();
        components::clean_leftovers(&layout);

        // yt-dlp first: titles can be fetched while the larger downloads finish
        self.ensure(Kind::YtDlp).await;
        self.fetch_pending_metadata();
        tokio::join!(self.ensure(Kind::Ffmpeg), self.ensure(Kind::Deno));

        if self.components_ready() {
            self.schedule();
            if self.settings().auto_update_engine && self.update_check_due() {
                let _ = self.update_engine().await;
            }
        }
    }

    async fn ensure(&self, kind: Kind) {
        if self.components.lock().unwrap().status_mut(kind).state == State::Ready {
            return;
        }
        let layout = &self.paths.layout;
        if let Some((path, source)) = layout.existing(kind) {
            let (binary, arg) = components::version_binary(kind, &path);
            if let Ok(line) = components::probe_version(&binary, arg).await {
                self.mark_ready(kind, path, source, components::short_version(kind, &line));
                return;
            }
            log::warn!("{} at {} does not run; reinstalling", kind.label(), path.display());
        }
        self.update_component(kind, |s| {
            s.state = State::Downloading;
            s.error = None;
            s.downloaded = 0;
            s.total = None;
        });
        let mut last_emit = Instant::now() - PROGRESS_INTERVAL;
        let result = components::install(&self.client, layout, kind, |done, total| {
            let mut components = self.components.lock().unwrap();
            let status = components.status_mut(kind);
            status.downloaded = done;
            status.total = total;
            if total.is_some_and(|t| done >= t) {
                status.state = State::Installing;
            }
            drop(components);
            if last_emit.elapsed() >= PROGRESS_INTERVAL {
                last_emit = Instant::now();
                self.components_changed();
            }
        })
        .await;
        match result {
            Ok(path) => {
                let (binary, arg) = components::version_binary(kind, &path);
                let version = components::probe_version(&binary, arg).await.unwrap_or_default();
                self.mark_ready(kind, path, Source::Managed, components::short_version(kind, &version));
            }
            Err(error) => self.update_component(kind, |s| {
                s.state = State::Failed;
                s.error = Some(error.to_string());
            }),
        }
    }

    fn mark_ready(&self, kind: Kind, path: PathBuf, source: Source, version: String) {
        {
            let mut components = self.components.lock().unwrap();
            match kind {
                Kind::YtDlp => components.tools.ytdlp = Some(path),
                Kind::Ffmpeg => components.tools.ffmpeg_dir = Some(path),
                Kind::Deno => components.tools.deno = Some(path),
            }
            let status = components.status_mut(kind);
            status.state = State::Ready;
            status.source = Some(source);
            status.version = Some(version);
            status.error = None;
        }
        self.components_changed();
    }

    fn update_check_due(&self) -> bool {
        std::fs::metadata(&self.paths.update_check_file)
            .and_then(|m| m.modified())
            .ok()
            .and_then(|t| t.elapsed().ok())
            .is_none_or(|age| age >= UPDATE_CHECK_INTERVAL)
    }

    /// Replace yt-dlp with the latest release when it differs. Returns the new version, if updated.
    pub async fn update_engine(self: &Arc<Self>) -> Result<Option<String>> {
        let layout = self.paths.layout.clone();
        if !layout.ytdlp().is_file() {
            return Err(Error::msg("yt-dlp is not installed yet"));
        }
        let available = components::ytdlp_update_available(&self.client, &layout).await?;
        let _ = std::fs::write(&self.paths.update_check_file, queue::now().to_string());
        if !available {
            return Ok(None);
        }
        if !self.running.lock().unwrap().is_empty() {
            return Err(Error::msg("Finish or pause the running downloads, then check again"));
        }
        // Holding Installing keeps the scheduler from starting downloads during the swap
        self.update_component(Kind::YtDlp, |s| {
            s.state = State::Downloading;
            s.downloaded = 0;
        });
        let installed = components::install(&self.client, &layout, Kind::YtDlp, |_, _| {}).await;
        let result = match installed {
            Ok(path) => {
                let line = components::probe_version(&path, "--version").await.unwrap_or_default();
                let version = components::short_version(Kind::YtDlp, &line);
                self.mark_ready(Kind::YtDlp, path, Source::Managed, version.clone());
                Ok(Some(version))
            }
            Err(error) => {
                // The old binary is still in place when installing fails
                self.update_component(Kind::YtDlp, |s| s.state = State::Ready);
                Err(error)
            }
        };
        self.schedule();
        result
    }

    pub fn retry_setup(self: &Arc<Self>) {
        let app = self.clone();
        tauri::async_runtime::spawn(async move { app.setup_components().await });
    }

    pub fn client(&self) -> &reqwest::Client {
        &self.client
    }
}
