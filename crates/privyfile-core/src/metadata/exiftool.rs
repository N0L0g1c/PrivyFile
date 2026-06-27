use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::OnceLock;

use crate::metadata::categories::{categorize_tag, compute_privacy_score};
use crate::types::{MetadataReport, PrivyFileError, Result, TagEntry};

static EXIFTOOL_DIR: OnceLock<PathBuf> = OnceLock::new();

/// Called by the Tauri shell at startup with the resolved resource directory.
pub fn configure_exiftool_dir(dir: PathBuf) {
    let _ = try_configure_exiftool_dir(&dir);
}

/// Returns true when the bundled directory contains a runnable ExifTool install.
pub fn try_configure_exiftool_dir(dir: &Path) -> bool {
    if !is_valid_exiftool_bundle(dir) {
        return false;
    }

    if EXIFTOOL_DIR.get().is_some() {
        return true;
    }

    EXIFTOOL_DIR.set(dir.to_path_buf()).is_ok()
}

/// Discover ExifTool next to the running executable (installed app layout).
pub fn bootstrap_exiftool_from_exe_dir() {
    if EXIFTOOL_DIR.get().is_some() {
        return;
    }

    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let Some(parent) = exe.parent() else {
        return;
    };

    for dir in install_candidate_dirs(parent) {
        if try_configure_exiftool_dir(&dir) {
            return;
        }
    }
}

fn install_candidate_dirs(exe_dir: &Path) -> Vec<PathBuf> {
    let platform = exiftool_platform_dir();
    vec![
        // Tauri Windows install: <InstallDir>/binaries/win/
        exe_dir.join("binaries").join(platform),
        // Some layouts nest under resources/ (debug or older bundles)
        exe_dir
            .join("resources")
            .join("binaries")
            .join(platform),
        exe_dir.join("binaries"),
        // Dev: target/debug -> project src-tauri/binaries/win
        exe_dir
            .join("..")
            .join("..")
            .join("src-tauri")
            .join("binaries")
            .join(platform),
        exe_dir
            .join("..")
            .join("src-tauri")
            .join("binaries")
            .join(platform),
    ]
}

fn is_valid_exiftool_bundle(dir: &Path) -> bool {
    dir.join(exiftool_binary_name()).is_file() && has_exiftool_support_files(dir)
}

fn has_exiftool_support_files(dir: &Path) -> bool {
    if dir.join("exiftool_files").is_dir() {
        return true;
    }

    cfg!(not(windows)) && dir.join("lib").is_dir()
}

pub fn resolve_exiftool_path() -> Option<PathBuf> {
    bootstrap_exiftool_from_exe_dir();

    exiftool_candidates()
        .into_iter()
        .find(|path| path.is_file())
}

pub fn exiftool_bundle_dir() -> Option<PathBuf> {
    bootstrap_exiftool_from_exe_dir();
    EXIFTOOL_DIR.get().cloned().or_else(|| {
        resolve_exiftool_path()
            .and_then(|path| path.parent().map(Path::to_path_buf))
    })
}

fn exiftool_candidates() -> Vec<PathBuf> {
    let mut paths = Vec::new();

    if let Some(dir) = EXIFTOOL_DIR.get() {
        paths.push(dir.join(exiftool_binary_name()));
    }

    if let Ok(resource) = std::env::var("PRIVYFILE_EXIFTOOL") {
        paths.push(PathBuf::from(resource));
    }

    if let Ok(resource_dir) = std::env::var("PRIVYFILE_EXIFTOOL_DIR") {
        paths.push(PathBuf::from(resource_dir).join(exiftool_binary_name()));
    }

    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            for dir in install_candidate_dirs(parent) {
                paths.push(dir.join(exiftool_binary_name()));
            }
        }
    }

    // Development tree when launched from project root
    paths.push(
        PathBuf::from("src-tauri")
            .join("binaries")
            .join(exiftool_platform_dir())
            .join(exiftool_binary_name()),
    );

    paths
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

fn exiftool_binary_name() -> &'static str {
    if cfg!(windows) {
        "exiftool.exe"
    } else {
        "exiftool"
    }
}

fn run_exiftool(binary: &Path) -> Command {
    let mut command = Command::new(binary);
    if let Some(dir) = exiftool_support_dir(binary) {
        // Windows build requires exiftool_files/ beside exiftool.exe
        command.current_dir(dir);
    }
    command
}

fn exiftool_support_dir(binary: &Path) -> Option<PathBuf> {
    binary.parent().map(Path::to_path_buf)
}

fn verify_exiftool_binary(binary: &Path) -> bool {
    if !binary.is_file() {
        return false;
    }

    let support_dir = match exiftool_support_dir(binary) {
        Some(dir) => dir,
        None => return false,
    };

    if cfg!(windows) && !support_dir.join("exiftool_files").is_dir() {
        return false;
    }

    if cfg!(not(windows))
        && !support_dir.join("lib").is_dir()
        && !support_dir.join("exiftool_files").is_dir()
    {
        return false;
    }

    run_exiftool(binary)
        .arg("-ver")
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

pub fn exiftool_available() -> bool {
    if let Some(path) = resolve_exiftool_path() {
        return verify_exiftool_binary(&path);
    }

    system_exiftool_path()
        .map(|path| verify_exiftool_binary(&path))
        .unwrap_or(false)
}

pub fn exiftool_version() -> Option<String> {
    let binary = resolve_exiftool_path().or_else(system_exiftool_path)?;
    let output = run_exiftool(&binary).arg("-ver").output().ok()?;
    if !output.status.success() {
        return None;
    }
    Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn read_metadata_with_exiftool(path: &Path) -> Result<Vec<TagEntry>> {
    bootstrap_exiftool_from_exe_dir();

    let Some(binary) = resolve_exiftool_path().or_else(system_exiftool_path) else {
        return Ok(Vec::new());
    };

    let output = run_exiftool(&binary)
        .args(["-json", "-a", "-G1", path.to_string_lossy().as_ref()])
        .output()
        .map_err(|error| PrivyFileError::Metadata(format!("Failed to run ExifTool: {error}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(PrivyFileError::Metadata(format!(
            "ExifTool failed: {stderr}"
        )));
    }

    parse_exiftool_json(&output.stdout)
}

pub fn clean_with_exiftool(
    path: &Path,
    output_path: &Path,
    remove_all: bool,
    tag_patterns: &[String],
) -> Result<()> {
    bootstrap_exiftool_from_exe_dir();

    let Some(binary) = resolve_exiftool_path().or_else(system_exiftool_path) else {
        return Err(PrivyFileError::Metadata(
            "ExifTool is not available. Reinstall PrivyFile or run npm run fetch-exiftool for development builds.".into(),
        ));
    };

    if !verify_exiftool_binary(&binary) {
        return Err(PrivyFileError::Metadata(
            "Bundled ExifTool failed to start. Ensure exiftool_files/ is installed beside exiftool.exe.".into(),
        ));
    }

    std::fs::copy(path, output_path)?;

    let mut command = run_exiftool(&binary);
    command.arg("-overwrite_original");

    if remove_all {
        command.arg("-all=");
    } else {
        for pattern in tag_patterns {
            command.arg(format!("-{pattern}="));
        }
    }

    command.arg(output_path);

    let output = command
        .output()
        .map_err(|error| PrivyFileError::Metadata(format!("Failed to run ExifTool: {error}")))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(PrivyFileError::Metadata(format!(
            "ExifTool clean failed: {stderr}"
        )));
    }

    Ok(())
}

fn system_exiftool_path() -> Option<PathBuf> {
    let name = if cfg!(windows) {
        "exiftool.exe"
    } else {
        "exiftool"
    };
    if verify_exiftool_binary(Path::new(name)) {
        Some(PathBuf::from(name))
    } else {
        None
    }
}

fn parse_exiftool_json(bytes: &[u8]) -> Result<Vec<TagEntry>> {
    let value: serde_json::Value = serde_json::from_slice(bytes)
        .map_err(|error| PrivyFileError::Metadata(format!("Invalid ExifTool JSON: {error}")))?;

    let Some(array) = value.as_array() else {
        return Ok(Vec::new());
    };

    let Some(first) = array.first() else {
        return Ok(Vec::new());
    };

    let Some(object) = first.as_object() else {
        return Ok(Vec::new());
    };

    let mut tags = Vec::new();
    for (name, value) in object {
        if name == "SourceFile" {
            continue;
        }
        let text = json_value_to_string(value);
        if text.is_empty() {
            continue;
        }
        tags.push(TagEntry {
            name: name.clone(),
            value: text,
            category: categorize_tag(name),
        });
    }

    Ok(tags)
}

fn json_value_to_string(value: &serde_json::Value) -> String {
    match value {
        serde_json::Value::String(text) => text.clone(),
        serde_json::Value::Number(number) => number.to_string(),
        serde_json::Value::Bool(boolean) => boolean.to_string(),
        serde_json::Value::Array(items) => items
            .iter()
            .map(json_value_to_string)
            .collect::<Vec<_>>()
            .join(", "),
        serde_json::Value::Null => String::new(),
        serde_json::Value::Object(map) => serde_json::to_string(map).unwrap_or_default(),
    }
}

pub fn metadata_report_from_tags(path: &Path, file_type: &str, tags: Vec<TagEntry>) -> MetadataReport {
    let privacy_score = compute_privacy_score(&tags);
    MetadataReport {
        file_path: path.to_string_lossy().into_owned(),
        file_type: file_type.to_string(),
        tags,
        privacy_score,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn write_fake_bundle(dir: &Path) {
        if cfg!(windows) {
            fs::write(dir.join("exiftool.exe"), b"").unwrap();
            fs::create_dir_all(dir.join("exiftool_files")).unwrap();
        } else {
            fs::write(dir.join("exiftool"), b"#!/usr/bin/perl\n").unwrap();
            fs::create_dir_all(dir.join("lib")).unwrap();
        }
    }

    #[test]
    fn validates_bundle_requires_support_files() {
        let temp = TempDir::new().unwrap();
        if cfg!(windows) {
            fs::write(temp.path().join("exiftool.exe"), b"").unwrap();
        } else {
            fs::write(temp.path().join("exiftool"), b"").unwrap();
        }
        assert!(!is_valid_exiftool_bundle(temp.path()));
        write_fake_bundle(temp.path());
        assert!(is_valid_exiftool_bundle(temp.path()));
    }

    #[test]
    fn configure_accepts_valid_bundle_layout() {
        let temp = TempDir::new().unwrap();
        write_fake_bundle(temp.path());
        assert!(try_configure_exiftool_dir(temp.path()));
    }

    #[test]
    fn install_candidates_include_platform_folder() {
        let dirs = install_candidate_dirs(Path::new("C:\\Program Files\\PrivyFile"));
        assert!(dirs
            .iter()
            .any(|dir| dir.ends_with("binaries\\win") || dir.ends_with("binaries/win")));
    }
}
