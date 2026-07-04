# 当前任务：修复 macOS Cmd+W 并发布 v1.1.25

## 目标

- 修复 GitHub issue #32：macOS 端用 Finder 双击打开 Markdown 后，`Command+W` 应能按系统习惯关闭预览窗口。
- 发布桌面版 `v1.1.25`，包含签名、公证、staple、Release notes、issue 回复和关闭。

## 非目标

- 不调整多窗口模型；当前仍按单窗口预览器处理，关闭窗口等同退出本次预览会话。
- 不处理 #29、#28、#19。

## 验收场景

- [x] macOS File 菜单包含标准 `Close Window`，快捷键为 `Cmd+W`，走 AppKit `performClose:` 响应链。
- [x] 现有打开、搜索、编辑、保存、打印快捷键不回退。
- [x] `scripts/verify.sh` 通过。
- [ ] GitHub Release `v1.1.25` 发布说明来自 `CHANGELOG.md` 对应段落，macOS DMG 已签名、公证并 staple。
- [ ] issue #32 已用中文回复并关闭。

## 执行记录

- [x] 确认 #32 为真实用户反馈：macOS 缺少轻量文件窗口常见的 `Command+W` 关闭行为。
- [x] 在 macOS 原生 File 菜单中新增 `Close Window` / `Cmd+W`，action 使用系统 `performClose:`。
- [x] 更新版本号、CHANGELOG、README 快捷键和官网结构化版本号。

## 验证记录

```text
cargo check
cargo test
./scripts/verify.sh

结果：通过。统一验证覆盖桌面快捷键静态检查、cargo test、桌面锚点/搜索/Sparkle 更新、Windows 自更新、iOS 生成/构建、Android debug/release 构建和移动端渲染检查。
```

---

# 上一任务存档：iOS signing / TestFlight 发布准备

## 目标

- 尽量完成 MD Preview iOS 版本发布前的本机准备工作。
- 将 iOS 版本号对齐到当前 mobile release 线。
- 验证 iOS 构建、归档、模拟器安装启动和移动端渲染。
- 配好 iOS signing team、App Store Connect API key、本机开发签名、真机安装和分发 IPA。
- 创建 App Store Connect app record、TestFlight 内部测试组，并上传首个 TestFlight build。
- 补齐 App Store metadata、截图、隐私、内容权利、分类、价格和销售范围。

## 非目标

- 不使用 experimental `asc web` 私有接口自动创建 App Store Connect app record，除非用户显式确认。
- 不直接提交 App Store 审核；正式提交仍需要 App Review 真实联系人并单独确认。
- 不修改 Android 发布配置。

## 验收场景

- [x] iOS `MARKETING_VERSION` 对齐到 `1.0.7`，`CURRENT_PROJECT_VERSION` 对齐到 `8`。
- [x] iOS project generation 成功。
- [x] iOS simulator build 成功，并可安装启动到 iPhone 17 Pro Simulator。
- [x] 首屏截图确认 app 启动到空状态，没有崩溃或明显遮挡。
- [x] iOS generic Release archive 在 `CODE_SIGNING_ALLOWED=NO` 下成功，证明代码和 archive 路径可用。
- [x] mobile renderer golden 验证通过。
- [x] mobile release readiness 通过。
- [x] App Store Connect API key 存入系统钥匙串并通过网络校验。
- [x] `app.mdpreview.mobile` 显式 bundle id 创建成功。
- [x] Xcode automatic signing 配置 Team `BUR55497B4`，真机构建、安装、启动成功。
- [x] App Store Connect 分发 IPA 导出成功，签名为 `Apple Distribution`。
- [x] App Store Connect app record 创建成功，app id 为 `6779451523`。
- [x] TestFlight 内部测试组创建成功，group id 为 `061ed3ee-dd2a-4449-87f8-3967c065ba1e`。
- [x] TestFlight build `1.0.7 (8)` 上传成功并处理为 `VALID`。
- [x] App Store version `1.0.7` 绑定 build `eadcd636-878e-40a4-95ee-f9ce93b86133`。
- [x] App Store metadata、隐私政策 URL、支持 URL、分类、内容权利、年龄分级、免费价格和 175 个国家/地区销售范围已配置。
- [x] iPhone 6.5-inch 与 iPad Pro 12.9-inch 截图各 3 张已重新生成并上传，重点展示 Mermaid diagrams、KaTeX notes、搜索和 README 渲染，均返回 `COMPLETE`。
- [x] App Privacy 已发布为“未收集数据”。
- [x] App Review 联系人信息已配置，版本已提交审核，状态为 `WAITING_FOR_REVIEW`。

## 执行记录

- [x] `mobile/ios/project.yml` 版本从 `1.0.6` build `7` 更新为 `1.0.7` build `8`。
- [x] `mobile/ios/project.yml` 配置 `DEVELOPMENT_TEAM: BUR55497B4` 和 `CODE_SIGN_STYLE: Automatic`。
- [x] 通过 `xcodegen generate` 重新生成本地 Xcode project。
- [x] 构建并安装到 iPhone 17 Pro Simulator，启动 bundle id `app.mdpreview.mobile`。
- [x] 截图保存到 `target/ios-qa/ios-simulator-launch.png`。
- [x] 生成无签名 Release archive：`mobile/ios/build/MDPreviewMobile.xcarchive`。
- [x] 使用用户提供的新 ASC API key 登录 `asc`，凭据存储在系统钥匙串。
- [x] 创建 bundle id：`6P439S39PG` / `app.mdpreview.mobile` / Team `BUR55497B4`。
- [x] Xcode 自动创建 Apple Development 证书和开发 profile，真机安装到连接的 iPhone。
- [x] 导出 App Store Connect IPA：`mobile/ios/build/export/MD Preview.ipa`。
- [x] 通过 App Store Connect 网页创建 app record：`Local Markdown Preview` / `app.mdpreview.mobile` / SKU `md-preview-ios`。
- [x] 创建 TestFlight 内部测试组：`Internal Testers`。
- [x] 上传 `mobile/ios/build/export/MD Preview.ipa` 到 TestFlight，并挂到 `Internal Testers`。
- [x] 新增公开页面：`docs/privacy.html` 和 `docs/support.html`。
- [x] 配置 App Store version：`1.0.7`、版权、关联 build、description、keywords、promotional text、support URL、marketing URL。
- [x] 配置 app-level 信息：名称 `Local Markdown Preview`、subtitle `Open Markdown files locally`、privacy policy URL、内容权利 `DOES_NOT_USE_THIRD_PARTY_CONTENT`。
- [x] 配置分类：primary `PRODUCTIVITY`、secondary `DEVELOPER_TOOLS`。
- [x] 配置年龄分级为 safe defaults：不含广告、赌博、聊天、用户生成内容、不受限网页访问等。
- [x] 配置免费价格，并通过网页初始化所有 175 个国家/地区发布时供应。
- [x] 新增 `scripts/generate-app-store-screenshots.mjs`，使用真实 `mobile/shared/preview.html`、设备 CSS viewport + DPR、真实 Mermaid/KaTeX DOM 生成 App Store 截图。
- [x] 重新生成并上传 App Store 截图到 localization `dde3155e-18ac-46b5-a814-d3db22746d35`，替换旧截图。
- [x] App Privacy 网页问卷选择“不收集数据”，并发布隐私答复。
- [x] App Review 登录信息改为“不需要登录”，并补充 reviewer notes。
- [x] App Review 联系人信息已按用户提供内容写入 App Store Connect，不在仓库记录明文。
- [x] 提交 App Store 审核，submission id `5f1e7fbe-9f52-46c1-bd7b-011d38395301`。
- [x] App Store version copyright 已从公司名改为 `2026 Vorojar`，审核状态保持 `WAITING_FOR_REVIEW`。

## 验证记录

```text
命令：xcrun devicectl list devices && xcrun xctrace list devices
结果：未通过真机可用性。电脑能看到多台 iPhone，但 Xcode/CoreDevice 标记为 `unavailable` / `Devices Offline`。

命令：security find-identity -v -p codesigning
结果：未找到 iOS Apple Development / Apple Distribution 证书；当前只有 macOS Developer ID Application。

命令：asc auth status
结果：通过。新 ASC API key 已存入 System Keychain，并设为默认 ASC profile。

命令：asc bundle-ids create --identifier app.mdpreview.mobile --name "MD Preview Mobile" --platform IOS
结果：通过。创建 bundle id `6P439S39PG`，identifier `app.mdpreview.mobile`，seed/team `BUR55497B4`。

命令：cd mobile/ios && xcodegen generate && xcodebuild -project MDPreviewMobile.xcodeproj -scheme MDPreviewMobile -destination 'platform=iOS Simulator,name=iPhone 17 Pro,OS=26.5' CODE_SIGNING_ALLOWED=NO build
结果：通过。

命令：xcrun simctl install 2D8BDB83-D40C-4E0A-85FF-219A7427ECA4 <MDPreviewMobile.app> && xcrun simctl launch 2D8BDB83-D40C-4E0A-85FF-219A7427ECA4 app.mdpreview.mobile
结果：通过。模拟器安装成功，启动进程 pid 97643。

命令：xcrun simctl io 2D8BDB83-D40C-4E0A-85FF-219A7427ECA4 screenshot target/ios-qa/ios-simulator-launch.png
结果：通过。截图显示 MD Preview 空状态首屏。

命令：NODE_PATH=/Users/longjiewu/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules /Users/longjiewu/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node mobile/scripts/verify-mobile-renderer.mjs
结果：通过。

命令：mobile/scripts/verify-release-readiness.sh
结果：通过。

命令：cd mobile/ios && xcodebuild clean archive -project MDPreviewMobile.xcodeproj -scheme MDPreviewMobile -configuration Release -destination 'generic/platform=iOS' -archivePath "$PWD/build/MDPreviewMobile.xcarchive" CODE_SIGNING_ALLOWED=NO
结果：通过。无签名 archive 生成成功；archive Info.plist 确认为 version `1.0.7` build `8`。

命令：cd mobile/ios && xcodebuild -project MDPreviewMobile.xcodeproj -scheme MDPreviewMobile -destination 'id=<CONNECTED_IPHONE_UDID>' -allowProvisioningUpdates -allowProvisioningDeviceRegistration -authenticationKeyPath <ASC_KEY_PATH> -authenticationKeyID <ASC_KEY_ID> -authenticationKeyIssuerID <ASC_ISSUER_ID> build
结果：通过。真机构建成功，签名身份为 Apple Development API-created certificate，provisioning profile 为 `iOS Team Provisioning Profile: *`。

命令：xcrun devicectl device install app --device <CONNECTED_IPHONE_UDID> <MDPreviewMobile.app> && xcrun devicectl device process launch --device <CONNECTED_IPHONE_UDID> app.mdpreview.mobile
结果：通过。真机安装并启动成功。

命令：xcodebuild clean archive ... && xcodebuild -exportArchive ... -exportOptionsPlist target/ios-qa/ExportOptions-app-store-connect.plist
结果：通过。导出 `mobile/ios/build/export/MD Preview.ipa`。IPA 内 app 签名为 `Apple Distribution: Ningbo Huli Huli Network Technology Co., Ltd. (BUR55497B4)`，profile 为 `iOS Team Store Provisioning Profile: app.mdpreview.mobile`。

命令：./scripts/verify.sh
结果：通过。覆盖 release signing contract、cargo test、anchor navigation、Sparkle update、Windows self-update、iOS xcodegen/build/parse、Android debug/release、mobile renderer、release readiness。

命令：asc apps list --bundle-id app.mdpreview.mobile --output json
结果：通过。返回 app id `6779451523`，名称 `Local Markdown Preview`，SKU `md-preview-ios`，primary locale `en-US`。

命令：asc testflight groups list --app 6779451523 --output json
结果：通过。返回内部测试组 `Internal Testers`，group id `061ed3ee-dd2a-4449-87f8-3967c065ba1e`。

命令：asc publish testflight --app 6779451523 --ipa "mobile/ios/build/export/MD Preview.ipa" --group "061ed3ee-dd2a-4449-87f8-3967c065ba1e" --test-notes "Initial iOS TestFlight build for Markdown preview file-open validation." --locale en-US --wait --timeout 30m --output json
结果：通过。上传 build id `eadcd636-878e-40a4-95ee-f9ce93b86133`，版本 `1.0.7` build `8`，processingState `VALID`，已关联内部测试组。

命令：asc builds latest --app 6779451523 --version 1.0.7 --platform IOS --output json
结果：通过。最新 build id `eadcd636-878e-40a4-95ee-f9ce93b86133`，processingState `VALID`，usesNonExemptEncryption `false`。

命令：asc versions update --version-id e1e365a2-150c-4348-9226-7f5c13ed8b66 --version 1.0.7 --copyright "2026 Ningbo Huli Huli Network Technology Co., Ltd." --release-type AFTER_APPROVAL
结果：通过。App Store version 更新为 `1.0.7`。

命令：asc versions attach-build --version-id e1e365a2-150c-4348-9226-7f5c13ed8b66 --build eadcd636-878e-40a4-95ee-f9ce93b86133
结果：通过。App Store version 已绑定 TestFlight build。

命令：asc apps info edit --app 6779451523 --version 1.0.7 --platform IOS --locale en-US ...
结果：通过。写入 description、keywords、promotional text、support URL 和 marketing URL。首发版本的 `whatsNew` 被 Apple 拒绝编辑，保留为空。

命令：asc app-setup info set --app 6779451523 ... && asc categories set ... && asc age-rating edit --app 6779451523 --all-none
结果：通过。写入 app-level metadata、内容权利、分类和年龄分级。

命令：asc app-setup pricing set --app 6779451523 --free --start-date 2026-06-11
结果：通过。免费价格生效。

命令：asc screenshots validate --path target/app-store-screenshots/iphone --device-type APP_IPHONE_65 && asc screenshots validate --path target/app-store-screenshots/ipad --device-type APP_IPAD_PRO_3GEN_129
结果：通过。iPhone 三张 `1284x2778`，iPad 三张 `2048x2732`，均符合 ASC 尺寸要求。

命令：asc screenshots upload --version-localization dde3155e-18ac-46b5-a814-d3db22746d35 --path target/app-store-screenshots/{iphone,ipad} ...
结果：通过。iPhone set `22e09205-308d-41f8-8cb9-c87030f39ef9`、iPad set `e380c0b0-559d-4135-89e3-e9ae9600ed14`，6 张截图状态均为 `COMPLETE`。

命令：NODE_PATH="$TMP_PLAYWRIGHT_DIR/node_modules" node scripts/generate-app-store-screenshots.mjs
结果：通过。重新生成 iPhone `1284x2778` 与 iPad `2048x2732` 截图各 3 张：`01-mermaid-katex-notes.png`、`02-search-in-document.png`、`03-readme-rendering.png`。生成时强制等待 Mermaid SVG、KaTeX、搜索高亮、alert/highlight/table/code DOM 出现。

命令：asc screenshots validate --path target/app-store-screenshots/iphone --device-type APP_IPHONE_65 && asc screenshots validate --path target/app-store-screenshots/ipad --device-type APP_IPAD_PRO_3GEN_129
结果：通过。6 张新截图均符合 Apple 尺寸要求。

命令：asc screenshots upload --version-localization dde3155e-18ac-46b5-a814-d3db22746d35 --path target/app-store-screenshots/{iphone,ipad} --replace
结果：通过。旧截图被替换；ASC 当前列表只保留 `01-mermaid-katex-notes.png`、`02-search-in-document.png`、`03-readme-rendering.png`，iPhone/iPad 共 6 张均为 `COMPLETE`。

命令：App Store Connect 网页 / Pricing and Availability
结果：通过。初始化所有 175 个国家/地区发布时供应。

命令：App Store Connect 网页 / App Privacy
结果：通过。隐私政策 URL 已显示；问卷选择“不收集数据”；隐私答复已发布，页面显示“未收集数据”。

命令：asc validate --app 6779451523 --version-id e1e365a2-150c-4348-9226-7f5c13ed8b66 --platform IOS --output json
结果：未完全通过。剩余 4 个 blocking errors 均为 App Review 联系人缺失：`contactFirstName`、`contactLastName`、`contactEmail`、`contactPhone`。另外 `whatsNew` 是首发版本不可编辑警告，App Privacy 是公共 API 无法验证的 info，但网页已确认发布。

命令：asc review details-update --id 0c166707-b198-4800-afb4-800c90fd9e8b ...
结果：通过。App Review 联系人字段和 reviewer notes 已配置；联系人明文不写入仓库。

命令：asc validate --app 6779451523 --version-id e1e365a2-150c-4348-9226-7f5c13ed8b66 --platform IOS --output json
结果：通过提交前检查。`errors: 0`、`blocking: 0`；仅剩首发 `whatsNew` 空的 warning，以及 App Privacy 公共 API 不可验证的 info。

命令：asc review doctor --app 6779451523 --version-id e1e365a2-150c-4348-9226-7f5c13ed8b66 --platform IOS --output json
结果：通过。无 submission blockers，next action 为提交版本。

命令：asc review submit --app 6779451523 --version-id e1e365a2-150c-4348-9226-7f5c13ed8b66 --build eadcd636-878e-40a4-95ee-f9ce93b86133 --platform IOS --confirm --output json
结果：通过。submission id `5f1e7fbe-9f52-46c1-bd7b-011d38395301`，submittedDate `2026-06-12T02:30:49.757Z`。

命令：asc review status --app 6779451523 --version-id e1e365a2-150c-4348-9226-7f5c13ed8b66 --platform IOS --output json
结果：通过。版本状态 `WAITING_FOR_REVIEW`，next action 为等待 App Store review outcome。

命令：asc versions update --version-id e1e365a2-150c-4348-9226-7f5c13ed8b66 --copyright "2026 Vorojar"
结果：通过。Apple 接受更新，版本状态仍为 `WAITING_FOR_REVIEW`。

命令：asc versions list --app 6779451523 --platform IOS --version 1.0.7 --output json
结果：通过。确认 `copyright` 为 `2026 Vorojar`，`appStoreState` 为 `WAITING_FOR_REVIEW`。
```

## 风险和假设

- App Store Connect app record 已创建，但 `MD Preview`、`Markdown Preview` 和 `Markdown Previewer` 均被 Apple 判定名称占用；当前公开名称使用 `Local Markdown Preview`。
- TestFlight build 已上传并处理为 `VALID`；App Store metadata、截图、隐私、价格和销售范围已补齐。
- 第一版截图的根因是把 App Store 目标像素尺寸当作浏览器 CSS viewport 截图，导致 iPad 画布比例失真、内容像窄屏手机稿；已改为真实设备 CSS viewport + DPR，并保留脚本防止回退。
- App Store 审核已提交，当前等待 Apple Review 结果。
- 真机已能安装启动 app，但 Open In / 分享面板真实文件流仍建议用户在手机上用 Files、微信、企业微信各测一次。

---

# 上一任务存档：发布签名链路修复

## 目标

- 让 MD Preview 发布脚本默认使用 remote-mac-sign 的本地优先入口，而不是直接调用远程签名机。
- 对 Apple notary 的瞬态失败增加一次自动重试，减少手动恢复发布的概率。
- 把签名入口约束加入统一验证，避免以后回退成远程优先。

## 非目标

- 不重新发布 `v1.1.19`。
- 不修改全局 remote-mac-sign skill，不影响其他项目的签名行为。
- 不改变 GitHub Release、Sparkle appcast 或 Homebrew Cask 的发布格式。

## 验收场景

- [x] 默认签名脚本为 `$HOME/.claude/skills/remote-mac-sign/sign.sh`，由 skill 决定本地可用时本地签、本地不可用时兜底远程。
- [x] 可以用 `MD_PREVIEW_SIGN_SCRIPT` 覆盖签名入口，用 `MD_PREVIEW_SIGN_ATTEMPTS` 控制重试次数。
- [x] `MD_PREVIEW_SIGN_ATTEMPTS=0 ./release-sign.sh v1.1.19` 在联网/下载 Release 前失败，提示必须是正整数。
- [x] `scripts/verify.sh` 会检查 `release-sign.sh` 不再硬编码 `sign_remote.sh`。

## 执行记录

- [x] `release-sign.sh` 默认签名入口从 `sign_remote.sh` 改为本地优先的 `sign.sh`。
- [x] `release-sign.sh` 增加签名重试循环，默认最多尝试 2 次，每次重试前清理本次工作目录里的旧 `signed-output`。
- [x] `scripts/release.sh --help` 补充签名脚本和签名重试环境变量。
- [x] `scripts/verify.sh` 增加 release signing contract 检查和 shell 语法检查。

## 验证记录

```text
命令：bash -n release-sign.sh scripts/release.sh scripts/verify.sh
结果：通过。

命令：MD_PREVIEW_SIGN_ATTEMPTS=0 ./release-sign.sh v1.1.19
结果：通过。脚本在联网/下载 Release 前以 exit 2 失败，并提示 `MD_PREVIEW_SIGN_ATTEMPTS must be a positive integer`。

命令：./scripts/verify.sh
结果：通过。覆盖 release signing contract、cargo test、anchor navigation、Sparkle update、Windows self-update、iOS xcodegen/build/parse、Android debug/release、mobile renderer、release readiness。
```

## 风险和假设

- 本次不触发真实发布和真实签名；真实 Apple notary 仍可能卡在 Apple 服务侧，但发布脚本会自动重试一次，并且默认会优先走本机签名。
- `sign.sh` 的本地可用性判断仍由 remote-mac-sign skill 负责：本机证书或 notary profile 缺失时会按 skill 设计自动兜底远程。

---

# 上一任务存档：v1.1.19 issues/release

## 目标

- 解决 GitHub issues #3、#20、#23，并完成 #19 的 Homebrew Cask 发布路径。
- 发布新的桌面版本 `v1.1.19`，推送到 GitHub，并验证 Release assets。
- 在验证通过后同步 issue 状态，避免已完成事项继续悬挂。

## 非目标

- 不重做 Markdown 引擎，不引入新的大型渲染依赖。
- 不调整主题选择器、CLI `--edit` / `--print` 等其他 open issues。
- 不改变移动端 App Store / Google Play 分发策略。

## 验收场景

- [x] `> [!IMPORTANT]` 渲染为 `markdown-alert-important`，并且 `[!IMPORTANT]` 标记本身不显示在正文里。
- [x] `==高亮 & tag==` 渲染为 `<mark class="mdp-mark">`，内部文本仍然安全转义。
- [x] inline code / fenced code 中的 `==literal==` 不会被误转成高亮。
- [x] Linux + NVIDIA 且用户没有手动设置 WebKit workaround 时，启动前自动设置 `WEBKIT_DISABLE_DMABUF_RENDERER=1`。
- [x] 用户已显式设置 `WEBKIT_DISABLE_DMABUF_RENDERER` 或 `WEBKIT_DISABLE_COMPOSITING_MODE` 时，程序不覆盖用户选择。
- [x] 移动端共享预览层同样支持 GitHub Alerts 和 `==highlight==`。
- [x] README / README_zh 记录新 Markdown 支持和 Linux NVIDIA 空白窗口 workaround。
- [x] 新版 `v1.1.19` GitHub Release 包含 macOS DMG、Windows EXE、Linux tarball、`appcast.xml`。
- [x] Homebrew Cask 使用新版 macOS DMG 的真实 sha256，并通过 `brew audit --cask`，PR 已提交到 Homebrew/homebrew-cask。

## 执行记录

- [x] 桌面 Markdown 解析启用 `Options::ENABLE_GFM`，使用 `pulldown-cmark` 原生 GitHub Alert 支持。
- [x] 桌面 Markdown event 层增加 `==highlight==` 到 `<mark class="mdp-mark">` 的转换。
- [x] 桌面和移动端补充 GitHub Alert / mark CSS。
- [x] Linux 启动阶段增加 NVIDIA/WebKitGTK DMABUF fallback。
- [x] 移动端共享增强脚本增加 GitHub Alert 和 mark DOM 增强。
- [x] 移动端 Playwright renderer fixture 覆盖 alert、mark、code literal。
- [x] 版本号更新到 `1.1.19`，CHANGELOG / README / README_zh 更新。
- [x] `v1.1.19` tag、GitHub Release、签名 DMG、Sparkle appcast 完成。
- [x] Homebrew Cask PR 创建：https://github.com/Homebrew/homebrew-cask/pull/269252

## 验证记录

```text
命令：cargo test -- --nocapture
结果：通过。17 个 Rust 单测全部通过，覆盖 GitHub Alerts、mark 渲染、code 不误伤、Linux NVIDIA fallback、既有锚点/搜索/更新逻辑。

命令：NODE_PATH=/Users/longjiewu/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/node_modules /Users/longjiewu/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node mobile/scripts/verify-mobile-renderer.mjs
结果：通过。移动端 fixture 渲染 KaTeX、Mermaid、GitHub Alert、mark、搜索、打印样式和 javascript: 链接拦截。

命令：./scripts/verify.sh
结果：通过。guard、cargo test、anchor navigation、Sparkle update、Windows self-update、iOS xcodegen/build/parse、Android debug/release、mobile renderer、release readiness 全部通过。

命令：cargo build --release
结果：通过。确认非 Linux release build 不再出现 `linux_webkit_compat_env` dead_code warning。

命令：scripts/release.sh v1.1.19
结果：GitHub Actions 三平台 release build 和 GitHub Release 创建通过；第一次远程签名阶段 Apple notary `NSURLErrorDomain Code=-1001` 超时。

命令：本地优先 remote-mac-sign 恢复签名并上传
结果：第一次本地 app.zip submission `94eb27f7-ac99-48fc-9c49-fc5ca17b04ec` 长时间停留 `In Progress`；清理临时挂载后第二次本地签名成功，inner app submission `391e4f1a-ae09-4484-90df-0b7b1a34882a` Accepted，DMG submission `7f990dd7-c963-402d-9b17-56ce7f42fd08` Accepted，DMG 和内层 app 均完成 staple。

命令：gh release view v1.1.19 -R vorojar/md-preview --json assets
结果：通过。Release asset 为 `appcast.xml`、`MD-Preview-linux-x64.tar.gz`、`MD-Preview-macOS-universal.dmg`、`MD-Preview-windows-x64.exe`。

命令：xcrun stapler validate target/MD-Preview-macOS-universal.dmg
结果：通过。The validate action worked。

命令：codesign --verify --deep --strict --verbose=2 target/MD\ Preview.app
结果：通过。app valid on disk，satisfies Designated Requirement。

命令：spctl -a -t open --context context:primary-signature target/MD-Preview-macOS-universal.dmg
结果：通过。

命令：curl -fsSL https://github.com/vorojar/md-preview/releases/download/v1.1.19/appcast.xml
结果：通过。appcast 指向 `v1.1.19/MD-Preview-macOS-universal.dmg`，并包含 `sparkle:edSignature`。

命令：brew audit --cask --new md-preview
结果：通过。

命令：brew style --cask md-preview
结果：通过。1 file inspected, no offenses detected。

命令：brew livecheck --cask md-preview --json
结果：通过。current/latest 均为 `1.1.19`。

命令：brew install --cask --appdir=$(mktemp -d) ./Casks/m/md-preview.rb && brew uninstall --cask md-preview
结果：通过。`MD Preview.app` 安装到临时 appdir，`/opt/homebrew/bin/md-preview` 正常 link/unlink。
```

## 风险和假设

- 本机不是 Linux/NVIDIA 环境，#3 的自动 fallback 通过确定性单元测试和文档验证；真实 GPU/WebKitGTK 渲染仍需用户环境回归。
- Homebrew Cask 已提交 PR，最终是否 merge 取决于 Homebrew 维护者审核。
- Apple notary 第一次远程提交和第一次本地 app.zip 等待都出现服务侧超时/长时间 In Progress；最终通过第二次本地提交完成签名、公证和 staple。
