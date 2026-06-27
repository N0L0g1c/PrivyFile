use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataCategory {
    Gps,
    Device,
    Author,
    Timestamps,
    Other,
}

impl MetadataCategory {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Gps => "GPS / Location",
            Self::Device => "Device Info",
            Self::Author => "Author / Creator",
            Self::Timestamps => "Timestamps",
            Self::Other => "Other",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagEntry {
    pub name: String,
    pub value: String,
    pub category: MetadataCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataReport {
    pub file_path: String,
    pub file_type: String,
    pub tags: Vec<TagEntry>,
    pub privacy_score: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanOptions {
    pub categories: Option<Vec<MetadataCategory>>,
    pub output_dir: Option<String>,
    pub preserve_original: bool,
    pub shred_original: bool,
    pub profile: Option<CleanProfileId>,
}

impl Default for CleanOptions {
    fn default() -> Self {
        Self {
            categories: None,
            output_dir: None,
            preserve_original: true,
            shred_original: false,
            profile: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanResult {
    pub source_path: String,
    pub output_path: Option<String>,
    pub removed_tags: Vec<TagEntry>,
    pub privacy_score_before: u8,
    pub privacy_score_after: u8,
    pub bytes_before: u64,
    pub bytes_after: Option<u64>,
    pub original_shredded: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShredMethod {
    Random1Pass,
    Dod5220,
    SevenPass,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShredOptions {
    pub method: ShredMethod,
    pub passes: u8,
}

impl Default for ShredOptions {
    fn default() -> Self {
        Self {
            method: ShredMethod::Random1Pass,
            passes: 1,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShredResult {
    pub file_path: String,
    pub passes_completed: u8,
    pub bytes_overwritten: u64,
    pub deleted: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItem {
    pub path: String,
    pub action: BatchAction,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BatchAction {
    Clean,
    Shred,
    CleanAndShred,
    MetadataOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchProgress {
    pub current: usize,
    pub total: usize,
    pub file_path: String,
    pub status: String,
    pub percent: u8,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchResult {
    pub items: Vec<BatchItemResult>,
    pub report_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchItemResult {
    pub path: String,
    pub success: bool,
    pub message: String,
    pub clean_result: Option<CleanResult>,
    pub shred_result: Option<ShredResult>,
    pub metadata: Option<MetadataReport>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CleanProfileId {
    SocialMediaShare,
    LegalDocument,
    PhotoBackup,
    RemoveAll,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppSettings {
    pub output_dir: Option<String>,
    pub preserve_original: bool,
    pub default_shred_method: ShredMethod,
    pub default_shred_passes: u8,
    pub enable_history: bool,
    pub default_profile: CleanProfileId,
    pub watch_folder_enabled: bool,
    pub watch_folder_path: Option<String>,
}

impl Default for AppSettings {
    fn default() -> Self {
        Self {
            output_dir: None,
            preserve_original: true,
            default_shred_method: ShredMethod::Random1Pass,
            default_shred_passes: 1,
            enable_history: true,
            default_profile: CleanProfileId::RemoveAll,
            watch_folder_enabled: false,
            watch_folder_path: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub timestamp: String,
    pub source_path: String,
    pub output_path: Option<String>,
    pub action: String,
    pub privacy_score_before: u8,
    pub privacy_score_after: u8,
}

#[derive(Debug, thiserror::Error)]
pub enum PrivyFileError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Metadata error: {0}")]
    Metadata(String),
    #[error("Shred error: {0}")]
    Shred(String),
    #[error("File not found: {0}")]
    NotFound(String),
    #[error("Unsupported file type: {0}")]
    Unsupported(String),
}

pub type Result<T> = std::result::Result<T, PrivyFileError>;
