#!/bin/bash
set -e

APP_NAME="MD Preview"
BUNDLE_ID="com.mdpreview.app"
MD_UTI="net.daringfireball.markdown"
APP_DIR="target/${APP_NAME}.app"

if [ ! -d "$APP_DIR" ]; then
    echo "App bundle not found. Run ./bundle.sh first."
    exit 1
fi

echo "Installing to /Applications..."
cp -r "$APP_DIR" "/Applications/"

echo "Registering app with Launch Services..."
/System/Library/Frameworks/CoreServices.framework/Versions/A/Frameworks/LaunchServices.framework/Versions/A/Support/lsregister -f "/Applications/${APP_NAME}.app"

echo "Setting MD Preview as default app for .md files..."
swift - <<'SWIFT'
import Foundation
import CoreServices
let uti = "net.daringfireball.markdown" as NSString
let bundleId = "com.mdpreview.app" as NSString
let result = LSSetDefaultRoleHandlerForContentType(uti, .viewer, bundleId)
if result == noErr { print("Default handler set.") }
else { print("Warning: could not set default handler (error \(result))") }
SWIFT

echo ""
echo "Done! MD Preview is now the default app for .md files."
echo "Double-click any .md file in Finder to open with MD Preview."
