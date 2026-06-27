# PrivyFile Security

## Threat Model

PrivyFile helps reduce accidental metadata leakage when sharing files and provides file shredding for local deletion. It is **not** a substitute for full-disk encryption or forensic-grade data destruction on all storage types.

## Secure Deletion Limitations

### SSDs and Flash Storage

Modern SSDs, NVMe drives, USB flash drives, and most mobile storage use wear leveling and TRIM commands. **Multi-pass overwriting cannot guarantee that all copies of data are destroyed.** PrivyFile displays this warning before shredding operations.

PrivyFile does **not** claim files are "forensically unrecoverable" on SSD/flash media.

### Hard Disk Drives (HDDs)

Multi-pass overwrite (including DoD 5220.22-M style) is more effective on traditional spinning hard drives but still depends on OS and filesystem behavior.

## Shredding Implementation

PrivyFile's shredder:

1. Opens the target file for writing
2. Overwrites the full file size with configured patterns per pass
3. Flushes buffers to disk
4. Deletes the file

Supported methods:

| Method | Passes |
|--------|--------|
| Random (1 pass) | 1 |
| DoD 5220.22-M style | 3 (0x00, 0xFF, random) |
| Secure wipe | 7 (0x00, 0xFF, 0xAA, 0x55, then random) |
| Custom | User-defined (1–35) |

## Metadata Removal

Metadata cleaning creates a new file copy by default. Original files are only modified or deleted when you explicitly choose shred options.

Cleaning effectiveness varies by format. ExifTool handles complex formats; Rust native handlers cover common JPEG/PNG cases.

## Permissions

The Tauri desktop app requests filesystem access to read and write files you select. PrivyFile does not access files outside user-initiated operations.

## Reporting Vulnerabilities

Please report security issues via GitHub Issues on [N0L0g1c/PrivyFile](https://github.com/N0L0g1c/PrivyFile). Do not disclose sensitive details publicly until addressed.

## Recommendations

- Use full-disk encryption (BitLocker, LUKS, FileVault)
- Use PrivyFile before sharing photos, documents, or videos
- For highly sensitive data on SSDs, rely on encryption rather than overwrite alone
