# MD Preview Mobile

手机端 MVP 目标是快速只读预览 Markdown 文件，重点覆盖从微信、企业微信、Files/文件管理器或系统分享面板打开 `.md` 文档。

## Scope

- iOS: 原生 UIKit + `WKWebView`，声明 Markdown 文档类型，支持系统文档选择器和 Open In。
- Android: 原生 Java + `WebView`，声明 `ACTION_VIEW` / `ACTION_SEND` intent filter，支持参与 Markdown 默认打开器选择。
- 渲染层: `mobile/shared` 中的一份离线 HTML/JS/CSS，首屏只加载 `marked` 和 highlight.js，KaTeX/Mermaid 按文档内容延迟加载。

## Build

```bash
# iOS project
cd mobile/ios
xcodegen generate
xcodebuild -project MDPreviewMobile.xcodeproj -scheme MDPreviewMobile -destination 'generic/platform=iOS' CODE_SIGNING_ALLOWED=NO build

# Android
cd mobile/android
gradle :app:assembleDebug
```

## Release

```bash
# One-time local Android upload key. Creates ignored files:
# .env.mobile-release and mobile/android/signing/md-preview-upload.keystore
mobile/scripts/generate-android-upload-keystore.sh

# Build signed Android APK/AAB and generate/check the iOS project.
mobile/scripts/build-release.sh

# Release readiness checks used by the root verify script.
mobile/scripts/verify-release-readiness.sh
```

Android release artifacts:

- `mobile/android/app/build/outputs/apk/release/app-release.apk`
- `mobile/android/app/build/outputs/bundle/release/app-release.aab`

## Notes

- iOS 不允许 App 静默把自己设为系统默认打开器；声明文档类型后，用户可以在分享/打开方式中选择 MD Preview。
- Android 的默认打开器由系统 resolver 和用户选择决定；本工程声明了 Markdown 常见 MIME 和扩展名入口。
- Android upload keystore 和 `.env.mobile-release` 是本机发布凭据，已被 `.gitignore` 忽略；不要提交或发到聊天/文档里。
- iOS archive 需要安装可用 iOS platform，并设置 `MD_PREVIEW_IOS_TEAM_ID` 或在 Xcode 中配置签名团队。
- 发布前真机验收清单见 `mobile/RELEASE_CHECKLIST.md`。
