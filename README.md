# MD Preview

**English · [简体中文](README_zh.md)**

[![GitHub stars](https://img.shields.io/github/stars/vorojar/md-preview)](https://github.com/vorojar/md-preview/stargazers)
[![Release](https://img.shields.io/github/v/release/vorojar/md-preview)](https://github.com/vorojar/md-preview/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux%20%7C%20iOS%20%7C%20Android-lightgrey)](https://github.com/vorojar/md-preview/releases)
[![App Store](https://img.shields.io/badge/App%20Store-Local%20Markdown%20Preview-blue?logo=appstore)](https://apps.apple.com/cn/app/local-markdown-preview/id6779451523)
[![Binary size](https://img.shields.io/badge/binary-~5MB-green)](https://github.com/vorojar/md-preview/releases)

> A native Markdown previewer for AI-generated docs, README files, plans, Mermaid diagrams, and technical notes. Open the file now, not a whole IDE.

MD Preview is a fast, local-first Markdown viewer built with **Rust** and the system **WebView** on desktop, plus native iOS and Android shells for opening Markdown from Files, WeChat, WeCom, and system share sheets. It does not bundle Chromium, does not require Electron, and keeps all rendering assets offline. Drop in a Markdown file, open one from the terminal, or keep it beside Cursor, Claude Code, Codex, VS Code, Vim, Zed, or any editor that writes Markdown.

![MD Preview screenshot](https://raw.githubusercontent.com/vorojar/md-preview/master/screenshots/hero.jpg)

## Why It Exists

AI coding tools now generate a lot of Markdown: `README.md`, `plan.md`, task specs, architecture notes, changelogs, KaTeX formulas, and Mermaid diagrams. Most Markdown tools are still either full writing studios or editor plugins. MD Preview is deliberately smaller:

- **Open fast** - native binary, system WebView, no bundled browser runtime.
- **Stay local** - Markdown, syntax highlighting, math, and diagrams render on your machine.
- **Follow your editor** - save the file in Vim, VS Code, Cursor, Zed, or anything else; the preview refreshes automatically.
- **Keep reading clean** - the toolbar only appears on hover, and the start screen gives you Open File plus recent files.
- **Handle real Markdown** - code blocks, tables, task lists, math formulas, Mermaid diagrams, images, links, and print all work offline.

## Fits AI Coding Workflows

Use it as a small read-only window next to the tools that generate or edit your docs:

- Preview Claude Code / Codex / Cursor-generated plans without opening a full IDE.
- Keep Mermaid and KaTeX docs readable while your editor stays in source mode.
- Review local project notes, specs, and README drafts with live reload, recent files, and in-document search.
- Print or export the rendered preview when you need a clean PDF.

## Download

Get the latest build from [GitHub Releases](https://github.com/vorojar/md-preview/releases).

| Platform | Package | Notes |
|---|---|---|
| macOS | `MD-Preview-macOS-universal.dmg` | Universal app for Apple Silicon and Intel. Releases are signed and notarized. |
| Windows | `MD-Preview-windows-x64.exe` | Single-file app. The in-app updater downloads the next exe, verifies its SHA-256 digest, replaces itself, and relaunches. |
| Linux | `MD-Preview-linux-x64.tar.gz` | Requires the system WebKitGTK runtime. |
| iOS / iPadOS | [Local Markdown Preview on the App Store](https://apps.apple.com/cn/app/local-markdown-preview/id6779451523) | Native iPhone and iPad viewer for opening Markdown from Files and the iOS share sheet. |
| Android | `MD-Preview-Android.apk` | Native Android viewer for opening Markdown files from Files, WeChat, WeCom, and share sheets. |

Android builds are published as separate mobile releases, for example [mobile-android-v1.0.7](https://github.com/vorojar/md-preview/releases/tag/mobile-android-v1.0.7). The iOS build is now available on the App Store as [Local Markdown Preview](https://apps.apple.com/cn/app/local-markdown-preview/id6779451523).

You can also build from source:

```bash
git clone https://github.com/vorojar/md-preview.git
cd md-preview
cargo build --release
./target/release/md-preview README.md
```

To create the macOS `.app` bundle locally:

```bash
chmod +x bundle.sh
./bundle.sh
cp -r "target/MD Preview.app" /Applications/
```

## Usage

```bash
# Open a file directly
md-preview README.md

# Or launch an empty window, use Open File, pick a recent file, or drag one in
md-preview
```

MD Preview accepts `.md` and `.txt` files through drag and drop, the open dialog, recent files, or the command line. Relative images are resolved from the Markdown file's directory, so local documentation folders render naturally.

On iPhone and iPad, Local Markdown Preview opens Markdown and plain-text files from Files and the iOS share sheet. On Android, MD Preview appears in the system "Open with" and share flows for Markdown files. Recent files are cached privately inside the app, so files opened from temporary providers such as WeChat or WeCom remain available later; stale recent entries are removed safely instead of crashing.

## Features

| Feature | What it means |
|---|---|
| Start screen | Empty launches show Open File and local recent files, so the app is useful before anything is loaded. |
| Mobile open | iOS opens Markdown from Files and the share sheet; Android can open Markdown from Files, WeChat, WeCom, and Android share sheets. |
| Drag and drop | Drop a Markdown file into the window and it opens immediately. |
| CLI open | `md-preview path/to/file.md` opens directly from a shell. |
| Find in preview | `Cmd/Ctrl+F` opens a compact search bar for the rendered document. |
| Live reload | External edits refresh the rendered document automatically. |
| Inline source edit | `Cmd/Ctrl+E` switches to source mode for quick edits; `Cmd/Ctrl+S` saves. |
| Native print | `Cmd/Ctrl+P` opens the platform print dialog and prints only the preview. |
| Syntax highlighting | highlight.js is embedded offline and injected after first paint. |
| Math | KaTeX renders `$...$`, `$$...$$`, `\(...\)`, and `\[...\]` on demand. |
| Diagrams | Mermaid fenced blocks render locally when the document actually uses them. |
| GitHub Alerts | `[!NOTE]`, `[!TIP]`, `[!IMPORTANT]`, `[!WARNING]`, and `[!CAUTION]` blockquotes render as alert callouts. |
| Highlights | `==highlight==` renders as marked text for notes and AI-generated docs. |
| Dark mode | Follows the system color scheme across macOS, Windows, and Linux. |
| GitHub-flavored Markdown | Tables, task lists, strikethrough, heading attributes, and anchors. |
| External links | `http`, `https`, and `mailto` links open in the system browser or mail app. |
| Window restore | Last size and position are restored when still visible on a connected monitor. |
| Updates | After first paint, MD Preview checks desktop GitHub Releases. macOS uses Sparkle for signed in-app updates; Windows self-updates the single exe after SHA-256 verification; Linux opens the matching release download. |

## Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `Cmd/Ctrl + O` | Open file |
| `Cmd/Ctrl + F` | Find in preview |
| `Cmd/Ctrl + E` | Toggle preview/source edit |
| `Cmd/Ctrl + S` | Save in source edit mode |
| `Cmd/Ctrl + P` | Print preview |
| `Esc` | Leave source edit mode and save if needed |

## Markdown Support

MD Preview uses `pulldown-cmark` for the base Markdown pass, then enhances the rendered document only when needed:

- CommonMark plus GFM-style tables, task lists, strikethrough, and heading attributes
- GitHub-style alert blockquotes for notes, tips, warnings, and cautions
- `==highlight==` text marks used by many Markdown note tools
- Offline code highlighting for 40+ languages, including Delphi/Pascal
- Offline KaTeX math rendering with safeguards so Markdown emphasis does not break formulas
- Offline Mermaid rendering for fenced ```` ```mermaid ```` blocks
- Relative image paths through a per-file `<base>` URL
- Print CSS that removes app controls from printed output

The cold path stays small: regular Markdown renders first, while heavier enhancers such as highlight.js, KaTeX, and Mermaid are deferred until after the first visible paint or loaded only for documents that need them.

## How It Stays Small

MD Preview is not a Tauri or Electron app. It uses:

- **Rust** for the native shell and Markdown pipeline
- **wry** for the system WebView: WebKit on macOS, WebView2 on Windows, WebKitGTK on Linux
- **tao** for the cross-platform window/event loop
- **pulldown-cmark** for Markdown parsing
- **notify** for file watching
- **rfd** for native open dialogs

The release profile enables size-oriented optimization, LTO, one codegen unit, symbol stripping, and `panic = "abort"`.

## Privacy

MD Preview has no accounts, no telemetry, and no analytics. Your Markdown files stay on disk. Rendering happens locally. The only network request made by the desktop app itself is the optional update check after the first paint; failed checks are ignored and never block startup. macOS updates are verified by Sparkle using the app's embedded EdDSA public key. Windows self-updates verify the SHA-256 digest returned by GitHub Releases before replacing the running exe.

## Troubleshooting

**Linux does not launch**

Install WebKitGTK 4.1 packages for your distribution. On Debian/Ubuntu:

```bash
sudo apt-get install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev
```

**Linux opens a blank window on NVIDIA**

MD Preview automatically applies a conservative WebKitGTK fallback on Linux systems with the NVIDIA driver loaded. If your distribution still shows a blank WebView, start it manually with:

```bash
WEBKIT_DISABLE_DMABUF_RENDERER=1 md-preview your-file.md
```

If that does not help, try:

```bash
WEBKIT_DISABLE_COMPOSITING_MODE=1 md-preview your-file.md
```

**Windows cannot set MD Preview as the default app automatically**

Windows does not allow apps to silently take over file associations. MD Preview registers itself in the "Open with" list; choose it from Explorer or Windows Settings.

**A formula or diagram shows as text**

Make sure the syntax is valid Markdown/KaTeX/Mermaid. Math and Mermaid are loaded on demand, so documents without those patterns do not pay the startup cost.

## Development

```bash
cargo build
cargo test
cargo build --release
```

CI builds macOS, Windows, and Linux. Release tags matching `v*` produce a macOS DMG, standalone Windows EXE, and Linux tarball through GitHub Actions.

Maintainer release flow:

```bash
scripts/release.sh v1.2.3
```

The script runs verification, pushes `master` and the tag, waits for GitHub Actions, signs/notarizes/staples the macOS DMG in the foreground, uploads `appcast.xml`, and verifies the final Release assets.

## License

[MIT](LICENSE)
