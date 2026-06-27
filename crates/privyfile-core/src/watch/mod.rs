use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use notify_debouncer_full::{new_debouncer, DebounceEventResult, Debouncer, RecommendedCache};
use notify::{RecommendedWatcher, RecursiveMode};

use crate::metadata::clean_file;
use crate::profiles::apply_profile;
use crate::types::{CleanOptions, PrivyFileError, Result};

pub struct WatchHandle {
    _debouncer: Debouncer<RecommendedWatcher, RecommendedCache>,
}

pub fn start_watch_folder(
    folder: &Path,
    options: CleanOptions,
    on_complete: Arc<dyn Fn(String) + Send + Sync>,
) -> Result<WatchHandle> {
    if !folder.exists() {
        return Err(PrivyFileError::NotFound(folder.to_string_lossy().into_owned()));
    }

    let folder = folder.to_path_buf();
    let callback = on_complete;
    let clean_options = Arc::new(Mutex::new(options));

    let mut debouncer = new_debouncer(
        Duration::from_secs(2),
        None,
        move |result: DebounceEventResult| {
            if let Ok(events) = result {
                for event in events {
                    for path in &event.paths {
                        if path.is_file() {
                            if let Ok(options) = clean_options.lock() {
                                let applied = apply_profile(options.clone());
                                if clean_file(path, &applied).is_ok() {
                                    callback(path.to_string_lossy().into_owned());
                                }
                            }
                        }
                    }
                }
            }
        },
    )
    .map_err(|error| PrivyFileError::Metadata(error.to_string()))?;

    debouncer
        .watch(&folder, RecursiveMode::NonRecursive)
        .map_err(|error| PrivyFileError::Metadata(error.to_string()))?;

    Ok(WatchHandle {
        _debouncer: debouncer,
    })
}
