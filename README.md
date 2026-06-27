# PrivyFile

**Sanitize files before you share or delete them.**

PrivyFile is a privacy-first, local-only desktop tool from [VassDev Studio](https://vassbrekke.no/vassdev/) that removes file metadata and securely deletes sensitive files.

## Features

- Drag-and-drop interface for files and folders
- Metadata removal for JPEG, PNG, WebP, HEIC, TIFF, PDF, Office docs, MP4/MOV, and more
- Preview metadata and privacy score (0–100) before cleaning
- Non-destructive cleaning by default (`-cleaned` copy)
- Secure multi-pass file shredding (Random, DoD 5220.22-M, custom)
- Batch processing with progress and HTML/JSON reports
- Smart cleaning profiles (Social Media, Legal Document, Photo Backup)
- Watch folder mode for automatic cleaning
- CLI for scripting and headless use
- Zero telemetry — all processing happens locally

## Tech Stack

- **Frontend:** React 19, TypeScript, Tailwind CSS v4
- **Desktop:** Tauri v2
- **Backend:** Rust (`privyfile-core`)
- **Metadata:** Bundled ExifTool + Rust native handlers for common images

## Development

### Prerequisites

- Node.js 22+
- Rust stable (MSVC on Windows)
- WebView2 (included on Windows 11)

### Setup

```powershell
cd "C:\Users\Username\Documents\Code Projects\PrivyFile"
npm install
npm run tauri dev
```

### CLI

```powershell
cargo run -p privyfile-cli -- metadata photo.jpg --json
cargo run -p privyfile-cli -- clean photo.jpg --output ./cleaned/
cargo run -p privyfile-cli -- shred document.pdf --method dod
cargo run -p privyfile-cli -- batch ./folder --clean --recursive
```

### ExifTool bundling (Windows, Linux, macOS)

ExifTool is **downloaded automatically** before `npm run tauri dev` and `npm run tauri build`:

```bash
npm run fetch-exiftool      # platform-specific bundle under src-tauri/binaries/
npm run verify-exiftool     # sanity-check bundled binary + support files
npm run test:exiftool-runtime
```

| Platform | Bundle path | Layout |
|----------|-------------|--------|
| Windows | `src-tauri/binaries/win/` | `exiftool.exe` + `exiftool_files/` |
| Linux | `src-tauri/binaries/linux/` | `exiftool` + `lib/` |
| macOS | `src-tauri/binaries/macos/` | `exiftool` + `lib/` |

Version is pinned in `scripts/exiftool-version.txt`. Windows downloads `exiftool-{version}_64.zip`; Linux and macOS download `Image-ExifTool-{version}.tar.gz` from SourceForge. Bundled copies are included in Tauri installers at `{InstallDir}/binaries/{platform}/`.

### GitHub Releases

Pushing a version tag builds all platforms and uploads installers to [GitHub Releases](https://github.com/N0L0g1c/PrivyFile/releases):

```bash
# Bump version in package.json and src-tauri/tauri.conf.json first
git tag v0.1.0
git push origin v0.1.0
```

The `Release PrivyFile` workflow produces Windows (MSI + NSIS), Linux (`.deb` / AppImage), and macOS (`.dmg`) assets.

If a release completes without installer files attached, open **Actions → Release PrivyFile**, inspect the failed platform job, fix the issue, then re-run the workflow or push an updated tag:

```bash
git tag -d v0.1.0
git push origin :refs/tags/v0.1.0
git tag v0.1.0
git push origin v0.1.0
```

## Context menu integration

- **Windows:** run `scripts/install-context-menu.ps1`
- **Linux:** copy `scripts/privyfile-context.desktop` to `~/.local/share/applications/` and adjust paths

## License

MIT — see [LICENSE](LICENSE)

## Privacy

See [PRIVACY.md](PRIVACY.md) and [SECURITY.md](SECURITY.md).
