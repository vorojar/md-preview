#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

WINSPARKLE_DIR="$(scripts/fetch-winsparkle.sh)"
test -f "$WINSPARKLE_DIR/x64/Release/WinSparkle.dll"
test -f "$WINSPARKLE_DIR/bin/winsparkle-tool.exe"

grep -q 'win_sparkle_set_appcast_url' src/main.rs
grep -q 'win_sparkle_set_eddsa_public_key' src/main.rs
grep -q 'appcast-windows.xml' src/main.rs
grep -q 'WinSparkle.dll' windows/installer.nsi
grep -q 'MD-Preview-windows-x64-Setup.exe' .github/workflows/release.yml

if [ "$(uname -s)" != "Darwin" ]; then
  echo "[winsparkle-verify] skip appcast signature: Sparkle sign_update verification requires macOS"
  exit 0
fi

mkdir -p target/verify-windows
INSTALLER="target/verify-windows/MD-Preview-windows-x64-Setup.exe"
printf 'dummy windows setup for appcast signing\n' > "$INSTALLER"
APPCAST="target/verify-windows/appcast-windows.xml"
scripts/generate-windows-appcast.sh "v$(grep -m1 '^version = ' Cargo.toml | cut -d '"' -f2)" "$INSTALLER" "$APPCAST" >/dev/null

python3 - <<'PY'
from pathlib import Path
import re

xml = Path("target/verify-windows/appcast-windows.xml").read_text()
assert "MD-Preview-windows-x64-Setup.exe" in xml
assert 'sparkle:os="windows-x64"' in xml
assert 'sparkle:installerArguments="/S"' in xml
assert "sparkle:edSignature" in xml
assert re.search(r"<sparkle:version>[^<]+</sparkle:version>", xml)
PY

echo "[winsparkle-verify] OK"
