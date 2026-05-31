#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

if [ "$(uname -s)" != "Darwin" ]; then
  echo "[sparkle-verify] skip: Sparkle bundle verification requires macOS"
  exit 0
fi

./bundle.sh

APP="target/MD Preview.app"
PLIST="$APP/Contents/Info.plist"
FRAMEWORK="$APP/Contents/Frameworks/Sparkle.framework"

test -x "$APP/Contents/MacOS/md-preview"
test -d "$FRAMEWORK"
test -x "$FRAMEWORK/Sparkle"
test -x "$FRAMEWORK/Autoupdate"
test -d "$FRAMEWORK/Updater.app"

/usr/libexec/PlistBuddy -c 'Print :SUFeedURL' "$PLIST" | grep -qx 'https://github.com/vorojar/md-preview/releases/latest/download/appcast.xml'
/usr/libexec/PlistBuddy -c 'Print :SUPublicEDKey' "$PLIST" | grep -qx 'fstkwGnjUNSrHFW4oq3LpBMQ1dhh9lQtax5K7nI0uoQ='
/usr/libexec/PlistBuddy -c 'Print :SUEnableAutomaticChecks' "$PLIST" | grep -qx 'true'

BENCH_OUTPUT="$(MD_PREVIEW_BENCH=1 "$APP/Contents/MacOS/md-preview" 2>&1)"
grep -q 'native_updater_started' <<<"$BENCH_OUTPUT"

if [ -f target/MD-Preview-macOS-universal.dmg ]; then
  APPCAST="target/test-appcast.xml"
  ./scripts/generate-appcast.sh "v$(/usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$PLIST")" \
    target/MD-Preview-macOS-universal.dmg "$APPCAST" >/dev/null
  python3 - <<'PY'
from pathlib import Path
import re
xml = Path("target/test-appcast.xml").read_text()
assert "sparkle:edSignature" in xml
assert "MD-Preview-macOS-universal.dmg" in xml
assert re.search(r"<sparkle:version>[^<]+</sparkle:version>", xml)
PY
fi

echo "[sparkle-verify] OK"
