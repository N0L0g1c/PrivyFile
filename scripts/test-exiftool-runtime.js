import { execSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");

if (process.platform === "win32") {
  execSync(
    "powershell -ExecutionPolicy Bypass -File ./scripts/test-exiftool-runtime.ps1",
    { stdio: "inherit", cwd: root },
  );
} else {
  execSync("bash ./scripts/test-exiftool-runtime.sh", {
    stdio: "inherit",
    cwd: root,
  });
}
