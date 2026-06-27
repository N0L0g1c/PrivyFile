use std::fs::OpenOptions;
use std::io::{Seek, SeekFrom, Write};
use std::path::{Path, PathBuf};

use rand::RngCore;
use walkdir::WalkDir;

use crate::types::{PrivyFileError, Result, ShredMethod, ShredOptions, ShredResult};

pub fn shred_file(path: &Path, options: &ShredOptions) -> Result<ShredResult> {
    if !path.exists() {
        return Err(PrivyFileError::NotFound(path.to_string_lossy().into_owned()));
    }

    if path.is_dir() {
        return Err(PrivyFileError::Shred(
            "Path is a directory; use shred_folder".into(),
        ));
    }

    let passes = effective_passes(options);
    let file_len = std::fs::metadata(path)?.len();
    let mut bytes_overwritten = 0u64;

    for pass in 0..passes {
        let pattern = pass_pattern(options.method, pass);
        overwrite_file(path, file_len, &pattern)?;
        bytes_overwritten += file_len;
    }

    std::fs::remove_file(path)?;

    Ok(ShredResult {
        file_path: path.to_string_lossy().into_owned(),
        passes_completed: passes,
        bytes_overwritten,
        deleted: true,
    })
}

pub fn shred_folder(path: &Path, recursive: bool, options: &ShredOptions) -> Result<Vec<ShredResult>> {
    if !path.exists() {
        return Err(PrivyFileError::NotFound(path.to_string_lossy().into_owned()));
    }

    let mut results = Vec::new();
    let walker = if recursive {
        WalkDir::new(path).into_iter()
    } else {
        WalkDir::new(path).max_depth(1).into_iter()
    };

    let mut files = Vec::new();
    for entry in walker.filter_map(|entry| entry.ok()) {
        if entry.file_type().is_file() {
            files.push(entry.path().to_path_buf());
        }
    }

    for file in files {
        results.push(shred_file(&file, options)?);
    }

    if recursive {
        remove_empty_dirs(path)?;
    }

    Ok(results)
}

fn remove_empty_dirs(path: &Path) -> Result<()> {
    if path.is_dir() {
        for entry in std::fs::read_dir(path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                remove_empty_dirs(&entry.path())?;
            }
        }
        if std::fs::read_dir(path)?.next().is_none() {
            let _ = std::fs::remove_dir(path);
        }
    }
    Ok(())
}

fn effective_passes(options: &ShredOptions) -> u8 {
    match options.method {
        ShredMethod::Random1Pass => 1,
        ShredMethod::Dod5220 => 3,
        ShredMethod::SevenPass => 7,
        ShredMethod::Custom => options.passes.max(1),
    }
}

fn pass_pattern(method: ShredMethod, pass_index: u8) -> Vec<u8> {
    match method {
        ShredMethod::Random1Pass => random_pattern(4096),
        ShredMethod::Dod5220 => match pass_index {
            0 => vec![0x00; 4096],
            1 => vec![0xFF; 4096],
            _ => random_pattern(4096),
        },
        ShredMethod::SevenPass => match pass_index {
            0 => vec![0x00; 4096],
            1 => vec![0xFF; 4096],
            2 => vec![0xAA; 4096],
            3 => vec![0x55; 4096],
            _ => random_pattern(4096),
        },
        ShredMethod::Custom => random_pattern(4096),
    }
}

fn random_pattern(size: usize) -> Vec<u8> {
    let mut buffer = vec![0u8; size];
    rand::rng().fill_bytes(&mut buffer);
    buffer
}

fn overwrite_file(path: &Path, file_len: u64, pattern: &[u8]) -> Result<()> {
    let mut file = OpenOptions::new().write(true).open(path)?;
    let mut remaining = file_len;
    let mut offset = 0u64;

    while remaining > 0 {
        let chunk_len = remaining.min(pattern.len() as u64) as usize;
        file.write_all(&pattern[..chunk_len])?;
        remaining -= chunk_len as u64;
        offset += chunk_len as u64;
        file.seek(SeekFrom::Start(offset))?;
    }

    file.sync_all()?;
    Ok(())
}

pub fn is_likely_ssd(path: &Path) -> bool {
    let path_str = path.to_string_lossy().to_ascii_lowercase();
    path_str.contains("ssd") || path_str.contains("nvme")
}

pub fn shred_warning_message() -> &'static str {
    "Secure deletion cannot be guaranteed on SSDs or flash storage due to wear leveling and TRIM. \
     Overwriting may not reach all physical blocks. Use full-disk encryption for stronger protection."
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn shred_deletes_file() {
        let temp = NamedTempFile::new().unwrap();
        temp.as_file().write_all(b"secret data").unwrap();
        let path = temp.path().to_path_buf();
        let result = shred_file(&path, &ShredOptions::default()).unwrap();
        assert!(result.deleted);
        assert!(!path.exists());
    }

    #[test]
    fn dod_has_three_passes() {
        assert_eq!(
            effective_passes(&ShredOptions {
                method: ShredMethod::Dod5220,
                passes: 1,
            }),
            3
        );
    }

    #[test]
    fn seven_pass_has_seven_overwrites() {
        assert_eq!(
            effective_passes(&ShredOptions {
                method: ShredMethod::SevenPass,
                passes: 1,
            }),
            7
        );
    }
}
