#!/usr/bin/env bash
set -euo pipefail

SPARKLE_VERSION="${SPARKLE_VERSION:-2.9.2}"
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
CACHE_DIR="$ROOT/target/sparkle"
DIST_DIR="$CACHE_DIR/Sparkle-$SPARKLE_VERSION"
ARCHIVE="$CACHE_DIR/Sparkle-$SPARKLE_VERSION.tar.xz"
URL="https://github.com/sparkle-project/Sparkle/releases/download/$SPARKLE_VERSION/Sparkle-$SPARKLE_VERSION.tar.xz"

if [ ! -d "$DIST_DIR/Sparkle.framework" ]; then
  mkdir -p "$CACHE_DIR"
  if [ ! -f "$ARCHIVE" ]; then
    curl -L --fail -o "$ARCHIVE" "$URL"
  fi
  rm -rf "$DIST_DIR"
  mkdir -p "$DIST_DIR"
  tar -xf "$ARCHIVE" -C "$DIST_DIR"
fi

echo "$DIST_DIR"
