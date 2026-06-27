import { execSync } from "node:child_process";
import { existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");

if (process.platform !== "win32") {
  console.log(`Skipping installer ExifTool verification on ${process.platform}`);
  process.exit(0);
}

const wxs = join(root, "src-tauri", "target", "release", "wix", "x64", "main.wxs");
if (!existsSync(wxs)) {
  console.log("WiX file not found; skipping installer verification");
  process.exit(0);
}

execSync(
  "powershell -ExecutionPolicy Bypass -File ./scripts/verify-exiftool-installer.ps1",
  { stdio: "inherit", cwd: root },
);
