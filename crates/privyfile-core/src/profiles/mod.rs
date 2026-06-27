use crate::types::{CleanOptions, CleanProfileId, MetadataCategory};

pub fn apply_profile(mut options: CleanOptions) -> CleanOptions {
    if let Some(profile) = options.profile {
        options.categories = Some(categories_for_profile(profile));
    }
    options
}

pub fn categories_for_profile(profile: CleanProfileId) -> Vec<MetadataCategory> {
    match profile {
        CleanProfileId::SocialMediaShare => vec![
            MetadataCategory::Gps,
            MetadataCategory::Device,
            MetadataCategory::Author,
        ],
        CleanProfileId::LegalDocument => vec![
            MetadataCategory::Author,
            MetadataCategory::Other,
        ],
        CleanProfileId::PhotoBackup => vec![MetadataCategory::Gps],
        CleanProfileId::RemoveAll => vec![
            MetadataCategory::Gps,
            MetadataCategory::Device,
            MetadataCategory::Author,
            MetadataCategory::Timestamps,
            MetadataCategory::Other,
        ],
    }
}

pub fn profile_label(profile: CleanProfileId) -> &'static str {
    match profile {
        CleanProfileId::SocialMediaShare => "Social Media Share",
        CleanProfileId::LegalDocument => "Legal Document",
        CleanProfileId::PhotoBackup => "Photo Backup",
        CleanProfileId::RemoveAll => "Remove All Metadata",
    }
}

pub fn profile_description(profile: CleanProfileId) -> &'static str {
    match profile {
        CleanProfileId::SocialMediaShare => {
            "Remove GPS, device serials, and author info. Keeps date taken."
        }
        CleanProfileId::LegalDocument => {
            "Remove author, edit history, and comments. Keeps creation date."
        }
        CleanProfileId::PhotoBackup => "Remove GPS/location only. Keeps camera settings and dates.",
        CleanProfileId::RemoveAll => "Strip all metadata categories from the file.",
    }
}
