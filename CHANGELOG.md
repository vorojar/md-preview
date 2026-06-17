# Changelog

## 1.1.23

- Fixed desktop print margins on macOS by adding an explicit `@page` margin to the rendered preview print stylesheet.
- Replaced desktop preview search's fragile `window.find()` dependency with deterministic in-page hit marking and navigation, restoring reliable `Cmd/Ctrl+F` search on macOS WebView.
- Added release verification coverage for desktop print margins and desktop search.

## 1.1.22

- Fixed `Cmd/Ctrl+F` in source edit mode by letting the WebView/editor native find action handle editing, while keeping the custom find bar for rendered previews.

## 1.1.19

- Added GitHub-style alert rendering for `[!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`, and `[!CAUTION]` blockquotes on desktop and mobile.
- Added lightweight `==highlight==` mark rendering while leaving code spans and code blocks unchanged.
- Added a Linux WebKitGTK/NVIDIA compatibility fallback that disables the DMABUF renderer when no WebKit workaround is already configured, plus documented the manual fallback commands.
- Added mobile renderer coverage for GitHub Alerts and highlight marks.

## 1.1.16

- Fixed in-document anchor clicks when a Markdown file has a local `<base href>` for relative assets. Table-of-contents links such as `[需求概述](#需求概述)` now scroll inside the preview instead of being treated as file navigation.
- Added a Playwright regression check for encoded Chinese anchor links with a `file://` base URL.

## 1.1.15

- Fixed in-document anchor links by automatically generating stable heading IDs, including Chinese headings such as `## 需求概述`, while preserving explicit `{#id}` heading attributes.
- Added a foreground maintainer release script that verifies the project, pushes the tag, waits for GitHub Actions, signs/notarizes/staples the macOS DMG, uploads the Sparkle appcast, and validates the final Release assets.

## 1.1.14

- Fixed in-preview search with Chinese IME input by deferring search while text composition is active and restoring focus after native find updates the selection.
- Made macOS in-app updates use Sparkle only when the app is installed in `/Applications` or `~/Applications`; launches from DMG, Downloads, or other transient locations now fall back to the GitHub download page instead of failing during install.

## 1.1.13

- Improved wide Markdown tables: multi-column tables now expand beyond the reading column on desktop and scroll horizontally when needed, avoiding cramped vertical headers and over-wrapped cells.

## 1.1.12

- Replaced the Windows installer/WinSparkle update path with a single-file self-updater.
- Windows releases now publish `MD-Preview-windows-x64.exe` directly; the app downloads the next exe, verifies GitHub's SHA-256 digest, exits, replaces itself with a temporary PowerShell script, and relaunches.
- Removed the Windows runtime dependency on `WinSparkle.dll` and the NSIS setup package from the release flow.

## 1.1.11

- Added native Windows self-update support with WinSparkle, including EdDSA-signed `appcast-windows.xml`.
- Added a per-user Windows installer (`MD-Preview-windows-x64-Setup.exe`) that installs `WinSparkle.dll`, Start Menu shortcuts, uninstall metadata, and Markdown open-with registration.
- Kept the portable Windows ZIP for manual use, now bundled with `WinSparkle.dll`.

## 1.1.10

- Added native macOS self-update support with Sparkle, including EdDSA update signing and a generated `appcast.xml` release asset.
- Updated the desktop update button so macOS uses Sparkle's in-app update flow while other desktop platforms still fall back to the GitHub download.
- Filtered update checks to desktop `vX.Y.Z` releases so Android mobile releases cannot interfere with desktop update detection.

## 1.1.9

- Made stale Recent entries safe on desktop by removing missing files from the start screen instead of trying to open them.
- Updated Android Recent handling so opened Markdown files are cached inside the app and stale entries are removed without crashing.
- Added mobile app information to the README files and updated release links.

## 1.1.8

- Removed the Recent heading underline while keeping recent file entries visually unchanged.
- Softened the Recent label on the desktop empty state.

## 1.1.7

- Hid Open, Search, and Print while editing, leaving only the preview toggle.
- Prevented edit mode from scrolling to the end of long documents.

## 1.1.6

- Restored the desktop empty-state visual style while keeping the new Open File and Recent entries.
- Matched the desktop toolbar Open icon to the mobile app's folder icon.

## 1.1.5

- Added a cleaner desktop start screen with a primary Open File action and local recent files.
- Added toolbar Open and Find actions, including `Cmd/Ctrl+F` in the rendered preview.
- Updated release links and docs for the new desktop workflow.

## 1.1.4

- Fixed hot reload after Vim/Neovim saves by watching the parent directory and filtering events for the active Markdown file, so atomic rewrite/rename saves continue to refresh correctly.

## 1.1.3

- Added `-h` / `--help` so CLI help no longer gets treated as a Markdown file path.
- Disabled the preview area's native WebView context menu and intercept `Cmd/Ctrl+R` to refresh from disk, preventing browser-style reload from blanking the in-memory app shell.

## 1.1.2

- 修复 KaTeX 公式与 Markdown 转义/强调语法冲突的问题：`$\{x\}$`、`$\bar{\mu}_{n}$` 这类公式现在会在 Markdown 解析阶段被保护，再交给 KaTeX 渲染，避免反斜杠和 `_` 被 Markdown 先处理

## 1.1.1

- 新增 GitHub Releases 更新检测：首屏渲染后异步检查最新正式版，发现新版本时在右上角工具栏显示更新按钮，点击打开 GitHub Release 页面
- 更新检测结果缓存 24 小时，网络失败、GitHub 限流或离线状态静默忽略，不阻塞启动和 Markdown 首屏渲染

## 1.1.0

- 新增 Mermaid 图表渲染：支持 ```` ```mermaid ```` fenced code block，普通 Markdown 首屏不加载 Mermaid，只有检测到 Mermaid 代码块时才在首屏 paint 后注入本地运行时并异步渲染
- 新增 KaTeX 数学公式渲染：支持 `$...$`、`$$...$$`、`\(...\)`、`\[...\]`，KaTeX JS/CSS/woff2 字体全部离线内置，且只在检测到公式语法时按需加载
- 保持冷启动轻路径：首屏 HTML 仍不包含 highlight.js、Mermaid 或 KaTeX runtime；无公式/图表的普通 Markdown 只走原有解析和首屏渲染路径

## 1.0.3

- 修复 macOS 编辑模式下 `Cmd+V` 可能无法粘贴的问题：补齐标准 App/Edit 菜单，让剪切、复制、粘贴、全选、撤销/重做经由系统响应链分发到当前编辑区

## 1.0.2

- 修复从文件打开 Markdown 时，相对路径图片在 Windows 等平台可能无法显示的问题：页面现在以 Markdown 文件所在目录作为 base URL
- LaTeX/数学公式渲染暂未纳入，本版保持轻量定位，不引入 KaTeX/MathJax 运行时

## 1.0.1

- 修复 Windows 长文档编辑时输入可能跳回顶部的问题：编辑区自动增高时保留页面滚动位置
- 修复 Linux Wayland 下 WebView 创建可能触发 `UnsupportedWindowHandle` 的问题：Linux 改走 wry 推荐的 GTK container 路径

## 1.0.0

**首次正式版本**。以 0.3.x 系列的所有功能为基础，本版专注冷启动性能。

### 冷启动提速 24%

首屏 HTML 大幅瘦身：121KB 的 highlight.js 不再嵌入首屏 `<script>` 标签，从 HTML 解析路径移除；改为页面首次 paint 后由 Rust 通过 `evaluate_script` 推入。Markdown 内容是首屏唯一 JS 体积。

同 630 行 Markdown 测试文件（70 个 Python 代码块），macOS Apple Silicon 上测 5 次均值：

| 阶段 | 0.3.12 基线 | 1.0.0 | 降幅 |
|------|-------------|-------|------|
| main → window 创建 | 130 ms | 92 ms | -29% |
| → webview 实例化 | 230 ms | 172 ms | -25% |
| → **首帧可见** | **448 ms** | **340 ms** | **-24%（快 108 ms）** |

用户感知：窗口弹出即可见 Markdown 内容；代码着色在首帧之后约 50ms 补上（肉眼几乎感觉不到）。

### 附带改进

- 新增 `MD_PREVIEW_BENCH=1` 环境变量：启用后 stderr 输出冷启动各阶段耗时，并在首帧 paint 后自动退出，便于可复现地度量启动性能
- 版本号对齐为 1.0.0（Cargo.toml 和 macOS `CFBundleVersion` / `CFBundleShortVersionString` 同步，此前一直停在 0.1.0）

### 自 0.3.12 以来的所有功能（汇总）

- macOS / Windows / Linux 原生 WebView，无 Electron，二进制 ~1MB
- Markdown 渲染（pulldown-cmark，GFM：表格、任务列表、删除线、标题锚点）
- 40+ 语言代码高亮（含 Pascal / Delphi），延迟加载不阻塞首屏
- 暗色 / 亮色模式跟随系统
- 内置源码编辑（`Cmd/Ctrl+E` 切换预览/编辑，`Cmd/Ctrl+S` 保存，工具栏按钮 hover 时浮现）
- 原生打印对话框（`Cmd/Ctrl+P`，跨平台一致）
- md 外链跳系统默认浏览器
- Windows 文件关联（右键"打开方式"）
- 窗口位置/大小退出时持久化，下次启动恢复；首次启动在主显示器居中
- i18n（简体中文 / English 跟随系统语言）
- Windows 上 WebView2 缓存迁移到 `%LOCALAPPDATA%`，不再污染 exe 同目录
- macOS dmg 和内部 .app 均签名 + 公证 + staple，Gatekeeper 无任何警告
- `git push origin vX.Y.Z` 自动触发签名 + 公证 + 替换 Release asset（`hooks/pre-push` + `release-sign.sh`）

## 0.3.12

- 新增 Pascal / Delphi 代码高亮支持（hljs 的 "Delphi" 语法，别名含 `pascal` / `pas` / `dpr` / `dfm`）。highlight.js 官方 common bundle 只含 36 种主流语言，Delphi 不在其中；现在把 `delphi.min.js`（2.2KB）单独附加到 hljs 源码末尾，随 hljs 一起在 idle 时加载注册，不阻塞首屏
- 为以后追加其它语言留了 `HLJS_EXTRA_LANGS` 合并点，按需新增 `.min.js` 子包即可

## 0.3.11

- 修复 v0.3.3 引入的语法高亮失效：hljs bundle 的 export 模式是 `var hljs = IIFE()` + CommonJS，没有浏览器 UMD 降级。我们用 `new Function(src)()` 在 idle 时延迟执行，导致 `var hljs` 变成函数作用域局部变量，永远不会挂到 `window.hljs`，`hljs.highlightAll()` 因为 `typeof hljs !== 'undefined'` 守护直接跳过。现在在 eval 的源码后追加 `;window.hljs=hljs;` 显式暴露到全局，代码高亮恢复正常。

## 0.3.10

- 修复 macOS 下打印按钮无反应：WKWebView 不实现 `window.print()`，之前点击是 no-op。现在打印按钮和 `Cmd/Ctrl+P` 都走 IPC 交给 Rust，调用 wry 的 `WebView::print()` 走各平台原生打印对话框（macOS NSPrintOperation / Windows WebView2 ShowPrintUI / Linux WebKitGTK print operation）

## 0.3.9

- 修复编辑模式下的"双滚动条"：textarea 原生的内部滚动条与页面滚动条叠加。现在 textarea 关闭内部滚动（`overflow: hidden`），用 JS 让其高度跟随 `scrollHeight` 自增长，所有滚动交给页面 html，单一滚动条；副作用：编辑时滚轮再也不会"在 textarea 内滚到底后卡一下才传递给页面"

## 0.3.8

- 工具栏按钮在编辑/预览模式间的位置漂移修复：`html { overflow-y: scroll; scrollbar-gutter: stable; }` 让滚动条空间永久预留，两种状态下 viewport 宽度一致，`position: fixed; right` 的定位不再因滚动条出现/消失而偏移
- 编辑模式下隐藏打印按钮：打印属于"读完"场景，与编辑心智冲突；Cmd/Ctrl+P 系统快捷键不受影响

## 0.3.7

- 工具栏改为 hover 时浮现：鼠标离开窗口自动淡出，不干扰阅读
- 空态（未打开文件）不再显示工具栏，避免"没内容可编辑/打印"的按钮误导
- 编辑模式切换为全宽布局：textarea 撑满窗口，提供更开阔的写作空间；预览仍保持 820px 居中阅读；模式差异成为切换的视觉反馈

## 0.3.6

- 新增 Source 编辑模式：右上角浮动按钮切换预览/编辑，编辑模式下 textarea 装 md 原文，打字即生效；Cmd/Ctrl+E 也可切换
- Cmd/Ctrl+S 保存改动，watcher 500ms 自写抑制避免因保存触发无谓重渲染；脏态在标题栏以「• 」前缀显示
- 新增打印按钮：Cmd/Ctrl+P 或工具栏右侧按钮调起系统打印；打印样式自动隐藏工具栏与编辑区，只打印预览内容
- 按钮使用内联 SVG 图标（非 emoji），明暗模式适配；tooltip 展示快捷键
- 保存后仅更新预览区（不触碰 textarea 光标位置）；外部改动走完整 `__setContent` 同步两者

## 0.3.5

- Windows 首次启动不再弹窗询问是否跳转「设置 › 默认应用」；后台静默写入注册表后即结束，用户按需在 .md 文件右键选 MD Preview 即可

## 0.3.4

- 窗口首次启动居中显示（计算 primary monitor 的可见区域）
- 记住窗口位置/大小：关闭时写入 `<config>/window.geom`，下次启动恢复；跨屏幕断开场景下自动回退居中，避免窗口飘到不可见区
- 空态提示与 Windows 首次启动弹窗按系统语言切换（`zh*` → 中文，其它 → 英文，基于 `sys-locale`）
- Windows：WebView2 的 UserDataFolder 迁移到 `%LOCALAPPDATA%\md-preview\WebView2`，exe 同目录不再产生 `*.WebView2/` 缓存目录
- 统一 `config_dir()` 跨平台：Windows 使用 `%LOCALAPPDATA%`，macOS/Linux 使用 `~/.config/md-preview`

## 0.3.3

- 首屏渲染加速：highlight.js（119KB）从同步 inline 改为 `<script type="text/x-hljs">` 的惰性文本，首次 paint 不再为 hljs 解析/执行等待；在 `requestIdleCallback` 里异步求值并上色，中低端机首屏阻塞从约 150ms 降至约 30ms
- 代码块首屏先以纯文本显示，~50ms 后自动着色（感知瞬间）
- 文件切换路径同步改走 idle 上色，避免对首次 idle callback 的时序竞态

## 0.3.2

- Windows：首次启动写 HKCU 注册表（.md/.markdown/.mdown/.mkd 的 OpenWithProgids + ProgID 定义 + Applications 条目），并弹窗引导用户去「设置 › 默认应用」完成关联（Win10+ 禁止应用静默设为默认）
- Windows：窗口左上角 HICON 现在使用应用图标（此前为系统默认）
- Windows：图标改为"白底粗体 #"风格，与 macOS 视觉一致；`gen_ico.py` 独立绘制每个尺寸（16/32/48/64/128/256），小尺寸不再因降采样而糊
- 所有平台：md 中的 http/https/mailto 链接改为通过系统默认浏览器/邮件客户端打开，不再劫持在 WebView 内导航
- 新增依赖：`image`（png+ico 解码，用于窗口图标）+ `winreg`（Windows-only）

## 0.3.1

- Windows: release 构建关闭 console subsystem，双击/Finder 式启动不再弹黑色 cmd 窗口 (#2)
- Windows: 通过 `winresource` 在 `.exe` 中嵌入应用图标（`assets/icon.ico`）
- `gen_icon.py` 同时产出 macOS `.icns` 与 Windows 多尺寸 `.ico`（16/32/48/64/128/256）

## 0.3.0

- 极简应用图标（白底 "#"，简洁干净）
- CLI 参数：`md-preview file.md` 直接打开
- 离线代码语法高亮（自研 2KB mini highlighter，替代 119KB highlight.js）
- 支持关键词、字符串、注释、数字、函数名着色
- 暗色/亮色主题自动切换
- 刷新保持滚动位置
- 去掉 image crate，二进制 986KB（< 1MB）
- macOS .app 打包脚本（`bundle.sh`）

## 0.1.0

- 初始版本
- 支持拖放 .md 文件预览
- 支持 Cmd/Ctrl+O 打开文件
- 文件修改自动刷新
- 支持 GFM 表格、任务列表、删除线
- 暗色模式自适应
- Release 二进制仅 ~970KB
