# MD Preview

**English · [简体中文](README_zh.md)**

[![GitHub stars](https://img.shields.io/github/stars/vorojar/md-preview)](https://github.com/vorojar/md-preview/stargazers)
[![Release](https://img.shields.io/github/v/release/vorojar/md-preview)](https://github.com/vorojar/md-preview/releases)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Platform](https://img.shields.io/badge/platform-macOS%20%7C%20Windows%20%7C%20Linux-lightgrey)](https://github.com/vorojar/md-preview/releases)
[![Binary size](https://img.shields.io/badge/binary-~5MB-green)](https://github.com/vorojar/md-preview/releases)

> A native Markdown previewer for AI-generated docs, README files, plans, Mermaid diagrams, and technical notes. Open the file now, not a whole IDE.

MD Preview is a fast, local-first Markdown viewer built with **Rust** and the system **WebView**. It does not bundle Chromium, does not require Electron, and keeps all rendering assets offline. Drop in a Markdown file, open one from the terminal, or keep it beside Cursor, Claude Code, Codex, VS Code, Vim, Zed, or any editor that writes Markdown.

![MD Preview screenshot](https://raw.githubusercontent.com/vorojar/md-preview/master/screenshots/hero.jpg)

## Why It Exists

AI coding tools now generate a lot of Markdown: `README.md`, `plan.md`, task specs, architecture notes, changelogs, KaTeX formulas, and Mermaid diagrams. Most Markdown tools are still either full writing studios or editor plugins. MD Preview is deliberately smaller:

- **Open fast** - native binary, system WebView, no bundled browser runtime.
- **Stay local** - Markdown, syntax highlighting, math, and diagrams render on your machine.
- **Follow your editor** - save the file in Vim, VS Code, Cursor, Zed, or anything else; the preview refreshes automatically.
- **Keep reading clean** - the toolbar only appears on hover, so the document stays the focus.
- **Handle real Markdown** - code blocks, tables, task lists, math formulas, Mermaid diagrams, images, links, and print all work offline.

## Fits AI Coding Workflows

Use it as a small read-only window next to the tools that generate or edit your docs:

- Preview Claude Code / Codex / Cursor-generated plans without opening a full IDE.
- Keep Mermaid and KaTeX docs readable while your editor stays in source mode.
- Review local project notes, specs, and README drafts with live reload.
- Print or export the rendered preview when you need a clean PDF.

## Download

Get the latest build from [GitHub Releases](https://github.com/vorojar/md-preview/releases).

| Platform | Package | Notes |
|---|---|---|
| macOS | `MD-Preview-macOS-universal.dmg` | Universal app for Apple Silicon and Intel. Releases are signed and notarized. |
| Windows | `MD-Preview-windows-x64.zip` | Includes a GUI executable with the app icon embedded. |
| Linux | `MD-Preview-linux-x64.tar.gz` | Requires the system WebKitGTK runtime. |

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

# Or launch an empty window and drag in a file
md-preview
```

MD Preview accepts `.md` and `.txt` files through drag and drop, the open dialog, or the command line. Relative images are resolved from the Markdown file's directory, so local documentation folders render naturally.

## Features

| Feature | What it means |
|---|---|
| Drag and drop | Drop a Markdown file into the window and it opens immediately. |
| CLI open | `md-preview path/to/file.md` opens directly from a shell. |
| Live reload | External edits refresh the rendered document automatically. |
| Inline source edit | `Cmd/Ctrl+E` switches to source mode for quick edits; `Cmd/Ctrl+S` saves. |
| Native print | `Cmd/Ctrl+P` opens the platform print dialog and prints only the preview. |
| Syntax highlighting | highlight.js is embedded offline and injected after first paint. |
| Math | KaTeX renders `$...$`, `$$...$$`, `\(...\)`, and `\[...\]` on demand. |
| Diagrams | Mermaid fenced blocks render locally when the document actually uses them. |
| Dark mode | Follows the system color scheme across macOS, Windows, and Linux. |
| GitHub-flavored Markdown | Tables, task lists, strikethrough, heading attributes, and anchors. |
| External links | `http`, `https`, and `mailto` links open in the system browser or mail app. |
| Window restore | Last size and position are restored when still visible on a connected monitor. |
| Update check | After first paint, MD Preview checks GitHub Releases and shows a small update button if a newer version exists. |

## Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `Cmd/Ctrl + O` | Open file |
| `Cmd/Ctrl + E` | Toggle preview/source edit |
| `Cmd/Ctrl + S` | Save in source edit mode |
| `Cmd/Ctrl + P` | Print preview |
| `Esc` | Leave source edit mode and save if needed |

## Markdown Support

MD Preview uses `pulldown-cmark` for the base Markdown pass, then enhances the rendered document only when needed:

- CommonMark plus GFM-style tables, task lists, strikethrough, and heading attributes
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

MD Preview has no accounts, no telemetry, and no analytics. Your Markdown files stay on disk. Rendering happens locally. The only network request made by the app itself is the optional GitHub Releases update check after the first paint; failed checks are ignored and never block startup.

## Troubleshooting

**Linux does not launch**

Install WebKitGTK 4.1 packages for your distribution. On Debian/Ubuntu:

```bash
sudo apt-get install libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev
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

CI builds macOS, Windows, and Linux. Release tags matching `v*` produce a macOS DMG, Windows ZIP, and Linux tarball through GitHub Actions.

## License

[MIT](LICENSE)
