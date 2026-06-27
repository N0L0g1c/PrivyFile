use crate::types::{MetadataCategory, TagEntry};

const GPS_PATTERNS: &[&str] = &[
    "gps", "location", "latitude", "longitude", "altitude", "coordinates", "geo",
];
const DEVICE_PATTERNS: &[&str] = &[
    "camera", "lens", "serial", "device", "make", "model", "software", "firmware",
    "bodyserial", "lensmodel", "uniquecamera",
];
const AUTHOR_PATTERNS: &[&str] = &[
    "author", "creator", "artist", "copyright", "owner", "producer", "publisher",
    "comment", "description", "title", "subject", "keywords", "company",
];
const TIMESTAMP_PATTERNS: &[&str] = &[
    "date", "time", "created", "modified", "digitized", "timestamp", "history",
];

pub fn categorize_tag(name: &str) -> MetadataCategory {
    let lower = name.to_lowercase();
    if matches_pattern(&lower, GPS_PATTERNS) {
        MetadataCategory::Gps
    } else if matches_pattern(&lower, DEVICE_PATTERNS) {
        MetadataCategory::Device
    } else if matches_pattern(&lower, AUTHOR_PATTERNS) {
        MetadataCategory::Author
    } else if matches_pattern(&lower, TIMESTAMP_PATTERNS) {
        MetadataCategory::Timestamps
    } else {
        MetadataCategory::Other
    }
}

fn matches_pattern(value: &str, patterns: &[&str]) -> bool {
    patterns.iter().any(|pattern| value.contains(pattern))
}

/// Raw metadata exposure risk (0 = none, 100 = severe). Higher means more sensitive data present.
pub fn compute_exposure_risk(tags: &[TagEntry]) -> u8 {
    let mut score = 0u32;
    for tag in tags {
        score += match tag.category {
            MetadataCategory::Gps => 30,
            MetadataCategory::Author => 20,
            MetadataCategory::Device => 15,
            MetadataCategory::Timestamps => 10,
            MetadataCategory::Other => 5,
        };
    }
    score.min(100) as u8
}

/// Privacy cleanliness rating (100 = fully clean, 0 = maximum metadata exposure).
pub fn compute_privacy_score(tags: &[TagEntry]) -> u8 {
    100 - compute_exposure_risk(tags)
}

pub fn filter_tags_by_categories(
    tags: &[TagEntry],
    categories: &[MetadataCategory],
) -> Vec<TagEntry> {
    tags.iter()
        .filter(|tag| categories.contains(&tag.category))
        .cloned()
        .collect()
}

pub fn tag_names_for_categories(categories: &[MetadataCategory]) -> Vec<String> {
    let mut names = Vec::new();
    for category in categories {
        match category {
            MetadataCategory::Gps => names.extend([
                "GPS*", "Composite:*GPS*", "XMP:GPS*", "Location*", "Geo*",
            ]),
            MetadataCategory::Device => names.extend([
                "Make", "Model", "SerialNumber", "Camera*", "Lens*", "Software",
                "Device*", "UniqueCameraModel", "BodySerialNumber",
            ]),
            MetadataCategory::Author => names.extend([
                "Author", "Creator", "Artist", "Copyright", "Owner", "Producer",
                "Publisher", "Comment", "Title", "Subject", "Keywords", "Company",
                "XMP:Creator*", "PDF:Author", "PDF:Creator", "PDF:Producer",
            ]),
            MetadataCategory::Timestamps => names.extend([
                "DateTime*", "CreateDate", "ModifyDate", "FileModifyDate",
                "FileCreateDate", "History*", "TrackCreateDate", "MediaCreateDate",
            ]),
            MetadataCategory::Other => names.push("-all:all"),
        }
    }
    names.iter().map(|s| s.to_string()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn privacy_score_weights() {
        let tags = vec![
            TagEntry {
                name: "GPSLatitude".into(),
                value: "59.9".into(),
                category: MetadataCategory::Gps,
            },
            TagEntry {
                name: "Author".into(),
                value: "Jane".into(),
                category: MetadataCategory::Author,
            },
        ];
        assert_eq!(compute_exposure_risk(&tags), 50);
        assert_eq!(compute_privacy_score(&tags), 50);
    }

    #[test]
    fn clean_file_has_perfect_privacy_score() {
        assert_eq!(compute_privacy_score(&[]), 100);
    }

    #[test]
    fn categorize_gps_tag() {
        assert_eq!(categorize_tag("GPSLongitude"), MetadataCategory::Gps);
        assert_eq!(categorize_tag("CameraModel"), MetadataCategory::Device);
    }
}
