# Mobile release checklist

## Must pass before calling the mobile app release-ready

- Android: `mobile/scripts/verify-release-readiness.sh` passes and produces a signed `app-release.apk` plus `app-release.aab`.
- Android: install the signed release APK on an emulator or phone and verify the app launches.
- Android: install `mobile/android/app/build/outputs/apk/debug/app-debug.apk` on a real phone and open `.md`, `.markdown`, `.mdown`, `.mkd` files from Files, WeChat, and WeCom.
- Android: long-press a Markdown file, choose "Open with", select MD Preview, then verify the system offers it again as the default handler.
- Android: verify `ACTION_SEND` from WeChat/WeCom share sheet opens the same document.
- iOS: install on a real iPhone, open Markdown files from Files, WeChat, and WeCom via the share sheet / Open In.
- iOS: verify the app appears for `.md`, `.markdown`, `.mdown`, and `.mkd`; iOS does not allow silently forcing a default handler.
- Both: test UTF-8, UTF-8 with BOM, UTF-16 LE/BE, large documents, tables wider than the screen, local images, external links, KaTeX, and Mermaid.
- Both: verify external `http`, `https`, and `mailto` links open outside the preview; `javascript:` / `data:` links must not execute.
- Both: capture cold-open timing from tapping the file to first readable text. Target: ordinary Markdown under 500 ms on recent phones, with Mermaid/KaTeX deferred until after the document is readable.

## Release packaging

- iOS: set the production bundle id, signing team, app category, and archive with Xcode on a machine with the iOS platform installed.
- Android: keep release signing credentials in `.env.mobile-release` or CI secrets only, build an `.aab`, and verify with `bundletool` or an internal testing track.
- Store metadata: screenshots from a real phone, privacy copy emphasizing offline local rendering, and a short note that iOS default file handling depends on the user's Open In choice.

## Nice-to-have polish

- Recent files list stored locally on device.
- Search within document.
- Share/export rendered PDF.
- Optional edit mode with explicit save-copy behavior, not silent overwrite.
- Shared renderer golden tests so desktop and mobile keep matching for tables, task lists, math, Mermaid, code blocks, and local images.
