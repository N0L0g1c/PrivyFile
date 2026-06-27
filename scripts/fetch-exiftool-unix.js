import {
  chmodSync,
  cpSync,
  existsSync,
  mkdirSync,
  readFileSync,
  rmSync,
  writeFileSync,
} from "node:fs";
import { execSync } from "node:child_process";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { randomUUID } from "node:crypto";
import {
  binaryName,
  bundleDir,
  platformBundleName,
  readExifToolVersion,
} from "./exiftool-paths.js";

const platform = process.platform;
const bundle = platformBundleName(platform);
const dest = bundleDir(platform);
const version = readExifToolVersion();
const marker = join(dest, ".exiftool-version");
const exePath = join(dest, binaryName(platform));
const libDir = join(dest, "lib");

if (
  existsSync(marker) &&
  readFileSync(marker, "utf8").trim() === version &&
  existsSync(exePath) &&
  existsSync(libDir)
) {
  console.log(`ExifTool ${version} already present in ${dest}`);
  process.exit(0);
}

mkdirSync(dest, { recursive: true });

const url = `https://sourceforge.net/projects/exiftool/files/Image-ExifTool-${version}.tar.gz/download`;
const tempRoot = join(tmpdir(), `privyfile-exiftool-${randomUUID()}`);
const tarPath = join(tempRoot, "Image-ExifTool.tar.gz");
const extractDir = join(tempRoot, "extract");

try {
  mkdirSync(extractDir, { recursive: true });
  console.log(`Downloading ExifTool ${version} for ${bundle} from SourceForge`);

  execSync(`curl -fsSL -o "${tarPath}" "${url}"`, { stdio: "inherit" });
  execSync(`tar -xzf "${tarPath}" -C "${extractDir}"`, { stdio: "inherit" });

  const sourceRoot = join(extractDir, `Image-ExifTool-${version}`);
  const sourceExe = join(sourceRoot, "exiftool");
  const sourceLib = join(sourceRoot, "lib");

  if (!existsSync(sourceExe) || !existsSync(sourceLib)) {
    throw new Error(
      `Downloaded archive is missing exiftool or lib/ (expected ${sourceRoot})`,
    );
  }

  if (existsSync(exePath)) {
    rmSync(exePath, { force: true });
  }
  if (existsSync(libDir)) {
    rmSync(libDir, { recursive: true, force: true });
  }

  cpSync(sourceExe, exePath);
  cpSync(sourceLib, libDir, { recursive: true });
  chmodSync(exePath, 0o755);

  writeFileSync(marker, version, "utf8");
  console.log(`Installed ExifTool ${version} to ${dest}`);
} finally {
  rmSync(tempRoot, { recursive: true, force: true });
}
