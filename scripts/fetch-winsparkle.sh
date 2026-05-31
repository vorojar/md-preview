#!/usr/bin/env bash
set -euo pipefail

WINSPARKLE_VERSION="${WINSPARKLE_VERSION:-0.9.3}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CACHE_DIR="$ROOT/target/winsparkle"
DIST_DIR="$CACHE_DIR/WinSparkle-$WINSPARKLE_VERSION"
ARCHIVE="$CACHE_DIR/WinSparkle-$WINSPARKLE_VERSION.zip"
URL="https://github.com/vslavik/winsparkle/releases/download/v$WINSPARKLE_VERSION/WinSparkle-$WINSPARKLE_VERSION.zip"
SHA256="745985f41d2ab26b2d5a1cf87d76e4ed851039db19038e50610eb25ea0b73772"

sha256_file() {
  if command -v shasum >/dev/null 2>&1; then
    shasum -a 256 "$1" | awk '{print $1}'
  elif command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  elif command -v certutil.exe >/dev/null 2>&1; then
    certutil.exe -hashfile "$(cygpath -w "$1")" SHA256 \
      | awk '/^[0-9a-fA-F]{64}$/ {print tolower($0); exit}'
  else
    echo "error: no sha256 tool found" >&2
    exit 2
  fi
}

extract_zip() {
  if command -v unzip >/dev/null 2>&1; then
    unzip -q "$ARCHIVE" -d "$CACHE_DIR"
  elif command -v powershell.exe >/dev/null 2>&1; then
    powershell.exe -NoProfile -Command \
      "Expand-Archive -Force -Path '$(cygpath -w "$ARCHIVE")' -DestinationPath '$(cygpath -w "$CACHE_DIR")'"
  else
    echo "error: no unzip tool found" >&2
    exit 2
  fi
}

if [ ! -f "$DIST_DIR/x64/Release/WinSparkle.dll" ]; then
  mkdir -p "$CACHE_DIR"
  if [ ! -f "$ARCHIVE" ]; then
    curl -L --fail -o "$ARCHIVE" "$URL"
  fi
  actual="$(sha256_file "$ARCHIVE")"
  if [ "$actual" != "$SHA256" ]; then
    echo "error: WinSparkle archive checksum mismatch" >&2
    echo "expected: $SHA256" >&2
    echo "actual:   $actual" >&2
    exit 2
  fi
  rm -rf "$DIST_DIR"
  extract_zip
fi

echo "$DIST_DIR"
