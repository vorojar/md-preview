#!/bin/bash
set -e

APP_NAME="MD Preview"
BUNDLE_ID="com.mdpreview.app"
MD_UTI="net.daringfireball.markdown"
BIN="target/release/md-preview"
APP_DIR="target/${APP_NAME}.app"
APP_VERSION="$(awk -F\" '/^version = / { print $2; exit }' Cargo.toml)"
SPARKLE_PUBLIC_KEY="fstkwGnjUNSrHFW4oq3LpBMQ1dhh9lQtax5K7nI0uoQ="
SPARKLE_FEED_URL="https://github.com/vorojar/md-preview/releases/latest/download/appcast.xml"

echo "Building release (arm64 + x86_64)..."
cargo build --release --target aarch64-apple-darwin
cargo build --release --target x86_64-apple-darwin

echo "Creating Universal Binary..."
lipo -create \
  target/aarch64-apple-darwin/release/md-preview \
  target/x86_64-apple-darwin/release/md-preview \
  -output target/release/md-preview-universal

echo "Creating app bundle..."
rm -rf "$APP_DIR"
mkdir -p "$APP_DIR/Contents/MacOS"
mkdir -p "$APP_DIR/Contents/Resources"
mkdir -p "$APP_DIR/Contents/Frameworks"
mkdir -p "$APP_DIR/Contents/PlugIns"

cp target/release/md-preview-universal "$APP_DIR/Contents/MacOS/md-preview"
cp assets/icon.icns "$APP_DIR/Contents/Resources/AppIcon.icns"
SPARKLE_DIR="$(scripts/fetch-sparkle.sh)"
ditto "$SPARKLE_DIR/Sparkle.framework" "$APP_DIR/Contents/Frameworks/Sparkle.framework"

echo "Building Finder Sync extension..."
FINDER_PROJECT="macos/finder-extension/MDPreviewFinderExtension.xcodeproj"
FINDER_DERIVED="target/finder-extension-derived"
xcodebuild \
  -project "$FINDER_PROJECT" \
  -scheme MDPreviewFinderExtension \
  -configuration Release \
  -derivedDataPath "$FINDER_DERIVED" \
  ARCHS="arm64 x86_64" \
  ONLY_ACTIVE_ARCH=NO \
  CODE_SIGNING_ALLOWED=NO \
  MARKETING_VERSION="$APP_VERSION" \
  CURRENT_PROJECT_VERSION="$APP_VERSION" \
  build >/dev/null
ditto \
  "$FINDER_DERIVED/Build/Products/Release/MDPreviewFinderExtension.appex" \
  "$APP_DIR/Contents/PlugIns/MDPreviewFinderExtension.appex"

cat > "$APP_DIR/Contents/Info.plist" << PLIST
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleDisplayName</key>
    <string>${APP_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>${BUNDLE_ID}</string>
    <key>CFBundleVersion</key>
    <string>${APP_VERSION}</string>
    <key>CFBundleShortVersionString</key>
    <string>${APP_VERSION}</string>
    <key>CFBundleExecutable</key>
    <string>md-preview</string>
    <key>CFBundleIconFile</key>
    <string>AppIcon</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>LSMinimumSystemVersion</key>
    <string>10.15</string>
    <key>SUFeedURL</key>
    <string>${SPARKLE_FEED_URL}</string>
    <key>SUPublicEDKey</key>
    <string>${SPARKLE_PUBLIC_KEY}</string>
    <key>SUEnableAutomaticChecks</key>
    <true/>
    <key>SUEnableInstallerLauncherService</key>
    <true/>
    <key>CFBundleURLTypes</key>
    <array>
        <dict>
            <key>CFBundleURLName</key>
            <string>MD Preview Finder Actions</string>
            <key>CFBundleURLSchemes</key>
            <array>
                <string>mdpreview</string>
            </array>
        </dict>
    </array>
    <key>CFBundleDocumentTypes</key>
    <array>
        <dict>
            <key>CFBundleTypeName</key>
            <string>Markdown Document</string>
            <key>CFBundleTypeRole</key>
            <string>Viewer</string>
            <key>LSHandlerRank</key>
            <string>Owner</string>
            <key>LSItemContentTypes</key>
            <array>
                <string>net.daringfireball.markdown</string>
            </array>
            <key>CFBundleTypeExtensions</key>
            <array>
                <string>md</string>
                <string>markdown</string>
                <string>mdown</string>
                <string>mkd</string>
            </array>
            <key>CFBundleTypeIconFile</key>
            <string>AppIcon</string>
        </dict>
    </array>
    <key>UTImportedTypeDeclarations</key>
    <array>
        <dict>
            <key>UTTypeIdentifier</key>
            <string>net.daringfireball.markdown</string>
            <key>UTTypeDescription</key>
            <string>Markdown Document</string>
            <key>UTTypeConformsTo</key>
            <array>
                <string>public.plain-text</string>
            </array>
            <key>UTTypeTagSpecification</key>
            <dict>
                <key>public.filename-extension</key>
                <array>
                    <string>md</string>
                    <string>markdown</string>
                    <string>mdown</string>
                    <string>mkd</string>
                </array>
            </dict>
        </dict>
    </array>
</dict>
</plist>
PLIST

codesign --force --sign - \
  --entitlements macos/finder-extension/Extension.entitlements \
  "$APP_DIR/Contents/PlugIns/MDPreviewFinderExtension.appex"
codesign --force --sign - "$APP_DIR"

echo "Done! App bundle at: $APP_DIR"
echo "Size: $(du -sh "$APP_DIR" | cut -f1)"
echo ""
echo "To install:"
echo "  ./install.sh"
