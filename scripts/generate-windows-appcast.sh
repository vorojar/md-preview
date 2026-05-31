#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TAG="${1:-}"
INSTALLER="${2:-}"
OUT="${3:-$ROOT/target/appcast-windows.xml}"

if [ -z "$TAG" ] || [ -z "$INSTALLER" ]; then
  echo "usage: $0 <tag> <windows-setup.exe> [appcast-windows.xml]" >&2
  exit 1
fi
if [ ! -f "$INSTALLER" ]; then
  echo "error: installer not found: $INSTALLER" >&2
  exit 2
fi

VERSION="${TAG#v}"
SPARKLE_DIR="$("$ROOT/scripts/fetch-sparkle.sh")"
SIGN_UPDATE="$SPARKLE_DIR/bin/sign_update"
ASSET_NAME="$(basename "$INSTALLER")"
DOWNLOAD_URL="https://github.com/vorojar/md-preview/releases/download/$TAG/$ASSET_NAME"
SIGNATURE_ATTRS="$("$SIGN_UPDATE" "$INSTALLER")"
LENGTH="$(wc -c < "$INSTALLER" | tr -d ' ')"
PUB_DATE="$(LC_ALL=C date -u '+%a, %d %b %Y %H:%M:%S +0000')"

mkdir -p "$(dirname "$OUT")"
cat > "$OUT" <<XML
<?xml version="1.0" encoding="utf-8"?>
<rss version="2.0" xmlns:sparkle="http://www.andymatuschak.org/xml-namespaces/sparkle">
  <channel>
    <title>MD Preview Windows Updates</title>
    <link>https://github.com/vorojar/md-preview/releases</link>
    <description>Windows updates for MD Preview.</description>
    <item>
      <title>MD Preview $VERSION</title>
      <sparkle:version>$VERSION</sparkle:version>
      <sparkle:shortVersionString>$VERSION</sparkle:shortVersionString>
      <pubDate>$PUB_DATE</pubDate>
      <enclosure
        url="$DOWNLOAD_URL"
        length="$LENGTH"
        type="application/octet-stream"
        sparkle:os="windows-x64"
        sparkle:installerArguments="/S"
        $SIGNATURE_ATTRS />
    </item>
  </channel>
</rss>
XML

echo "$OUT"
