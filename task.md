# 当前任务：新建 Markdown、可靠自动保存与 v1.3.0 发布

## 目标

- 标签栏 `+` 不再重复“打开文件”，改为新建 Markdown；选择保存位置后创建新标签并直接进入编辑。
- 编辑内容停止输入后自动保存；预览、切换标签、关闭标签、关闭窗口和退出应用前都强制刷新待保存内容。
- 保存失败时不关闭、不切换、不丢编辑内容，并给出明确错误。
- 检查应用内 GitHub Release 查询、平台资产选择、Sparkle appcast 与签名更新入口，确保 v1.3.0 发布后可被旧版本发现。
- 完成单测、完整验证、真实 macOS UI、签名、公证、Gatekeeper、Release assets 与应用内更新闭环后发布 v1.3.0。

## 非目标

- 不实现内存态“未命名文档”、草稿云同步、富文本编辑或文件重命名。
- 不改变 Finder 右键 Text / JSON / HTML 的行为，也不改移动端单文档预览流程。
- 不提交 App Store Review、推广资料、AGENTS.md 及其他既有未提交内容。

## 验收场景

- [x] `+` 与 `Cmd/Ctrl+N` 打开新建 Markdown 保存面板；默认目录跟随当前文档，确认后创建 `.md` 标签并聚焦编辑器；取消时不创建文件或标签。
- [x] 文件夹按钮与 `Cmd/Ctrl+O` 仍只打开已有文件，和 `+` 语义不重复。
- [x] 输入停止约 700ms 后内容自动落盘，标签 dirty 状态恢复为已保存；`Cmd/Ctrl+S` 仍立即保存。
- [x] 未点击“预览”时，切换标签、标签 `×`、`Cmd/Ctrl+W`、窗口关闭与 macOS `Cmd+Q` 都保存成功；重开文件内容存在。
- [x] 保存失败会阻止切换或关闭，编辑器内容和 dirty 状态保留，用户可修复后重试。
- [x] 外部文件变化不会被待保存编辑静默覆盖；自写 watcher 事件不会破坏光标或正文。
- [x] GitHub Release 查询忽略 mobile、draft 与 prerelease，能选择正确平台资产；最新版显示“已是最新”，新版本走受信 URL 与原生更新入口。
- [x] `cargo test`、`cargo check`、`./scripts/verify.sh` 与真实 macOS UI 全部通过；三平台 CI 等待 PR。
- [ ] macOS DMG 及内部 app/appex 完成 Developer ID 签名、公证、staple、Gatekeeper 与 Sparkle appcast 验证；Release 正文来自 CHANGELOG。

## 最小验证命令

```bash
cargo test
cargo check
./scripts/verify.sh
./bundle.sh
codesign --verify --deep --strict --verbose=2 "target/MD Preview.app"
```

## 风险与假设

- MD Preview 当前所有标签都是文件背书；新建流程使用保存面板先确定路径，避免引入无法恢复的内存草稿。
- 自动保存会增加写盘频率，必须去抖并继续抑制自写 watcher；外部修改与本地 dirty 冲突时以不覆盖用户编辑为优先。
- 正常退出保存需要覆盖 macOS `Cmd+Q` 真实路径，不能只依据窗口关闭代码推断。

## 当前验证记录

- TDD：新建扩展名、关闭最后编辑标签、外部变化决策与自写事件内容核对均经历失败测试后修复，当前相关测试已通过。
- 真实 UI：`+`/`Cmd+N` 新建并进入编辑、取消不建文件、700ms 自动保存、`Cmd+W`、红色窗口关闭与 `Cmd+Q` 写盘均通过。
- 失败恢复：只读文件显示 `Permission denied`，阻止关闭并保留内容；恢复权限后 `Cmd+S` 写盘通过。
- 更新检查：`verify-sparkle-update.sh` 通过；线上最新桌面版为 `v1.2.0`，平台资产与 `appcast.xml` 完整；应用菜单显示 `MD Preview Is Up to Date / 1.2.0`。
- 完整验证：`scripts/verify.sh` 通过，包含 33 个 Rust 测试、桌面搜索/锚点、Sparkle、Windows updater、iOS build、Android debug/release、移动渲染与 release readiness。

---

# 已完成任务：v1.2.0 官网、README、About 与社区补发

## 补发目标

- 官网和中英文 README 直接表达 v1.2.0 解决的三个核心痛点：多文档标签、会话恢复、Finder 新建后直接编辑。
- 官网使用真实 v1.2.0 多标签截图，更新 SEO、Open Graph 与结构化版本信息。
- GitHub About 和应用 About 源码使用一致、克制的产品定位；本次不重新打包二进制。
- 回复 GitHub Issue #28 并按已完成关闭；回复 #33，明确相对 Markdown 链接仍未实现并保持开放。

## 补发非目标

- 不改变 v1.2.0 功能实现，不新增相对链接、文件夹树或缩放功能。
- 不重新签名、公证或发布安装包。
- 不提交 App Store Review、推广资料及其他既有未提交改动。

## 补发验收

- [x] 官网首屏、功能区、安装说明及元数据均包含 v1.2.0 新能力，桌面和移动视口无溢出或遮挡。
- [x] README.md 与 README_zh.md 同步描述多标签、恢复、缺失文件、Finder 操作和快捷键。
- [x] 应用 About 源码包含新定位与 What's New 入口；Rust 测试和完整验证通过。
- [x] GitHub About 更新；官网线上内容与提交一致。
- [x] #28 有发布说明并关闭；#33 有明确状态回复并保持开放。

## 补发验证记录

- 官网 HTML、JSON-LD 与中英文属性解析通过；结构化版本为 `1.2.0`，新版主图为真实多标签界面。
- 真实浏览器验收：桌面 1280×720 与移动 390×844 均无横向溢出；中英文切换、首屏、功能区和安装说明正确。
- 移动验收发现语言切换点击区不足 44px，已修复为最小 44×44px；主要下载按钮最小高度约 52px。
- 应用 About 真实运行：新版定位完整显示，Home、GitHub、What's New 三个入口存在，无文字裁切。
- `cargo fmt --check`、`cargo check`、`cargo test`（28/28）及 `./scripts/verify.sh` 全部通过。
- GitHub About 已更新为多标签、会话恢复、macOS Finder 工作流与本地优先定位。
- Issue #28 已回复并按 completed 关闭；#33 已回复并保持开放。
- PR #34 在 macOS、Linux、Windows CI 全绿后合并；GitHub Pages 构建提交为 `5f2b9dd`。
- 线上 `index.html` 与 `hero.jpg` 分别和仓库文件哈希完全一致；线上桌面与移动视口复验通过。
- 官网主图进一步换成真实发布版渲染：同屏展示 3 个标签、GitHub Alert、Mermaid、KaTeX 与 Rust 代码高亮；拒绝使用会重绘产品像素的生成式图片作为事实截图。

---

# 已完成任务：桌面多标签、Finder 新建闭环与 v1.2.0 发布

## 目标

- 桌面端使用顶部标签同时打开多份 Markdown / Text 文档；重复打开同一路径时激活已有标签。
- 持久化标签顺序和活动标签；重启只加载活动文档内容，其他标签在点击时从磁盘懒加载。
- 文件被移动或删除时保留标签并显示明确的缺失状态，支持重新定位或关闭标签。
- 每个标签独立管理 dirty 状态；切换、关闭和外部修改不得串到其他文档。
- macOS 将 Finder Sync 右键工具嵌入 `MD Preview.app`；新建 Markdown 后直接在新标签进入源码编辑。
- 完成自动化、真实 UI、安装包、签名、公证、Gatekeeper 和 GitHub Release 验收后发布 `v1.2.0`。

## 非目标

- 本次不把多标签扩展到 iOS / Android；移动端继续保持单文档快速预览。
- 不增加左侧文件树、项目管理、富文本编辑或云同步。
- Finder 右键中的 Text / JSON / HTML 创建动作保持现有行为；只有 Markdown 直达 MD Preview 编辑闭环。
- 不修改或提交当前工作区已有的 App Store Review 文档、推广资料及其他无关改动。

## 验收场景

- [x] 打开、拖入或由 Finder 传入多份支持文件时创建顶部标签；一次拖入多份文件全部加入，同一路径不重复。
- [x] 标签切换显示对应内容和窗口标题；关闭当前标签后选择相邻标签，关闭最后一个标签回到空状态。
- [x] `Cmd/Ctrl+W` 关闭当前标签；没有标签时才关闭窗口。
- [x] 退出后重启恢复标签顺序和活动标签；会话文件只记录路径/顺序/活动项，不缓存未修改文档正文。
- [x] 启动时缺失文件不会被静默删除或连续弹窗；标签显示警告，点击后可重新定位或关闭。
- [x] 活动文件运行中被删除时显示缺失状态；后台标签点击时读取磁盘最新内容。
- [x] 编辑 dirty 状态按标签隔离；切换标签会保存当前编辑，保存失败时保留当前标签和未保存内容并提示。
- [x] Finder 右键“新建文件 > Markdown (.md)”创建不冲突文件名，启动/唤醒 MD Preview，在新标签进入编辑并聚焦。
- [x] Finder 扩展打入 app bundle，使用稳定 bundle id，可被 `pluginkit` 枚举；主应用正常预览不依赖扩展启用。
- [x] `cargo test`、`cargo check`、`./scripts/verify.sh` 和 macOS universal Release 构建全部通过。
- [x] 真实 macOS UI 验证顶部标签、溢出、暗色/亮色、缺失状态、编辑切换与窗口关键尺寸，无文字遮挡。
- [x] 最终 DMG 与内部 app/appex 通过签名、公证、staple、`spctl`；GitHub Release 含三平台资产、appcast 和来自本版本 CHANGELOG 的完整说明。

## 最小验证命令

```bash
cargo test
cargo check
./scripts/verify.sh
./bundle.sh
codesign --verify --deep --strict --verbose=2 "target/MD Preview.app"
pluginkit -m -A -p com.apple.FinderSync | grep MDPreviewFinder
```

## 风险与假设

- Finder Sync 扩展首次启用受 macOS 用户授权控制；应用可以注册、检测和引导，但不能绕过系统策略静默替用户授权。
- 当前代码是单文件、单 watcher、单 dirty 状态；必须先建立文档会话模型，不能只增加视觉标签。
- 发布从本次提交的干净副本执行，以保留当前工作区已有未提交内容。

## 执行记录

- [x] 已确认最新正式版为 `v1.1.25`，本次使用语义化次版本 `v1.2.0`。
- [x] 已检查现有单文件状态、最近文件持久化、文件 watcher、内置源码编辑和 macOS 发布链路。
- [x] 已读取本地优先签名、公证和 GitHub 发布流程。
- [x] 已实现会话模型、顶部标签、懒加载、缺失文件状态、逐标签 dirty 与切换/关闭前保存。
- [x] 已将 Finder Sync 扩展打入 `MD Preview.app`，并完成 Finder 右键创建 Markdown → 新标签源码编辑 → 保存的真实闭环。
- [x] 暗色视觉验收发现 Markdown Alert 对比度回归，已修复 CSS 层叠顺序并加入回归断言。

## 验证记录

- `cargo check`：通过。
- `cargo test`：28/28 通过；覆盖路径去重、相邻标签关闭、会话恢复/缺失文件、Finder URL 与不冲突文件名。
- `./scripts/verify.sh`：通过；桌面搜索、锚点、Sparkle、Windows 更新、iOS build、Android debug/release 与移动渲染均通过。
- `./bundle.sh`：通过；macOS 主程序与 Finder extension 均为 `x86_64 arm64`，extension 版本 `1.2.0`。
- `codesign --verify --deep --strict`：通过；extension 保留 `com.apple.security.app-sandbox=true`。
- 隔离会话真实验收：重启后 3 个标签顺序不变、活动索引为 1，`session.json` 只保存路径与活动项，无正文。
- 真实编辑验收：标签显示 dirty，切换后写盘成功并激活目标文档；窗口标题与内容一致。
- 真实缺失验收：删除后台文件后点击标签，标签保留并显示“重新定位/关闭标签”。
- 真实视觉验收：亮色、暗色、约 560px 紧凑窗口、7 个含长文件名标签溢出均无重叠；暗色 Alert 修复后复验通过。
- Finder 系统验收：`pluginkit` 显示 `+ com.mdpreview.app.FinderExtension(1.2.0)`；Finder 空白处右键显示完整菜单；创建 `新建.md` 后应用进入聚焦源码编辑，`Cmd+S` 写盘内容核对通过。
- 干净副本正式发布：GitHub Actions `29553519458` 的 Linux、Windows、macOS 构建与 Release 发布全部成功。
- Apple 公证：内部 app submission `4af9070d-c93f-481c-ac5c-ce373caa52cd`、外层 DMG submission `9ddb3b73-225f-4df1-824e-35a9b4d9f20d` 均为 `Accepted`，两者 staple 成功。
- 最终产物验收：已安装 `/Applications/MD Preview.app` 为 `1.2.0`，主程序与 Finder extension 均为 `x86_64 arm64`；`codesign --verify --deep --strict`、DMG `stapler validate` 与 app/DMG `spctl` 全部通过，来源为 `Notarized Developer ID`。
- 已安装公证版真实运行：隔离会话启动两份文档，顶部标签、活动文档、正文与窗口标题正确，应用正常退出。
- Finder 扩展最终状态：`+ com.mdpreview.app.FinderExtension(1.2.0)`；旧独立扩展仅禁用保留，避免重复菜单。
- GitHub Release `v1.2.0` 已发布，包含 Linux、Windows、macOS DMG 与 `appcast.xml`；Release 正文与 `CHANGELOG.md` 的六条版本说明一致。
