# MD Preview

**[English](README.md) · 简体中文**

[![GitHub stars](https://img.shields.io/github/stars/vorojar/md-preview)](https://github.com/vorojar/md-preview/stargazers) [![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](https://opensource.org/licenses/MIT) [![Release](https://img.shields.io/github/v/release/vorojar/md-preview)](https://github.com/vorojar/md-preview/releases) [![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)](https://github.com/vorojar/md-preview/releases) [![Binary size](https://img.shields.io/badge/binary%20size-~1MB-green)](https://github.com/vorojar/md-preview)

> 极轻量的 Markdown 预览工具，约 1MB 二进制，无 Electron，纯原生实现。

用 **Rust** 和系统 **WebView** 写的跨平台 Markdown 预览工具。不打包浏览器、没有沉重运行时 —— 单文件约 1MB，把 `.md` 文件渲染得漂漂亮亮。

![MD Preview](https://raw.githubusercontent.com/vorojar/md-preview/master/screenshots/hero.jpg)

## 为什么用 MD Preview？

|  | MD Preview | Electron 类 |
|---|---|---|
| **二进制大小** | ~1.1 MB | 150+ MB |
| **内存占用** | ~15 MB | 200+ MB |
| **启动时间** | 瞬间 | 2–5 秒 |
| **运行时** | 系统 WebView | 内置 Chromium |

## 功能

- **拖拽打开** — 把任意 `.md` 拖到窗口
- **命令行** — `md-preview README.md` 直接打开
- **实时刷新** — 文件改动自动重渲染（基于文件监听）
- **源码编辑** — 右上角 hover 时浮现的工具栏切换预览 / 编辑模式；`Cmd/Ctrl+E` 切换，`Cmd/Ctrl+S` 保存；只想阅读时工具栏自动隐去，不打扰
- **打印** — `Cmd/Ctrl+P` 或工具栏按钮调起系统打印；工具栏和编辑区在打印样式下自动隐藏
- **代码高亮** — 40+ 语言，完全离线（延迟到首屏之后加载，不阻塞渲染）
- **暗色模式** — 自动跟随系统主题（macOS / Windows / Linux）
- **GFM 支持** — 表格、任务列表、删除线、标题锚点
- **外链跳系统浏览器** — md 里的 http/https/mailto 链接用默认浏览器/邮件客户端打开
- **文件关联（Windows）** — 首次启动自动把 `.md` 注册到"打开方式"列表
- **记住窗口** — 退出时记录位置/大小，下次启动恢复；首次启动在主显示器居中

## 下载预编译版

直接去 [Releases](https://github.com/vorojar/md-preview/releases) 下载对应系统的包：

- macOS：`MD-Preview-macOS-universal.dmg`（Universal，Apple Silicon + Intel 通用）
- Windows：`MD-Preview-windows-x64.zip`
- Linux：`MD-Preview-linux-x64.tar.gz`

## 从源码构建

```bash
# 需要 Rust 工具链：https://rustup.rs
git clone https://github.com/vorojar/md-preview.git
cd md-preview
cargo build --release
# 产物位置：target/release/md-preview
```

### 打成 macOS .app

```bash
chmod +x bundle.sh
./bundle.sh
# 安装到 /Applications
cp -r "target/MD Preview.app" /Applications/
```

## 使用

```bash
# 带文件参数打开
md-preview README.md

# 或者空启动，拖文件进来
md-preview
```

### 快捷键

| 快捷键 | 作用 |
|---|---|
| `Cmd/Ctrl + O` | 打开文件对话框 |
| `Cmd/Ctrl + E` | 切换预览 / 源码编辑 |
| `Cmd/Ctrl + S` | 保存（编辑模式下）|
| `Cmd/Ctrl + P` | 打印 |
| `Esc` | 退出编辑模式（自动保存）|
| 拖拽 | 打开 `.md` / `.txt` 文件 |

## 技术栈

- **[Rust](https://www.rust-lang.org/)** — 系统级语言，零成本抽象
- **[wry](https://github.com/tauri-apps/wry)** — 跨平台 WebView 库（macOS: WebKit，Windows: WebView2，Linux: WebKitGTK）
- **[pulldown-cmark](https://github.com/raphlinus/pulldown-cmark)** — CommonMark / GFM 解析器
- **[highlight.js](https://highlightjs.org/)** — 40+ 语言语法高亮（离线嵌入）
- **[notify](https://github.com/notify-rs/notify)** — 跨平台文件监听
- **[rfd](https://github.com/PolyMeilex/rfd)** — 原生文件对话框

## 跨平台支持

| 平台 | WebView 引擎 | 状态 |
|---|---|---|
| macOS | WebKit (WKWebView) | ✅ 已测试 |
| Windows | WebView2 (Edge) | ✅ 支持 |
| Linux | WebKitGTK | ✅ 支持 |

## 许可证

MIT
