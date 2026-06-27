use std::path::Path;
use std::sync::{Arc, Mutex};

use privyfile_core::{
    append_history, batch, clean_file as core_clean_file, clear_history as core_clear_history,
    detect_file_type, exiftool_available as core_exiftool_available,
    exiftool_bundle_dir, exiftool_version as core_exiftool_version,
    get_metadata as core_get_metadata, history_entry_from_clean,
    load_history as core_load_history, profile_description, profile_label,
    profiles::apply_profile, shred_file as core_shred_file, shred_folder as core_shred_folder,
    shred_warning_message, start_watch_folder as core_start_watch_folder, AppSettings, BatchItem,
    BatchProgress, BatchResult, CleanOptions, CleanProfileId, CleanResult, HistoryEntry,
    MetadataReport, ShredOptions, ShredResult,
};
use privyfile_core::watch::WatchHandle;
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};
use tauri_plugin_opener::OpenerExt;

#[derive(Debug, Serialize)]
pub struct ProfileInfo {
    pub id: CleanProfileId,
    pub label: String,
    pub description: String,
}

#[tauri::command]
pub fn get_metadata(path: String) -> Result<MetadataReport, String> {
    core_get_metadata(Path::new(&path)).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn clean_file(path: String, options: CleanOptions) -> Result<CleanResult, String> {
    let applied = apply_profile(options.clone());
    let result = core_clean_file(Path::new(&path), &applied).map_err(|error| error.to_string())?;

    if options.preserve_original {
        let _ = append_history(history_entry_from_clean(
            &result.source_path,
            result.output_path.as_deref(),
            result.privacy_score_before,
            result.privacy_score_after,
        ));
    }

    Ok(result)
}

#[tauri::command]
pub fn shred_file(path: String, options: ShredOptions) -> Result<ShredResult, String> {
    core_shred_file(Path::new(&path), &options).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn shred_folder(
    path: String,
    recursive: bool,
    options: ShredOptions,
) -> Result<Vec<ShredResult>, String> {
    core_shred_folder(Path::new(&path), recursive, &options).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn process_batch(
    app: AppHandle,
    items: Vec<BatchItem>,
    clean_options: CleanOptions,
    shred_options: ShredOptions,
) -> Result<BatchResult, String> {
    let app_handle = app.clone();
    let progress = Box::new(move |progress: BatchProgress| {
        let _ = app_handle.emit("batch-progress", progress);
    }) as batch::ProgressCallback;

    batch::process_batch(&items, &clean_options, &shred_options, Some(&progress))
        .map_err(|error| error.to_string())
}

#[tauri::command]
pub fn load_settings() -> Result<AppSettings, String> {
    let path = privyfile_core::history::settings_file_path();
    if !path.exists() {
        return Ok(AppSettings::default());
    }
    let json = std::fs::read_to_string(path).map_err(|error| error.to_string())?;
    serde_json::from_str(&json).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn save_settings(settings: AppSettings) -> Result<(), String> {
    let path = privyfile_core::history::settings_file_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    let json = serde_json::to_string_pretty(&settings).map_err(|error| error.to_string())?;
    std::fs::write(path, json).map_err(|error| error.to_string())
}

#[tauri::command]
pub fn load_history() -> Result<Vec<HistoryEntry>, String> {
    core_load_history().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn clear_history() -> Result<(), String> {
    core_clear_history().map_err(|error| error.to_string())
}

#[tauri::command]
pub fn exiftool_available() -> bool {
    core_exiftool_available()
}

#[derive(Debug, Serialize)]
pub struct ExifToolStatus {
    pub available: bool,
    pub version: Option<String>,
    pub path: Option<String>,
    pub bundle_dir: Option<String>,
}

#[tauri::command]
pub fn exiftool_status() -> ExifToolStatus {
    ExifToolStatus {
        available: core_exiftool_available(),
        version: core_exiftool_version(),
        path: privyfile_core::resolve_exiftool_path().map(|path| path.to_string_lossy().into_owned()),
        bundle_dir: exiftool_bundle_dir().map(|path| path.to_string_lossy().into_owned()),
    }
}

#[tauri::command]
pub fn exiftool_version() -> Option<String> {
    core_exiftool_version()
}

#[tauri::command]
pub fn shred_warning() -> String {
    shred_warning_message().to_string()
}

#[tauri::command]
pub fn start_watch_folder(
    app: AppHandle,
    folder: String,
    options: CleanOptions,
    watch_state: State<'_, Mutex<Option<WatchHandle>>>,
) -> Result<(), String> {
    let callback = Arc::new(move |path: String| {
        let _ = app.emit("watch-file-cleaned", path);
    });
    let handle = core_start_watch_folder(Path::new(&folder), options, callback)
        .map_err(|error| error.to_string())?;
    *watch_state.lock().map_err(|error| error.to_string())? = Some(handle);
    Ok(())
}

#[tauri::command]
pub fn stop_watch_folder(watch_state: State<'_, Mutex<Option<WatchHandle>>>) -> Result<(), String> {
    *watch_state.lock().map_err(|error| error.to_string())? = None;
    Ok(())
}

#[tauri::command]
pub fn list_profiles() -> Vec<ProfileInfo> {
    [
        CleanProfileId::SocialMediaShare,
        CleanProfileId::LegalDocument,
        CleanProfileId::PhotoBackup,
        CleanProfileId::RemoveAll,
    ]
    .into_iter()
    .map(|id| ProfileInfo {
        id,
        label: profile_label(id).to_string(),
        description: profile_description(id).to_string(),
    })
    .collect()
}

#[tauri::command]
pub fn open_report(app: AppHandle, path: String) -> Result<(), String> {
    app.opener()
        .open_path(path, None::<&str>)
        .map_err(|error| error.to_string())
}

#[allow(dead_code)]
fn supported_extension(path: &str) -> bool {
    detect_file_type(Path::new(path)).is_some()
}
