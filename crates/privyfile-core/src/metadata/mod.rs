mod categories;
mod exiftool;

use std::fs::File;
use std::io::BufReader;
use std::path::Path;

pub use categories::{categorize_tag, compute_privacy_score};
pub use exiftool::{
    bootstrap_exiftool_from_exe_dir, clean_with_exiftool, configure_exiftool_dir,
    exiftool_available, exiftool_bundle_dir, exiftool_version, metadata_report_from_tags,
    read_metadata_with_exiftool, resolve_exiftool_path, try_configure_exiftool_dir,
};

use crate::types::{
    CleanOptions, CleanResult, MetadataReport, PrivyFileError, Result, TagEntry,
};

pub fn detect_file_type(path: &Path) -> Option<&'static str> {
    path.extension()
        .and_then(|ext| ext.to_str())
        .map(|ext| ext.to_ascii_lowercase())
        .and_then(|ext| match ext.as_str() {
            "jpg" | "jpeg" => Some("jpeg"),
            "png" => Some("png"),
            "webp" => Some("webp"),
            "heic" | "heif" => Some("heic"),
            "tif" | "tiff" => Some("tiff"),
            "pdf" => Some("pdf"),
            "docx" => Some("docx"),
            "xlsx" => Some("xlsx"),
            "pptx" => Some("pptx"),
            "mp4" => Some("mp4"),
            "mov" => Some("mov"),
            "mp3" => Some("mp3"),
            "flac" => Some("flac"),
            "zip" => Some("zip"),
            "avi" | "mkv" | "webm" => Some("video"),
            _ => None,
        })
}

pub fn is_rust_native_image(file_type: &str) -> bool {
    matches!(file_type, "jpeg" | "png" | "webp")
}

pub fn get_metadata(path: &Path) -> Result<MetadataReport> {
    if !path.exists() {
        return Err(PrivyFileError::NotFound(path.to_string_lossy().into_owned()));
    }

    let file_type = detect_file_type(path)
        .ok_or_else(|| PrivyFileError::Unsupported(path.to_string_lossy().into_owned()))?;

    let tags = if exiftool_available() {
        read_metadata_with_exiftool(path)?
    } else if file_type == "jpeg" {
        read_jpeg_metadata(path).unwrap_or_default()
    } else {
        Vec::new()
    };

    Ok(metadata_report_from_tags(path, file_type, tags))
}

pub fn clean_file(path: &Path, options: &CleanOptions) -> Result<CleanResult> {
    if !path.exists() {
        return Err(PrivyFileError::NotFound(path.to_string_lossy().into_owned()));
    }

    let before = get_metadata(path)?;
    let bytes_before = std::fs::metadata(path)?.len();
    let file_type = detect_file_type(path)
        .ok_or_else(|| PrivyFileError::Unsupported(path.to_string_lossy().into_owned()))?;

    let output_path = resolve_output_path(path, options)?;
    let remove_all = options.categories.is_none();
    let categories = options.categories.clone().unwrap_or_default();
    let removed_tags = if remove_all {
        before.tags.clone()
    } else {
        before
            .tags
            .iter()
            .filter(|tag| categories.contains(&tag.category))
            .cloned()
            .collect()
    };

    if exiftool_available() {
        let patterns = if remove_all {
            Vec::new()
        } else {
            categories::tag_names_for_categories(&categories)
        };
        clean_with_exiftool(path, &output_path, remove_all, &patterns)?;
    } else if is_rust_native_image(file_type) {
        clean_native_image(path, &output_path)?;
    } else {
        return Err(PrivyFileError::Metadata(
            "ExifTool is required to clean this file type. Reinstall PrivyFile to restore the bundled copy.".into(),
        ));
    }

    let after = get_metadata(&output_path)?;
    let bytes_after = std::fs::metadata(&output_path)?.len();

    if !options.preserve_original && options.shred_original {
        crate::shred::shred_file(path, &Default::default())?;
    }

    Ok(CleanResult {
        source_path: path.to_string_lossy().into_owned(),
        output_path: Some(output_path.to_string_lossy().into_owned()),
        removed_tags,
        privacy_score_before: before.privacy_score,
        privacy_score_after: after.privacy_score,
        bytes_before,
        bytes_after: Some(bytes_after),
        original_shredded: !options.preserve_original && options.shred_original,
    })
}

fn resolve_output_path(path: &Path, options: &CleanOptions) -> Result<std::path::PathBuf> {
    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .unwrap_or("file");
    let extension = path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or("bin");

    let output_dir = options
        .output_dir
        .as_ref()
        .map(std::path::PathBuf::from)
        .or_else(|| path.parent().map(std::path::Path::to_path_buf))
        .ok_or_else(|| PrivyFileError::Metadata("Unable to resolve output directory".into()))?;

    std::fs::create_dir_all(&output_dir)?;

    let mut candidate = output_dir.join(format!("{stem}-cleaned.{extension}"));
    let mut counter = 1;
    while candidate.exists() {
        candidate = output_dir.join(format!("{stem}-cleaned-{counter}.{extension}"));
        counter += 1;
    }

    Ok(candidate)
}

fn read_jpeg_metadata(path: &Path) -> Result<Vec<TagEntry>> {
    let file = File::open(path)?;
    let mut reader = BufReader::new(file);
    let exif = exif::Reader::new()
        .read_from_container(&mut reader)
        .map_err(|error| PrivyFileError::Metadata(error.to_string()))?;

    let mut tags = Vec::new();
    for field in exif.fields() {
        let name = format!("{:?}", field.tag);
        tags.push(TagEntry {
            name: name.clone(),
            value: field.display_value().with_unit(&exif).to_string(),
            category: categorize_tag(&name),
        });
    }
    Ok(tags)
}

fn clean_native_image(path: &Path, output_path: &Path) -> Result<()> {
    let bytes = std::fs::read(path)?;
    let img = image::load_from_memory(&bytes)
        .map_err(|error| PrivyFileError::Metadata(error.to_string()))?;
    img.save(output_path)
        .map_err(|error| PrivyFileError::Metadata(error.to_string()))?;
    Ok(())
}

pub fn compare_metadata_size(before: u64, after: u64) -> i64 {
    after as i64 - before as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_jpeg() {
        assert_eq!(detect_file_type(Path::new("photo.JPG")), Some("jpeg"));
    }
}
