//! StreamQ 2: a Tauri shell around the yt-dlp command-line engine.

mod app;
mod components;
mod engine;
mod error;
mod queue;
mod settings;

#[cfg(test)]
mod integration_tests;

use std::sync::Arc;

use serde::Serialize;
use tauri::{Manager, State};
use tauri_plugin_opener::OpenerExt;

use app::{AddResult, App, Paths, Snapshot};
use engine::Format;
use error::{Error, Result};
use settings::Settings;

type AppState<'a> = State<'a, Arc<App>>;

const APP_RELEASES: &str = "https://github.com/ivanerror/streamq/releases/latest";

#[tauri::command]
fn get_snapshot(app: AppState) -> Snapshot {
    app.snapshot()
}

#[tauri::command]
fn add_urls(app: AppState, urls: Vec<String>, format: Option<Format>, quality: Option<u32>) -> AddResult {
    app.add_urls(urls, format, quality)
}

#[tauri::command]
fn set_item_options(app: AppState, id: String, format: Format, quality: u32) -> Result<()> {
    app.set_item_options(&id, format, quality)
}

#[tauri::command]
fn retry_items(app: AppState, ids: Option<Vec<String>>) -> usize {
    app.retry(ids)
}

#[tauri::command]
fn cancel_item(app: AppState, id: String) {
    app.cancel(&id)
}

#[tauri::command]
fn cancel_all(app: AppState) {
    app.cancel_all()
}

#[tauri::command]
fn set_paused(app: AppState, paused: bool) {
    app.set_paused(paused)
}

#[tauri::command]
fn remove_item(app: AppState, id: String) {
    app.remove(&id)
}

#[tauri::command]
fn clear_completed(app: AppState) -> usize {
    app.clear_completed()
}

#[tauri::command]
async fn expand_playlist(app: AppState<'_>, id: String) -> Result<usize> {
    app.inner().clone().expand_playlist(&id).await
}

#[tauri::command]
fn get_log(app: AppState, id: String) -> Vec<String> {
    app.log(&id)
}

#[tauri::command]
fn save_settings(app: AppState, settings: Settings) -> Result<Settings> {
    app.save_settings(settings)
}

#[tauri::command]
async fn update_engine(app: AppState<'_>) -> Result<Option<String>> {
    app.inner().clone().update_engine().await
}

#[tauri::command]
fn retry_setup(app: AppState) {
    app.retry_setup()
}

#[tauri::command]
fn reveal_output(handle: tauri::AppHandle, app: AppState, id: String) -> Result<()> {
    let path = app.output_path(&id).ok_or_else(|| Error::msg("This download has no saved file"))?;
    handle.opener().reveal_item_in_dir(&path).map_err(|e| Error::msg(e.to_string()))
}

#[tauri::command]
fn open_output_dir(handle: tauri::AppHandle, app: AppState) -> Result<()> {
    let dir = app.settings().output_dir;
    std::fs::create_dir_all(&dir)?;
    handle.opener().open_path(&dir, None::<&str>).map_err(|e| Error::msg(e.to_string()))
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct AppUpdate {
    version: String,
    url: String,
}

/// Compare dotted versions numerically ("2.10.0" > "2.9.1").
fn is_newer(candidate: &str, current: &str) -> bool {
    let parse = |v: &str| -> Vec<u64> {
        v.trim_start_matches('v')
            .split(['.', '-'])
            .map_while(|part| part.parse().ok())
            .collect()
    };
    parse(candidate) > parse(current)
}

/// Each release publishes a small version.json next to the installers.
#[tauri::command]
async fn check_app_update(app: AppState<'_>) -> Result<Option<AppUpdate>> {
    let response = app.client().get(format!("{APP_RELEASES}/download/version.json")).send().await?;
    // Releases made before 2.0 have no version.json
    if response.status() == reqwest::StatusCode::NOT_FOUND {
        return Ok(None);
    }
    let json: serde_json::Value = serde_json::from_str(&response.error_for_status()?.text().await?)?;
    let version = json.get("version").and_then(|v| v.as_str()).unwrap_or_default().to_string();
    let url = json.get("url").and_then(|v| v.as_str()).unwrap_or(APP_RELEASES).to_string();
    Ok(is_newer(&version, env!("CARGO_PKG_VERSION")).then_some(AppUpdate { version, url }))
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_single_instance::init(|handle, _args, _cwd| {
            if let Some(window) = handle.get_webview_window("main") {
                let _ = window.unminimize();
                let _ = window.set_focus();
            }
        }))
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_clipboard_manager::init())
        .setup(|tauri_app| {
            let path = tauri_app.path();
            let data = path.app_data_dir()?;
            let local = path.app_local_data_dir()?;
            let default_output = path
                .download_dir()
                .or_else(|_| path.home_dir())?
                .join("StreamQ")
                .to_string_lossy()
                .into_owned();
            let paths = Paths {
                queue_file: data.join("queue.json"),
                settings_file: data.join("settings.json"),
                update_check_file: local.join("last-engine-check"),
                default_output,
                layout: components::Layout { root: local.join("components") },
            };
            let app = App::new(tauri_app.handle().clone(), paths).map_err(|e| e.to_string())?;
            tauri_app.manage(app.clone());
            tauri::async_runtime::spawn(app.setup_components());
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            add_urls,
            set_item_options,
            retry_items,
            cancel_item,
            cancel_all,
            set_paused,
            remove_item,
            clear_completed,
            expand_playlist,
            get_log,
            save_settings,
            update_engine,
            retry_setup,
            reveal_output,
            open_output_dir,
            check_app_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running StreamQ");
}

#[cfg(test)]
mod tests {
    use super::is_newer;

    #[test]
    fn compares_versions_numerically() {
        assert!(is_newer("2.10.0", "2.9.1"));
        assert!(is_newer("v2.0.1", "2.0.0"));
        assert!(!is_newer("2.0.0", "2.0.0"));
        assert!(!is_newer("", "2.0.0"));
    }
}
