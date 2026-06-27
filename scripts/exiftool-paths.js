import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

export const root = join(dirname(fileURLToPath(import.meta.url)), "..");

export function readExifToolVersion() {
  const version = readFileSync(
    join(root, "scripts", "exiftool-version.txt"),
    "utf8",
  ).trim();
  if (!version) {
    throw new Error("Missing ExifTool version in scripts/exiftool-version.txt");
  }
  return version;
}

export function platformBundleName(platform = process.platform) {
  if (platform === "win32") {
    return "win";
  }
  if (platform === "darwin") {
    return "macos";
  }
  if (platform === "linux") {
    return "linux";
  }
  throw new Error(`Unsupported platform for bundled ExifTool: ${platform}`);
}

export function bundleDir(platform = process.platform) {
  return join(root, "src-tauri", "binaries", platformBundleName(platform));
}

export function binaryName(platform = process.platform) {
  return platform === "win32" ? "exiftool.exe" : "exiftool";
}
