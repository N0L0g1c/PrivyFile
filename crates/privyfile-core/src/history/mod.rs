use std::fs;
use std::path::PathBuf;

use chrono::Utc;

use crate::types::{HistoryEntry, PrivyFileError, Result};

pub fn history_file_path() -> PathBuf {
    dirs_path().join("history.json")
}

pub fn settings_file_path() -> PathBuf {
    dirs_path().join("settings.json")
}

fn dirs_path() -> PathBuf {
    if let Ok(custom) = std::env::var("PRIVYFILE_DATA_DIR") {
        return PathBuf::from(custom);
    }

    let base = std::env::var("APPDATA")
        .or_else(|_| std::env::var("XDG_DATA_HOME"))
        .unwrap_or_else(|_| std::env::temp_dir().to_string_lossy().into_owned());

    PathBuf::from(base).join("PrivyFile")
}

pub fn append_history(entry: HistoryEntry) -> Result<()> {
    let path = history_file_path();
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    let mut entries = load_history().unwrap_or_default();
    entries.push(entry);
    if entries.len() > 500 {
        let drain = entries.len() - 500;
        entries.drain(0..drain);
    }

    let json = serde_json::to_string_pretty(&entries)
        .map_err(|error| PrivyFileError::Metadata(error.to_string()))?;
    fs::write(path, json)?;
    Ok(())
}

pub fn load_history() -> Result<Vec<HistoryEntry>> {
    let path = history_file_path();
    if !path.exists() {
        return Ok(Vec::new());
    }
    let json = fs::read_to_string(path)?;
    serde_json::from_str(&json).map_err(|error| PrivyFileError::Metadata(error.to_string()))
}

pub fn clear_history() -> Result<()> {
    let path = history_file_path();
    if path.exists() {
        fs::remove_file(path)?;
    }
    Ok(())
}

pub fn history_entry_from_clean(source: &str, output: Option<&str>, before: u8, after: u8) -> HistoryEntry {
    HistoryEntry {
        timestamp: Utc::now().to_rfc3339(),
        source_path: source.to_string(),
        output_path: output.map(str::to_string),
        action: "clean".into(),
        privacy_score_before: before,
        privacy_score_after: after,
    }
}
