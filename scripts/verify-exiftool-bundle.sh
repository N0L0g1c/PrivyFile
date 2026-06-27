#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
VERSION="$(tr -d '[:space:]' < "$ROOT/scripts/exiftool-version.txt")"

case "$(uname -s)" in
  Darwin) BUNDLE="macos" ;;
  Linux) BUNDLE="linux" ;;
  MINGW*|MSYS*|CYGWIN*)
    echo "Use verify-exiftool-bundle.ps1 on Windows"
    exit 1
    ;;
  *)
    echo "Unsupported platform: $(uname -s)"
    exit 1
    ;;
esac

DEST="$ROOT/src-tauri/binaries/$BUNDLE"
EXE="$DEST/exiftool"
LIB="$DEST/lib"

if [[ ! -f "$EXE" ]]; then
  echo "Missing bundled ExifTool executable: $EXE" >&2
  exit 1
fi

if [[ ! -d "$LIB" ]]; then
  echo "Missing bundled ExifTool lib directory: $LIB" >&2
  exit 1
fi

FILE_COUNT="$(find "$LIB" -type f | wc -l | tr -d ' ')"
if [[ "$FILE_COUNT" -lt 100 ]]; then
  echo "ExifTool lib bundle looks incomplete ($FILE_COUNT files in lib/)" >&2
  exit 1
fi

if [[ ! -x "$EXE" ]]; then
  chmod +x "$EXE"
fi

pushd "$DEST" >/dev/null
VER="$(./exiftool -ver)"
if [[ -z "$VER" ]]; then
  echo "Bundled ExifTool failed to run" >&2
  exit 1
fi

TEST_FILE="$(mktemp "${TMPDIR:-/tmp}/privyfile-exiftool-verify.XXXXXX.jpg")"
cleanup() {
  rm -f "$TEST_FILE"
  popd >/dev/null
}
trap cleanup EXIT

BASE64="/9j/4AAQSkZJRgABAQEASABIAAD/2wBDAP//////////////////////////////////////////////////////////////////////////////////////2wBDAf//////////////////////////////////////////////////////////////////////////////////////wAARCAABAAEDASIAAhEBAxEB/8QAFAABAAAAAAAAAAAAAAAAAAAACf/EABQQAQAAAAAAAAAAAAAAAAAAAAD/xAAUAQEAAAAAAAAAAAAAAAAAAAAA/8QAFBEBAAAAAAAAAAAAAAAAAAAAAP/aAAwDAQACEQMRAD8A0f/Z"
python3 - <<PY "$TEST_FILE" "$BASE64"
import base64, sys
path, data = sys.argv[1], sys.argv[2]
with open(path, "wb") as fh:
    fh.write(base64.b64decode(data))
PY

./exiftool "-GPSLatitude=47.6" "-GPSLongitude=-122.3" -overwrite_original "$TEST_FILE" >/dev/null
LAT="$(./exiftool -GPSLatitude -s3 "$TEST_FILE" || true)"
if [[ -z "$LAT" ]]; then
  echo "ExifTool failed to read/write metadata (lib/ may be broken)" >&2
  exit 1
fi

echo "Bundled ExifTool OK (version $VER)"
