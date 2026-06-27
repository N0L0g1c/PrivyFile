export type MetadataCategory =
  | "gps"
  | "device"
  | "author"
  | "timestamps"
  | "other";

export interface TagEntry {
  name: string;
  value: string;
  category: MetadataCategory;
}

export interface MetadataReport {
  file_path: string;
  file_type: string;
  tags: TagEntry[];
  privacy_score: number;
}

export interface CleanOptions {
  categories?: MetadataCategory[];
  output_dir?: string;
  preserve_original: boolean;
  shred_original: boolean;
  profile?: CleanProfileId;
}

export interface CleanResult {
  source_path: string;
  output_path?: string;
  removed_tags: TagEntry[];
  privacy_score_before: number;
  privacy_score_after: number;
  bytes_before: number;
  bytes_after?: number;
  original_shredded: boolean;
}

export type ShredMethod = "random1_pass" | "dod5220" | "seven_pass" | "custom";

export interface ShredOptions {
  method: ShredMethod;
  passes: number;
}

export interface ShredResult {
  file_path: string;
  passes_completed: number;
  bytes_overwritten: number;
  deleted: boolean;
}

export type BatchAction = "clean" | "shred" | "clean_and_shred" | "metadata_only";

export interface BatchItem {
  path: string;
  action: BatchAction;
}

export interface BatchProgress {
  current: number;
  total: number;
  file_path: string;
  status: string;
  percent: number;
}

export interface BatchItemResult {
  path: string;
  success: boolean;
  message: string;
  clean_result?: CleanResult;
  shred_result?: ShredResult;
  metadata?: MetadataReport;
}

export interface BatchResult {
  items: BatchItemResult[];
  report_path?: string;
}

export type CleanProfileId =
  | "social_media_share"
  | "legal_document"
  | "photo_backup"
  | "remove_all";

export interface ProfileInfo {
  id: CleanProfileId;
  label: string;
  description: string;
}

export interface AppSettings {
  output_dir?: string;
  preserve_original: boolean;
  default_shred_method: ShredMethod;
  default_shred_passes: number;
  enable_history: boolean;
  default_profile: CleanProfileId;
  watch_folder_enabled: boolean;
  watch_folder_path?: string;
}

export interface HistoryEntry {
  timestamp: string;
  source_path: string;
  output_path?: string;
  action: string;
  privacy_score_before: number;
  privacy_score_after: number;
}

export interface QueuedFile {
  id: string;
  path: string;
  name: string;
  metadata?: MetadataReport;
  status: "pending" | "processing" | "done" | "error";
  message?: string;
}

export const DEFAULT_SETTINGS: AppSettings = {
  preserve_original: true,
  default_shred_method: "random1_pass",
  default_shred_passes: 1,
  enable_history: true,
  default_profile: "remove_all",
  watch_folder_enabled: false,
};

export const CATEGORY_LABELS: Record<MetadataCategory, string> = {
  gps: "GPS / Location",
  device: "Device Info",
  author: "Author / Creator",
  timestamps: "Timestamps",
  other: "Other",
};
