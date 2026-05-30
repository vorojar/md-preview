#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$ROOT"

if [ -f "$ROOT/.env.mobile-release" ]; then
  set -a
  # shellcheck disable=SC1091
  . "$ROOT/.env.mobile-release"
  set +a
fi

fail() {
  echo "[release-readiness] FAIL: $*" >&2
  exit 1
}

echo "[release-readiness] root: $ROOT"

test -f mobile/ios/MDPreviewMobile/PrivacyInfo.xcprivacy || fail "missing iOS privacy manifest"
plutil -lint mobile/ios/MDPreviewMobile/Info.plist mobile/ios/MDPreviewMobile/PrivacyInfo.xcprivacy >/dev/null
python3 -m json.tool mobile/ios/MDPreviewMobile/Assets.xcassets/Contents.json >/dev/null
python3 -m json.tool mobile/ios/MDPreviewMobile/Assets.xcassets/AppIcon.appiconset/Contents.json >/dev/null

echo "[release-readiness] Android release build"
(
  cd mobile/android
  gradle :app:assembleRelease :app:bundleRelease
)

APK="mobile/android/app/build/outputs/apk/release/app-release-unsigned.apk"
if [ -f mobile/android/app/build/outputs/apk/release/app-release.apk ]; then
  APK="mobile/android/app/build/outputs/apk/release/app-release.apk"
fi
test -f "$APK" || fail "missing Android release APK"
test -f mobile/android/app/build/outputs/bundle/release/app-release.aab || fail "missing Android release AAB"

AAPT="$(find "$HOME/Library/Android/sdk/build-tools" -name aapt -type f | sort | tail -1 || true)"
if [ -n "$AAPT" ]; then
  "$AAPT" dump permissions "$APK" | grep -q "INTERNET" && fail "Android release requests INTERNET permission"
  "$AAPT" dump xmltree "$APK" AndroidManifest.xml | grep -q "android.intent.action.VIEW" || fail "Android VIEW intent missing"
  "$AAPT" dump xmltree "$APK" AndroidManifest.xml | grep -q "text/markdown" || fail "Android markdown MIME missing"
fi

APKSIGNER="$(find "$HOME/Library/Android/sdk/build-tools" -name apksigner -type f | sort | tail -1 || true)"
if [ -n "${MD_PREVIEW_ANDROID_KEYSTORE:-}" ]; then
  if [ -n "$APKSIGNER" ]; then
    "$APKSIGNER" verify --verbose "$APK" >/dev/null || fail "Android release APK is not signed"
  fi
  jarsigner -verify mobile/android/app/build/outputs/bundle/release/app-release.aab >/dev/null 2>&1 || fail "Android release AAB is not signed"
else
  echo "[release-readiness] Android signing env not set; release artifacts are buildable but not store-uploadable"
fi

if command -v xcodegen >/dev/null 2>&1; then
  echo "[release-readiness] iOS project generation"
  (cd mobile/ios && xcodegen generate)
else
  fail "xcodegen missing"
fi

if command -v xcrun >/dev/null 2>&1; then
  echo "[release-readiness] iOS Swift parse"
  xcrun --sdk iphoneos swiftc -parse \
    mobile/ios/MDPreviewMobile/AppDelegate.swift \
    mobile/ios/MDPreviewMobile/PreviewViewController.swift
fi

echo "[release-readiness] OK"
