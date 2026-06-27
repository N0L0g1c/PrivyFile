mod commands;

use std::sync::Mutex;

use privyfile_core::{
    bootstrap_exiftool_from_exe_dir, exiftool_available, try_configure_exiftool_dir,
};
use privyfile_core::watch::WatchHandle;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_opener::init())
        .manage(Mutex::new(None::<WatchHandle>))
        .setup(|app| {
            init_exiftool(app);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_metadata,
            commands::clean_file,
            commands::shred_file,
            commands::shred_folder,
            commands::process_batch,
            commands::load_settings,
            commands::save_settings,
            commands::load_history,
            commands::clear_history,
            commands::exiftool_available,
            commands::exiftool_version,
            commands::exiftool_status,
            commands::shred_warning,
            commands::start_watch_folder,
            commands::stop_watch_folder,
            commands::list_profiles,
            commands::open_report,
        ])
        .run(tauri::generate_context!())
        .expect("error while running PrivyFile");
}

fn init_exiftool(app: &tauri::App) {
    let platform = exiftool_platform_dir();

    if let Ok(dir) = app.path().resolve(
        format!("binaries/{platform}"),
        tauri::path::BaseDirectory::Resource,
    ) {
        if try_configure_exiftool_dir(&dir) && exiftool_available() {
            return;
        }
    }

    if let Ok(resource_dir) = app.path().resource_dir() {
        let dir = resource_dir.join("binaries").join(platform);
        if try_configure_exiftool_dir(&dir) && exiftool_available() {
            return;
        }
    }

    bootstrap_exiftool_from_exe_dir();
}

fn exiftool_platform_dir() -> &'static str {
    if cfg!(windows) {
        "win"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "linux"
    }
}
