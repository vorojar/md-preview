#!/usr/bin/env bash
set -euo pipefail

# Builds a .deb (via cargo-deb) and an AppImage for the current release binary.
# Neither bundles WebKitGTK/GTK — both rely on the system runtime, same as the
# existing tar.gz distribution (see README's Linux install notes).

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

APPIMAGETOOL="$ROOT/.tools/appimagetool"
APPIMAGETOOL_URL="https://github.com/AppImage/appimagetool/releases/download/continuous/appimagetool-x86_64.AppImage"

if ! command -v cargo-deb >/dev/null 2>&1; then
  echo "cargo-deb not found — install with: cargo install cargo-deb --locked" >&2
  exit 1
fi

if [ ! -x "$APPIMAGETOOL" ]; then
  mkdir -p "$ROOT/.tools"
  curl -L --fail -o "$APPIMAGETOOL" "$APPIMAGETOOL_URL"
  chmod +x "$APPIMAGETOOL"
fi

echo "== cargo build --release =="
cargo build --release

echo "== .deb (cargo-deb) =="
cargo deb --no-build

echo "== AppImage =="
rm -rf "$ROOT/AppDir"
mkdir -p "$ROOT/AppDir/usr/bin"
cp "$ROOT/target/release/md-preview" "$ROOT/AppDir/usr/bin/"
cp "$ROOT/assets/icon_1024.png" "$ROOT/AppDir/md-preview.png"
cp "$ROOT/assets/linux/md-preview.desktop" "$ROOT/AppDir/md-preview.desktop"
ln -sf usr/bin/md-preview "$ROOT/AppDir/AppRun"

mkdir -p "$ROOT/target/appimage"
ARCH=x86_64 "$APPIMAGETOOL" "$ROOT/AppDir" "$ROOT/target/appimage/MD-Preview-x86_64.AppImage"
rm -rf "$ROOT/AppDir"

echo ""
echo "Done:"
find "$ROOT/target/debian" -maxdepth 1 -name '*.deb'
echo "$ROOT/target/appimage/MD-Preview-x86_64.AppImage"
