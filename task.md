# 当前任务：桌面多标签、Finder 新建闭环与 v1.2.0 发布

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
- [ ] 最终 DMG 与内部 app/appex 通过签名、公证、staple、`spctl`；GitHub Release 含三平台资产、appcast 和来自本版本 CHANGELOG 的完整说明。

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
- 待完成：干净副本正式发布、CI、Developer ID 签名、公证/staple、Gatekeeper 与 GitHub Release 资产/说明复验。
