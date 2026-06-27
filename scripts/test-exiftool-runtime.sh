#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

case "$(uname -s)" in
  Darwin) BUNDLE="macos" ;;
  Linux) BUNDLE="linux" ;;
  *)
    echo "Unsupported platform for runtime test: $(uname -s)" >&2
    exit 1
    ;;
esac

DEST="$ROOT/src-tauri/binaries/$BUNDLE"
CLI="$ROOT/target/release/privyfile-cli"

if [[ ! -f "$DEST/exiftool" || ! -d "$DEST/lib" ]]; then
  echo "Bundled ExifTool missing. Run: npm run fetch-exiftool" >&2
  exit 1
fi

TEST_FILE="$(mktemp "${TMPDIR:-/tmp}/privyfile-runtime.XXXXXX.jpg")"
OUTPUT_DIR="$(mktemp -d "${TMPDIR:-/tmp}/privyfile-runtime-out.XXXXXX")"
cleanup() {
  rm -f "$TEST_FILE"
  rm -rf "$OUTPUT_DIR"
  unset PRIVYFILE_EXIFTOOL_DIR
}
trap cleanup EXIT

BASE64="/9j/4AAQSkZJRgABAQEASABIAAD/2wBDAP//////////////////////////////////////////////////////////////////////////////////////2wBDAf//////////////////////////////////////////////////////////////////////////////////////wAARCAABAAEDASIAAhEBAxEB/8QAFAABAAAAAAAAAAAAAAAAAAAACf/EABQQAQAAAAAAAAAAAAAAAAAAAAD/xAAUAQEAAAAAAAAAAAAAAAAAAAAA/8QAFBEBAAAAAAAAAAAAAAAAAAAAAP/aAAwDAQACEQMRAD8A0f/Z"
python3 - <<PY "$TEST_FILE" "$BASE64"
import base64, sys
path, data = sys.argv[1], sys.argv[2]
with open(path, "wb") as fh:
    fh.write(base64.b64decode(data))
PY

pushd "$DEST" >/dev/null
./exiftool "-GPSLatitude=47.6" "-GPSLongitude=-122.3" -overwrite_original "$TEST_FILE" >/dev/null
popd >/dev/null

export PRIVYFILE_EXIFTOOL_DIR="$DEST"

pushd "$ROOT" >/dev/null
cargo build --release -p privyfile-cli --target-dir target

if [[ ! -x "$CLI" ]]; then
  echo "CLI binary not found at $CLI" >&2
  exit 1
fi

BEFORE="$("$CLI" metadata "$TEST_FILE" --json)"
if ! grep -qi 'gps' <<<"$BEFORE"; then
  echo "Test image has no GPS tags before clean" >&2
  exit 1
fi

"$CLI" clean "$TEST_FILE" --output "$OUTPUT_DIR"

CLEANED="$(find "$OUTPUT_DIR" -name '*-cleaned*.jpg' | head -n 1 || true)"
if [[ -z "$CLEANED" ]]; then
  echo "Clean did not produce an output file" >&2
  exit 1
fi

AFTER="$("$CLI" metadata "$CLEANED" --json)"
if grep -qi 'gps' <<<"$AFTER"; then
  echo "GPS tags remain after clean; ExifTool may not have been used" >&2
  exit 1
fi
popd >/dev/null

echo "ExifTool runtime clean test passed"
