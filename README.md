# MD Preview

> Ultra-lightweight Markdown preview app. ~1MB binary, zero Electron, pure native.

A blazing-fast, cross-platform Markdown preview tool built with **Rust** and system **WebView**. No bundled browser, no heavy runtimes — just a single ~1MB binary that renders your `.md` files beautifully.

![MD Preview Welcome Screen](https://raw.githubusercontent.com/vorojar/md-preview/master/screenshots/welcome.png)

## Why MD Preview?

| | MD Preview | Electron-based |
|---|---|---|
| **Binary size** | ~1.1 MB | 150+ MB |
| **Memory usage** | ~15 MB | 200+ MB |
| **Startup time** | Instant | 2-5 seconds |
| **Runtime** | System WebView | Bundled Chromium |

## Features

- **Drag & drop** — drop any `.md` file onto the window
- **CLI support** — `md-preview README.md` opens directly
- **Live reload** — edits refresh automatically via file watcher
- **Syntax highlighting** — 40+ languages, powered by highlight.js (fully offline)
- **Dark mode** — auto-follows system theme (macOS / Windows / Linux)
- **GFM support** — tables, task lists, strikethrough, heading anchors
- **Keyboard shortcuts** — `Cmd/Ctrl+O` to open files

## Install

### Build from source

```bash
# Prerequisites: Rust toolchain (https://rustup.rs)
git clone https://github.com/vorojar/md-preview.git
cd md-preview
cargo build --release
# Binary at: target/release/md-preview
```

### macOS .app bundle

```bash
# Build and package as macOS app
chmod +x bundle.sh
./bundle.sh
# Install to Applications
cp -r "target/MD Preview.app" /Applications/
```

## Usage

```bash
# Open with file argument
md-preview README.md

# Or launch and drag files in
md-preview
```

### Keyboard Shortcuts

| Shortcut | Action |
|---|---|
| `Cmd/Ctrl + O` | Open file dialog |
| Drag & Drop | Open `.md` / `.txt` file |

## Tech Stack

- **[Rust](https://www.rust-lang.org/)** — systems language, zero-cost abstractions
- **[wry](https://github.com/nicedavisit/nicedavisit)** — cross-platform WebView library (macOS: WebKit, Windows: WebView2, Linux: WebKitGTK)
- **[pulldown-cmark](https://github.com/raphlinus/pulldown-cmark)** — CommonMark/GFM Markdown parser
- **[highlight.js](https://highlightjs.org/)** — syntax highlighting for 40+ languages (embedded offline)
- **[notify](https://github.com/notify-rs/notify)** — cross-platform file watcher
- **[rfd](https://github.com/PolyMeilex/rfd)** — native file dialogs

## Cross-Platform

| Platform | WebView Engine | Status |
|---|---|---|
| macOS | WebKit (WKWebView) | ✅ Tested |
| Windows | WebView2 (Edge) | ✅ Supported |
| Linux | WebKitGTK | ✅ Supported |

## License

MIT
