import { execSync } from "node:child_process";
import { existsSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { binaryName, bundleDir } from "./exiftool-paths.js";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");

if (process.platform === "win32") {
  execSync(
    "powershell -ExecutionPolicy Bypass -File ./scripts/fetch-exiftool.ps1",
    { stdio: "inherit", cwd: root },
  );
} else if (process.platform === "linux" || process.platform === "darwin") {
  await import("./fetch-exiftool-unix.js");
} else {
  console.error(`Unsupported platform for bundled ExifTool: ${process.platform}`);
  process.exit(1);
}

const exePath = join(bundleDir(), binaryName());
if (!existsSync(exePath)) {
  console.error(`ExifTool fetch failed: missing ${exePath}`);
  process.exit(1);
}
