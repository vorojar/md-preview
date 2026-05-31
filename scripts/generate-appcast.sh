#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TAG="${1:-}"
DMG="${2:-}"
OUT="${3:-$ROOT/target/appcast.xml}"

if [ -z "$TAG" ] || [ -z "$DMG" ]; then
  echo "usage: $0 <tag> <signed-dmg> [appcast.xml]" >&2
  exit 1
fi
if [ ! -f "$DMG" ]; then
  echo "error: dmg not found: $DMG" >&2
  exit 2
fi

VERSION="${TAG#v}"
SPARKLE_DIR="$("$ROOT/scripts/fetch-sparkle.sh")"
SIGN_UPDATE="$SPARKLE_DIR/bin/sign_update"
ASSET_NAME="$(basename "$DMG")"
DOWNLOAD_URL="https://github.com/vorojar/md-preview/releases/download/$TAG/$ASSET_NAME"
SIGNATURE_ATTRS="$("$SIGN_UPDATE" "$DMG")"
PUB_DATE="$(LC_ALL=C date -u '+%a, %d %b %Y %H:%M:%S +0000')"

mkdir -p "$(dirname "$OUT")"
cat > "$OUT" <<XML
<?xml version="1.0" encoding="utf-8"?>
<rss version="2.0" xmlns:sparkle="http://www.andymatuschak.org/xml-namespaces/sparkle">
  <channel>
    <title>MD Preview Updates</title>
    <link>https://github.com/vorojar/md-preview/releases</link>
    <description>Updates for MD Preview.</description>
    <item>
      <title>MD Preview $VERSION</title>
      <sparkle:version>$VERSION</sparkle:version>
      <sparkle:shortVersionString>$VERSION</sparkle:shortVersionString>
      <pubDate>$PUB_DATE</pubDate>
      <enclosure url="$DOWNLOAD_URL" $SIGNATURE_ATTRS type="application/octet-stream" />
    </item>
  </channel>
</rss>
XML

echo "$OUT"
