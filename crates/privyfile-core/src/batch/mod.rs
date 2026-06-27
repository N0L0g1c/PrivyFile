use std::path::Path;

use crate::metadata::{clean_file, get_metadata};
use crate::profiles::apply_profile;
use crate::reports::write_batch_report;
use crate::shred::shred_file;
use crate::types::{
    BatchAction, BatchItem, BatchItemResult, BatchProgress, BatchResult, CleanOptions, Result,
    ShredOptions,
};

pub type ProgressCallback = Box<dyn Fn(BatchProgress) + Send + Sync>;

pub fn process_batch(
    items: &[BatchItem],
    clean_options: &CleanOptions,
    shred_options: &ShredOptions,
    progress: Option<&ProgressCallback>,
) -> Result<BatchResult> {
    let total = items.len();
    let mut results = Vec::new();

    for (index, item) in items.iter().enumerate() {
        let path = Path::new(&item.path);
        emit_progress(progress, index, total, &item.path, "Processing");

        let result = match item.action {
            BatchAction::MetadataOnly => BatchItemResult {
                path: item.path.clone(),
                success: true,
                message: "Metadata read".into(),
                clean_result: None,
                shred_result: None,
                metadata: Some(get_metadata(path)?),
            },
            BatchAction::Clean => {
                let options = apply_profile(clean_options.clone());
                let clean = clean_file(path, &options)?;
                BatchItemResult {
                    path: item.path.clone(),
                    success: true,
                    message: "Cleaned".into(),
                    clean_result: Some(clean),
                    shred_result: None,
                    metadata: None,
                }
            }
            BatchAction::Shred => {
                let shred = shred_file(path, shred_options)?;
                BatchItemResult {
                    path: item.path.clone(),
                    success: true,
                    message: "Shredded".into(),
                    clean_result: None,
                    shred_result: Some(shred),
                    metadata: None,
                }
            }
            BatchAction::CleanAndShred => {
                let mut options = apply_profile(clean_options.clone());
                options.preserve_original = false;
                options.shred_original = true;
                let clean = clean_file(path, &options)?;
                BatchItemResult {
                    path: item.path.clone(),
                    success: true,
                    message: "Cleaned and shredded original".into(),
                    clean_result: Some(clean),
                    shred_result: None,
                    metadata: None,
                }
            }
        };

        results.push(result);
        emit_progress(progress, index + 1, total, &item.path, "Done");
    }

    let report_path = write_batch_report(&results)?;

    Ok(BatchResult {
        items: results,
        report_path: Some(report_path),
    })
}

fn emit_progress(
    callback: Option<&ProgressCallback>,
    current: usize,
    total: usize,
    file_path: &str,
    status: &str,
) {
    if let Some(callback) = callback {
        let percent = if total == 0 {
            100
        } else {
            ((current as f32 / total as f32) * 100.0) as u8
        };
        callback(BatchProgress {
            current,
            total,
            file_path: file_path.to_string(),
            status: status.to_string(),
            percent,
        });
    }
}

pub fn collect_files_from_paths(paths: &[String], recursive: bool) -> Result<Vec<String>> {
    use walkdir::WalkDir;

    let mut files = Vec::new();
    for path in paths {
        let path = Path::new(path);
        if path.is_file() {
            files.push(path.to_string_lossy().into_owned());
        } else if path.is_dir() {
            let walker = if recursive {
                WalkDir::new(path)
            } else {
                WalkDir::new(path).max_depth(1)
            };
            for entry in walker.into_iter().filter_map(|entry| entry.ok()) {
                if entry.file_type().is_file() {
                    files.push(entry.path().to_string_lossy().into_owned());
                }
            }
        }
    }
    Ok(files)
}
