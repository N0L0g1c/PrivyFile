import { execSync } from "node:child_process";
import { existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { binaryName, bundleDir } from "./exiftool-paths.js";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");

if (process.platform === "win32") {
  execSync(
    "powershell -ExecutionPolicy Bypass -File ./scripts/verify-exiftool-bundle.ps1",
    { stdio: "inherit", cwd: root },
  );
} else if (process.platform === "linux" || process.platform === "darwin") {
  execSync("bash ./scripts/verify-exiftool-bundle.sh", {
    stdio: "inherit",
    cwd: root,
  });
} else {
  console.log(`Skipping ExifTool bundle verification on ${process.platform}`);
  process.exit(0);
}

const exePath = join(bundleDir(), binaryName());
if (!existsSync(exePath)) {
  console.error("ExifTool bundle verification failed");
  process.exit(1);
}
