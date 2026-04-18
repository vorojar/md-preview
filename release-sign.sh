#!/usr/bin/env bash
# release-sign.sh <tag>
#
# One-shot post-tag step for macOS distribution.
#
# Prerequisite: you've already pushed the tag (e.g. `git push origin v0.4.0`),
# and the GitHub Actions release workflow is running or finished.
#
# This script:
#   1. waits for the Release's unsigned MD-Preview-macOS-universal.dmg to land
#   2. downloads it
#   3. signs + notarizes + staples (both inner .app AND the dmg) via the
#      remote signing machine at yihafo1109@192.168.3.207
#   4. uploads the signed dmg back to the Release, overwriting the unsigned one
#   5. drops the stapled .app into target/ and /Applications if that copy exists
#
# Expected end state: the Release's macOS dmg and the .app inside it both
# pass `stapler validate`, `codesign --verify`, and `spctl --assess` —
# Gatekeeper accepts them offline, no warnings for end users.

set -euo pipefail

TAG="${1:-}"
if [ -z "$TAG" ]; then
  echo "usage: $0 <tag>  e.g. $0 v0.4.0" >&2
  exit 1
fi

REPO="vorojar/md-preview"
ASSET="MD-Preview-macOS-universal.dmg"
SIGN_SCRIPT="$HOME/.claude/skills/remote-mac-sign/sign_remote.sh"

if [ ! -x "$SIGN_SCRIPT" ]; then
  echo "error: remote-mac-sign skill not found at $SIGN_SCRIPT" >&2
  exit 2
fi

WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT

echo "[1/5] waiting for $TAG Release to expose $ASSET (poll 15s, up to 15min)..."
for i in $(seq 1 60); do
  if gh release view "$TAG" -R "$REPO" --json assets -q \
      ".assets[] | select(.name==\"$ASSET\") | .name" 2>/dev/null \
      | grep -q "^$ASSET$"; then
    echo "    found."
    break
  fi
  if [ "$i" -eq 60 ]; then
    echo "    timed out waiting for $ASSET; check GH Actions for failures." >&2
    exit 3
  fi
  sleep 15
done

echo "[2/5] downloading unsigned dmg..."
cd "$WORK"
gh release download "$TAG" -R "$REPO" -p "$ASSET" --clobber

echo "[3/5] signing + notarizing + stapling (this takes ~5min, Apple notary x2)..."
"$SIGN_SCRIPT" "$WORK/$ASSET"

SIGNED="$WORK/signed-output/signed_$ASSET"
if [ ! -f "$SIGNED" ]; then
  echo "    signing returned no output at $SIGNED" >&2
  exit 4
fi

# Quick sanity check before uploading.
xcrun stapler validate "$SIGNED" >/dev/null || { echo "    stapler validate failed" >&2; exit 5; }
spctl -a -t open --context context:primary-signature "$SIGNED" >/dev/null 2>&1 \
  || { echo "    spctl assess failed" >&2; exit 6; }

echo "[4/5] uploading signed dmg to $TAG (replacing unsigned)..."
cp "$SIGNED" "$WORK/$ASSET"
gh release upload "$TAG" "$WORK/$ASSET" -R "$REPO" --clobber

echo "[5/5] deploying stapled .app locally..."
MOUNT=$(mktemp -d)/mnt
mkdir -p "$MOUNT"
hdiutil attach "$SIGNED" -nobrowse -mountpoint "$MOUNT" >/dev/null

REPO_ROOT="$(cd "$(dirname "$0")" && pwd)"
TARGET_APP="$REPO_ROOT/target/MD Preview.app"
if [ -d "$MOUNT/MD Preview.app" ]; then
  rm -rf "$TARGET_APP"
  mkdir -p "$REPO_ROOT/target"
  ditto "$MOUNT/MD Preview.app" "$TARGET_APP"
  echo "    replaced $TARGET_APP"
fi

APPL="/Applications/MD Preview.app"
if [ -d "$APPL" ] && [ -w "/Applications" ]; then
  rm -rf "$APPL"
  ditto "$MOUNT/MD Preview.app" "$APPL"
  echo "    replaced $APPL"
elif [ -d "$APPL" ]; then
  echo "    /Applications not writable; skipping system-wide replace"
fi

hdiutil detach "$MOUNT" >/dev/null

echo ""
echo "DONE. $TAG: dmg and inner .app both signed + notarized + stapled."
echo "Release: https://github.com/$REPO/releases/tag/$TAG"
