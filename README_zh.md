# MD Preview

**[English](README.md) · 简体中文**

[![GitHub stars](https://img.shields.io/github/stars/vorojar/md-preview)](https://github.com/vorojar/md-preview/stargazers)
[![Release](https://img.shields.io/github/v/release/vorojar/md-preview)](https://github.com/vorojar/md-preview/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux%20%7C%20iOS%20%7C%20Android-lightgrey)](https://github.com/vorojar/md-preview/releases)
[![App Store](https://img.shields.io/badge/App%20Store-Local%20Markdown%20Preview-blue?logo=appstore)](https://apps.apple.com/cn/app/local-markdown-preview/id6779451523)
[![Binary size](https://img.shields.io/badge/binary-~5MB-green)](https://github.com/vorojar/md-preview/releases)

> 给 AI 生成文档、README、计划文档、Mermaid 图和技术笔记用的原生 Markdown 预览器：打开 `.md` 文件，不必顺手启动一整个 IDE。

MD Preview 是用 **Rust** 和系统 **WebView** 写的本地优先 Markdown 预览工具，桌面端覆盖 macOS、Windows、Linux，手机端提供 iOS 和 Android 原生外壳，方便从文件管理器、微信、企业微信和系统分享面板打开 Markdown。它不打包 Chromium，不依赖 Electron，渲染资源全部离线内置。你可以拖入文件、从命令行打开文件，或者把它放在 Cursor、Claude Code、Codex、VS Code、Vim、Zed 等常用工具旁边，当一扇干净的预览窗口。

![MD Preview 截图](https://raw.githubusercontent.com/vorojar/md-preview/master/screenshots/hero.jpg)

## 为什么做它

AI 编程工具现在会生成大量 Markdown：`README.md`、`plan.md`、任务说明、架构笔记、变更记录、KaTeX 公式和 Mermaid 图。很多 Markdown 工具仍然要么是完整写作套件，要么是编辑器插件。MD Preview 刻意保持更小：

- **打开快**：原生二进制、系统 WebView，不带一份浏览器运行时。
- **本地渲染**：Markdown、代码高亮、数学公式、Mermaid 图表都在本机完成。
- **跟随你的编辑器**：用 Vim、VS Code、Cursor、Zed 或任何编辑器保存文件，预览自动刷新。
- **阅读不打扰**：工具栏只在 hover 时出现，空白首页提供打开文件和最近文件，文档始终是主角。
- **覆盖真实文档**：代码块、表格、任务列表、公式、图表、图片、链接、打印都能离线工作。

## 适合 AI 编程工作流

把它放在生成或编辑文档的工具旁边，做一个轻量只读窗口：

- 预览 Claude Code / Codex / Cursor 生成的计划文档，不必打开完整 IDE。
- 编辑器保持源码模式，旁边实时看 Mermaid 和 KaTeX 渲染结果。
- 审阅本地项目笔记、规格说明和 README 草稿，保存后自动刷新，也能从最近文件继续并在预览里搜索。
- 需要干净 PDF 时，直接打印渲染后的预览。

## 下载

从 [GitHub Releases](https://github.com/vorojar/md-preview/releases) 下载最新版。

| 平台 | 包名 | 说明 |
|---|---|---|
| macOS | `MD-Preview-macOS-universal.dmg` | Apple Silicon 和 Intel 通用。Release 版本会签名、公证。 |
| Windows | `MD-Preview-windows-x64.exe` | 单文件应用。应用内更新会下载新版 exe，校验 SHA-256，退出后替换自己并重启。 |
| Linux | `MD-Preview-linux-x64.tar.gz` | 需要系统 WebKitGTK 运行时。 |
| iOS / iPadOS | [App Store 上的 Local Markdown Preview](https://apps.apple.com/cn/app/local-markdown-preview/id6779451523) | 原生 iPhone / iPad 预览器，可从“文件”和 iOS 分享面板打开 Markdown。 |
| Android | `MD-Preview-Android.apk` | 原生 Android 预览器，可从文件管理器、微信、企业微信和分享面板打开 Markdown。 |

Android 版本以单独的 mobile release 发布，例如 [mobile-android-v1.0.7](https://github.com/vorojar/md-preview/releases/tag/mobile-android-v1.0.7)。iOS 版本已经在 App Store 上架，名称为 [Local Markdown Preview](https://apps.apple.com/cn/app/local-markdown-preview/id6779451523)。

也可以从源码构建：

```bash
git clone https://github.com/vorojar/md-preview.git
cd md-preview
cargo build --release
./target/release/md-preview README.md
```

本地打包 macOS `.app`：

```bash
chmod +x bundle.sh
./bundle.sh
cp -r "target/MD Preview.app" /Applications/
```

## 使用

```bash
# 直接打开文件
md-preview README.md

# 或空启动后点打开文件、选择最近文件，或拖入文件
md-preview
```

MD Preview 支持通过拖拽、打开对话框、最近文件或命令行打开 `.md` / `.txt` 文件。相对路径图片会按 Markdown 文件所在目录解析，本地文档目录可以自然渲染。

iPhone 和 iPad 上，Local Markdown Preview 可以从“文件”和 iOS 分享面板打开 Markdown / 文本文件。Android 上，MD Preview 会出现在 Markdown 文件的“打开方式”和分享流程中。Recent 文件会缓存到应用私有目录，从微信、企业微信等临时来源打开过的文档后续也能继续打开；如果条目失效，会安全移除而不是闪退。

## 功能

| 功能 | 说明 |
|---|---|
| 启动首页 | 空白启动时显示打开文件和本机最近文件，没加载文档也有明确入口。 |
| 手机端打开 | iOS 支持从“文件”和分享面板打开 Markdown；Android 支持从文件管理器、微信、企业微信和系统分享面板打开 Markdown。 |
| 拖拽打开 | 把 Markdown 文件拖进窗口即可打开。 |
| 命令行打开 | `md-preview path/to/file.md` 直接从 shell 打开。 |
| 预览搜索 | `Cmd/Ctrl+F` 打开轻量搜索栏，在渲染后的文档内查找。 |
| 实时刷新 | 外部编辑保存后，预览自动更新。 |
| 源码编辑 | `Cmd/Ctrl+E` 切到源码模式快速改字，`Cmd/Ctrl+S` 保存。 |
| 原生打印 | `Cmd/Ctrl+P` 打开系统打印对话框，只打印预览内容。 |
| 代码高亮 | highlight.js 离线内置，首屏之后再注入，不阻塞打开。 |
| 数学公式 | KaTeX 按需渲染 `$...$`、`$$...$$`、`\(...\)`、`\[...\]`。 |
| 图表 | Mermaid fenced code block 只在文档实际使用时本地渲染。 |
| GitHub Alerts | `[!NOTE]`、`[!TIP]`、`[!IMPORTANT]`、`[!WARNING]`、`[!CAUTION]` blockquote 会渲染成提示块。 |
| 高亮标记 | `==高亮==` 会渲染成 marked text，适合笔记和 AI 生成文档。 |
| 暗色模式 | 自动跟随 macOS、Windows、Linux 的系统主题。 |
| GFM 支持 | 表格、任务列表、删除线、heading attributes、标题锚点。 |
| 外链跳转 | `http`、`https`、`mailto` 链接交给系统浏览器或邮件客户端。 |
| 窗口恢复 | 下次启动恢复上次位置和大小；断开显示器后自动回到可见屏幕。 |
| 更新 | 首屏之后检查桌面版 GitHub Releases。macOS 使用 Sparkle 做签名校验和应用内更新；Windows 校验 SHA-256 后自替换单 exe；Linux 打开对应 release 下载。 |

## 快捷键

| 快捷键 | 作用 |
|---|---|
| `Cmd/Ctrl + O` | 打开文件 |
| `Cmd/Ctrl + F` | 在预览里搜索 |
| `Cmd/Ctrl + E` | 切换预览 / 源码编辑 |
| `Cmd/Ctrl + S` | 源码编辑模式下保存 |
| `Cmd/Ctrl + P` | 打印预览 |
| `Esc` | 退出源码编辑模式，并在需要时保存 |

## Markdown 支持

MD Preview 先用 `pulldown-cmark` 完成基础 Markdown 解析，再只在需要时增强渲染结果：

- CommonMark 加 GFM 风格表格、任务列表、删除线、heading attributes
- GitHub 风格 Alerts blockquote，支持 note、tip、important、warning、caution
- 很多笔记工具常用的 `==高亮==` 文本标记
- 40+ 语言离线代码高亮，包含 Delphi / Pascal
- KaTeX 离线数学公式渲染，并保护公式不被 Markdown 强调语法破坏
- Mermaid 离线渲染 ```` ```mermaid ```` fenced code block
- 通过按文件设置 `<base>` 支持相对路径图片
- 打印样式自动移除应用工具栏和源码编辑区

普通 Markdown 走轻路径：文档先可见，highlight.js、KaTeX、Mermaid 这类较重的增强逻辑延迟到首屏之后，或只在文档真正需要时加载。

## 为什么能小

MD Preview 不是 Tauri，也不是 Electron。它主要使用：

- **Rust**：原生 shell 和 Markdown 管线
- **wry**：系统 WebView，macOS 是 WebKit，Windows 是 WebView2，Linux 是 WebKitGTK
- **tao**：跨平台窗口和事件循环
- **pulldown-cmark**：Markdown 解析
- **notify**：文件监听
- **rfd**：原生打开文件对话框

Release profile 使用面向体积的优化、LTO、单 codegen unit、符号裁剪和 `panic = "abort"`。

## 隐私

MD Preview 没有账号、没有 telemetry、没有 analytics。你的 Markdown 文件留在本地磁盘，渲染也在本机完成。桌面应用自身唯一的网络请求是首屏之后的可选更新检测；失败会静默忽略，不影响启动和预览。macOS 更新由 Sparkle 使用应用内置的 EdDSA 公钥校验；Windows 自更新会先校验 GitHub Releases 返回的 SHA-256 digest，再替换当前 exe。

## 常见问题

**Linux 启动不了**

请安装发行版对应的 WebKitGTK 4.1 包。Debian / Ubuntu 可以执行：

```bash
sudo apt-get install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev
```

**Linux NVIDIA 环境打开后窗口空白**

MD Preview 会在检测到 Linux 系统已加载 NVIDIA 驱动时自动启用更保守的 WebKitGTK fallback。如果你的发行版仍然出现 WebView 空白，可以手动这样启动：

```bash
WEBKIT_DISABLE_DMABUF_RENDERER=1 md-preview your-file.md
```

如果仍然异常，再尝试：

```bash
WEBKIT_DISABLE_COMPOSITING_MODE=1 md-preview your-file.md
```

**Windows 不能自动设为默认 Markdown 应用**

Windows 不允许应用静默接管文件关联。MD Preview 会把自己注册到“打开方式”列表，你可以在资源管理器或 Windows 设置里选择它。

**公式或图表显示成普通文本**

先确认语法是合法的 Markdown / KaTeX / Mermaid。公式和图表都是按需加载的，不包含这些语法的文档不会承担额外启动成本。

## 开发

```bash
cargo build
cargo test
cargo build --release
```

CI 会构建 macOS、Windows、Linux。推送匹配 `v*` 的 tag 后，GitHub Actions 会产出 macOS DMG、Windows 单文件 EXE 和 Linux tarball。

维护者发版流程：

```bash
scripts/release.sh v1.2.3
```

脚本会前台执行验证、推送 `master` 和 tag、等待 GitHub Actions、签名/公证/staple macOS DMG、上传 `appcast.xml`，并验证最终 Release assets。

## 许可证

[MIT](LICENSE)
