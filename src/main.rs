#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

use notify::{Event, RecursiveMode, Watcher};
use pulldown_cmark::{html, Options, Parser};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tao::dpi::{LogicalPosition, LogicalSize};
use tao::event::{Event as TaoEvent, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use tao::window::{Window, WindowBuilder};
use wry::WebViewBuilder;

const ICON_BYTES: &[u8] = include_bytes!("../assets/icon.ico");
const DEFAULT_W: f64 = 900.0;
const DEFAULT_H: f64 = 700.0;

#[derive(Debug)]
enum UserEvent {
    FileChanged, // external change: refresh preview AND textarea
    FileSaved,   // our own save: refresh preview only, leave textarea cursor alone
    DirtyChanged(bool),
    Print, // route print through wry's native API (WKWebView ignores window.print())
    Ready, // first paint landed: inject hljs now; if bench mode, also exit
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
struct EnhanceFlags {
    math: bool,
    mermaid: bool,
}

impl EnhanceFlags {
    fn any(self) -> bool {
        self.math || self.mermaid
    }
}

fn is_help_arg(arg: &str) -> bool {
    arg == "-h" || arg == "--help"
}

fn print_help() {
    println!(
        "MD Preview {}\n\nUsage:\n  md-preview [file.md]\n\nOptions:\n  -h, --help    Show this help message",
        env!("CARGO_PKG_VERSION")
    );
}

#[derive(Copy, Clone)]
enum Lang {
    Zh,
    En,
}

fn detect_lang() -> Lang {
    sys_locale::get_locale()
        .map(|l| {
            if l.to_lowercase().starts_with("zh") {
                Lang::Zh
            } else {
                Lang::En
            }
        })
        .unwrap_or(Lang::En)
}

struct Strings {
    drop_hint: &'static str,
    cannot_read: &'static str,
    btn_edit: &'static str,
    btn_preview: &'static str,
    btn_print: &'static str,
    btn_update: &'static str,
}

impl Strings {
    fn for_lang(lang: Lang) -> Self {
        match lang {
            Lang::Zh => Strings {
                drop_hint: "拖入 .md 文件 或按 Cmd/Ctrl+O 打开",
                cannot_read: "无法读取文件",
                btn_edit: "编辑 (Cmd/Ctrl+E)",
                btn_preview: "预览 (Cmd/Ctrl+E)",
                btn_print: "打印 (Cmd/Ctrl+P)",
                btn_update: "发现新版本",
            },
            Lang::En => Strings {
                drop_hint: "Drop a .md file here or press Cmd/Ctrl+O to open",
                cannot_read: "Cannot read file",
                btn_edit: "Edit (Cmd/Ctrl+E)",
                btn_preview: "Preview (Cmd/Ctrl+E)",
                btn_print: "Print (Cmd/Ctrl+P)",
                btn_update: "Update available",
            },
        }
    }
}

fn config_dir() -> PathBuf {
    #[cfg(target_os = "windows")]
    {
        std::env::var_os("LOCALAPPDATA")
            .or_else(|| std::env::var_os("APPDATA"))
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("md-preview")
    }
    #[cfg(not(target_os = "windows"))]
    {
        std::env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_default()
            .join(".config/md-preview")
    }
}

#[derive(Copy, Clone)]
struct WindowGeom {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}

fn geom_path() -> PathBuf {
    config_dir().join("window.geom")
}

fn load_window_geom() -> Option<WindowGeom> {
    let txt = fs::read_to_string(geom_path()).ok()?;
    let parts: Vec<&str> = txt.trim().split(',').collect();
    if parts.len() != 4 {
        return None;
    }
    Some(WindowGeom {
        x: parts[0].parse().ok()?,
        y: parts[1].parse().ok()?,
        w: parts[2].parse().ok()?,
        h: parts[3].parse().ok()?,
    })
}

fn save_window_geom(window: &Window) {
    let Ok(pos) = window.outer_position() else {
        return;
    };
    let scale = window.scale_factor();
    let size = window.inner_size();
    let geom = WindowGeom {
        x: pos.x as f64 / scale,
        y: pos.y as f64 / scale,
        w: size.width as f64 / scale,
        h: size.height as f64 / scale,
    };
    if geom.w < 200.0 || geom.h < 150.0 {
        return;
    }
    let dir = config_dir();
    let _ = fs::create_dir_all(&dir);
    let _ = fs::write(
        dir.join("window.geom"),
        format!("{},{},{},{}", geom.x, geom.y, geom.w, geom.h),
    );
}

/// Return the saved geometry only when its center still falls inside
/// some connected monitor — prevents the window from landing off-screen
/// after a display swap.
fn geom_visible(geom: &WindowGeom, event_loop: &EventLoop<UserEvent>) -> bool {
    let cx = geom.x + geom.w / 2.0;
    let cy = geom.y + geom.h / 2.0;
    event_loop.available_monitors().any(|mon| {
        let scale = mon.scale_factor();
        let mp = mon.position();
        let ms = mon.size();
        let mx = mp.x as f64 / scale;
        let my = mp.y as f64 / scale;
        let mw = ms.width as f64 / scale;
        let mh = ms.height as f64 / scale;
        cx >= mx && cx <= mx + mw && cy >= my && cy <= my + mh
    })
}

fn centered_geom(event_loop: &EventLoop<UserEvent>) -> WindowGeom {
    if let Some(mon) = event_loop.primary_monitor() {
        let scale = mon.scale_factor();
        let mp = mon.position();
        let ms = mon.size();
        let mx = mp.x as f64 / scale;
        let my = mp.y as f64 / scale;
        let mw = ms.width as f64 / scale;
        let mh = ms.height as f64 / scale;
        WindowGeom {
            x: mx + (mw - DEFAULT_W) / 2.0,
            y: my + (mh - DEFAULT_H) / 2.0,
            w: DEFAULT_W,
            h: DEFAULT_H,
        }
    } else {
        WindowGeom {
            x: 100.0,
            y: 100.0,
            w: DEFAULT_W,
            h: DEFAULT_H,
        }
    }
}

fn md_to_html(md: &str) -> String {
    let opts = Options::ENABLE_TABLES
        | Options::ENABLE_STRIKETHROUGH
        | Options::ENABLE_TASKLISTS
        | Options::ENABLE_HEADING_ATTRIBUTES
        | Options::ENABLE_MATH;
    let parser = Parser::new_ext(md, opts);
    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);
    html_out
}

// Embedded highlight.js + themes (offline)
const HLJS_JS: &str = include_str!("../assets/hljs/highlight.min.js");
const HLJS_LIGHT: &str = include_str!("../assets/hljs/github.min.css");
const HLJS_DARK: &str = include_str!("../assets/hljs/github-dark.min.css");
// Extra language pack(s) not in the `common` bundle. Each file
// ends with `hljs.registerLanguage(...)` and only works if evaluated
// in the same scope as the main bundle — we concat them into hljs-src.
const HLJS_EXTRA_LANGS: &str = concat!(
    // Delphi / Pascal (aliases: dpr, dfm, pas, pascal) — user requested
    include_str!("../assets/hljs/delphi.min.js"),
);
const PREVIEW_ENHANCE_JS: &str = include_str!("../assets/enhance/preview-enhance.js");
const UPDATE_CHECK_JS: &str = include_str!("../assets/enhance/update-check.js");
const KATEX_JS: &str = include_str!("../assets/katex/katex.min.js");
const KATEX_CSS: &str = include_str!("../assets/katex/katex.inline.css");
const MERMAID_JS: &str = include_str!("../assets/mermaid/mermaid.min.js");

fn html_escape_ta(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;")
}

fn html_escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
}

fn percent_encode_file_path(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for &b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b':' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{b:02X}")),
        }
    }
    out
}

fn base_href_for_file(path: &Path) -> Option<String> {
    let dir = path.parent()?;
    Some(file_url_for_path_dir(dir))
}

fn file_url_for_path_dir(dir: &Path) -> String {
    let mut path = dir.to_string_lossy().replace('\\', "/");
    if cfg!(windows) && !path.starts_with('/') {
        path.insert(0, '/');
    }
    if !path.ends_with('/') {
        path.push('/');
    }
    format!("file://{}", percent_encode_file_path(&path))
}

fn starts_mermaid_fence(line: &str) -> bool {
    let trimmed = line.trim_start();
    let rest = trimmed
        .strip_prefix("```")
        .or_else(|| trimmed.strip_prefix("~~~"));
    let Some(info) = rest else {
        return false;
    };
    let info = info.trim_start();
    info == "mermaid"
        || info
            .strip_prefix("mermaid")
            .and_then(|s| s.chars().next())
            .map(|c| c.is_whitespace() || c == '{')
            .unwrap_or(false)
}

fn has_unescaped_at(s: &str, index: usize, needle: &str) -> bool {
    if !s[index..].starts_with(needle) {
        return false;
    }
    let mut backslashes = 0;
    for b in s[..index].bytes().rev() {
        if b == b'\\' {
            backslashes += 1;
        } else {
            break;
        }
    }
    backslashes % 2 == 0
}

fn has_unescaped_pair(s: &str, open: &str, close: &str) -> bool {
    let mut pos = 0;
    while let Some(rel) = s[pos..].find(open) {
        let start = pos + rel;
        if !has_unescaped_at(s, start, open) {
            pos = start + open.len();
            continue;
        }
        let body_start = start + open.len();
        let mut search = body_start;
        while let Some(close_rel) = s[search..].find(close) {
            let close_at = search + close_rel;
            if has_unescaped_at(s, close_at, close) {
                return true;
            }
            search = close_at + close.len();
        }
        pos = body_start;
    }
    false
}

fn has_inline_dollar_math(s: &str) -> bool {
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'$' || !has_unescaped_at(s, i, "$") {
            i += 1;
            continue;
        }
        if bytes.get(i + 1).copied() == Some(b'$')
            || bytes
                .get(i + 1)
                .map(|b| b.is_ascii_whitespace())
                .unwrap_or(true)
        {
            i += 1;
            continue;
        }
        let mut j = i + 1;
        while j < bytes.len() {
            if bytes[j] == b'$'
                && has_unescaped_at(s, j, "$")
                && bytes
                    .get(j.wrapping_sub(1))
                    .map(|b| !b.is_ascii_whitespace())
                    .unwrap_or(false)
            {
                return true;
            }
            j += 1;
        }
        i += 1;
    }
    false
}

fn enhance_flags_for(md: &str) -> EnhanceFlags {
    EnhanceFlags {
        math: has_unescaped_pair(md, "$$", "$$")
            || has_unescaped_pair(md, "\\[", "\\]")
            || has_unescaped_pair(md, "\\(", "\\)")
            || has_inline_dollar_math(md),
        mermaid: md.lines().any(starts_mermaid_fence),
    }
}

fn build_enhancer_bootstrap(flags: EnhanceFlags, loaded: EnhanceFlags) -> Vec<String> {
    if !flags.any() {
        return Vec::new();
    }

    let mut scripts = Vec::new();
    if flags.math && !loaded.math {
        let mut js = String::from("(function(){\nif(!window.katex){\n");
        js.push_str(KATEX_JS);
        js.push_str("\n;try{window.katex=katex;}catch(e){}\n}\n");
        js.push_str("if(window.__setKatexCss)window.__setKatexCss('");
        js.push_str(&escape_js(KATEX_CSS));
        js.push_str("');\n})();");
        scripts.push(js);
    }
    if flags.mermaid && !loaded.mermaid {
        // Mermaid's standalone bundle expects global script scope. Keep it
        // out of the function wrapper that is safe for KaTeX/highlight.js.
        let mut js = String::with_capacity(MERMAID_JS.len() + 80);
        js.push_str(MERMAID_JS);
        js.push_str("\n;try{window.mermaid=mermaid;}catch(e){}\n");
        scripts.push(js);
    }
    scripts.push("if(window.__enhancePreview)window.__enhancePreview();".to_string());
    scripts
}

fn build_page(
    preview_html: &str,
    raw_md: &str,
    base_href: Option<&str>,
    flags: EnhanceFlags,
    s: &Strings,
    empty: bool,
) -> String {
    let body_class = if empty { "empty" } else { "" };
    let base_tag = base_href
        .map(|href| format!(r#"<base id="base-href" href="{}">"#, html_escape_attr(href)))
        .unwrap_or_else(|| r#"<base id="base-href">"#.to_string());
    format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8">
{base_tag}
<style id="hljs-light">{css_light}</style>
<style id="hljs-dark" media="not all">{css_dark}</style>
<script>
(function(){{
  var mq = window.matchMedia('(prefers-color-scheme: dark)');
  function apply(e) {{
    document.getElementById('hljs-light').media = e.matches ? 'not all' : '';
    document.getElementById('hljs-dark').media = e.matches ? '' : 'not all';
  }}
  apply(mq); mq.addEventListener('change', apply);
}})();
</script>
<style>
:root {{ color-scheme: light dark; }}
/* Reserve scrollbar space permanently so the fixed toolbar doesn't shift
   between modes (one with scrollbar, one without). */
html {{ overflow-y: scroll; scrollbar-gutter: stable; }}
body {{
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif;
  margin: 0; padding: 0;
  line-height: 1.6; font-size: 15px;
  color: #1a1a1a; background: #fff;
}}
#app {{ max-width: 820px; margin: 0 auto; padding: 24px; }}
@media (prefers-color-scheme: dark) {{
  body {{ color: #d4d4d4; background: #1e1e1e; }}
  #preview a {{ color: #6cb6ff; }}
  #preview pre {{ background: #2d2d2d !important; }}
  #preview code:not(pre code) {{ background: #2d2d2d; }}
  #preview blockquote {{ border-color: #444; color: #999; }}
  #preview table th {{ background: #2d2d2d; color: #f0f0f0; }}
  #preview table td, #preview table th {{ border-color: #444; }}
  #preview hr {{ border-color: #333; }}
}}
#preview h1,#preview h2,#preview h3,#preview h4 {{ margin-top: 1.4em; }}
#preview h1 {{ border-bottom: 1px solid #e1e4e8; padding-bottom: .3em; }}
#preview h2 {{ border-bottom: 1px solid #e1e4e8; padding-bottom: .2em; }}
#preview code {{ background: #f0f0f0; padding: 2px 6px; border-radius: 4px; font-size: 90%; }}
#preview pre {{ background: #f6f8fa; padding: 16px; border-radius: 8px; overflow-x: auto; }}
#preview pre code {{ background: none; padding: 0; font-size: 14px; }}
#preview blockquote {{ border-left: 4px solid #ddd; margin: 0; padding: 0 1em; color: #666; }}
#preview table {{ border-collapse: collapse; width: 100%; }}
#preview table th, #preview table td {{ border: 1px solid #ddd; padding: 8px 12px; text-align: left; }}
#preview table th {{ background: #f6f8fa; font-weight: 600; color: #1a1a1a; }}
#preview img {{ max-width: 100%; }}
#preview .katex-display {{ overflow-x: auto; overflow-y: hidden; padding: 0.15em 0; }}
#preview .mdp-mermaid {{ margin: 1.2em 0; overflow-x: auto; text-align: center; }}
#preview .mdp-mermaid svg {{ max-width: 100%; height: auto; }}
#preview .mdp-mermaid-error, #preview .mdp-math-error {{ color: #b42318; }}
#preview hr {{ border: none; border-top: 1px solid #e1e4e8; margin: 2em 0; }}
#preview a {{ color: #0969da; text-decoration: none; }}
#preview a:hover {{ text-decoration: underline; }}
#preview ul, #preview ol {{ padding-left: 2em; }}
#preview input[type="checkbox"] {{ margin-right: 6px; }}
.empty {{ display: flex; flex-direction: column; align-items: center; justify-content: center;
  height: 60vh; color: #999; font-size: 18px; gap: 12px; }}
.empty .icon {{ font-size: 48px; opacity: 0.4; }}

/* Floating toolbar (top-right) — hover-reveal, hidden in empty state */
.toolbar {{
  position: fixed; top: 10px; right: 12px;
  display: flex; gap: 6px; z-index: 100;
  opacity: 0; pointer-events: none;
  transition: opacity 0.18s ease;
}}
html:hover .toolbar {{ opacity: 1; pointer-events: auto; }}
body.empty .toolbar {{ display: none !important; }}
.toolbar button {{
  width: 34px; height: 34px; padding: 0;
  background: rgba(255,255,255,0.8);
  backdrop-filter: blur(6px);
  -webkit-backdrop-filter: blur(6px);
  border: 1px solid rgba(0,0,0,0.08);
  border-radius: 8px;
  display: grid; place-items: center;
  cursor: pointer; color: #555;
  transition: color 0.15s, background 0.15s;
}}
.toolbar button:hover {{ color: #000; background: rgba(255,255,255,1); }}
.toolbar button[hidden] {{ display: none !important; }}
.toolbar .update-btn {{ color: #0969da; }}
@media (prefers-color-scheme: dark) {{
  .toolbar button {{
    background: rgba(40,40,40,0.8);
    border-color: rgba(255,255,255,0.1);
    color: #bbb;
  }}
  .toolbar button:hover {{ color: #fff; background: rgba(55,55,55,1); }}
  .toolbar .update-btn {{ color: #6cb6ff; }}
}}

/* Source editor textarea — height is auto-grown by JS to match content,
   so the page (html) owns the only vertical scrollbar. */
#editor {{
  display: none;
  width: 100%;
  box-sizing: border-box;
  border: none; outline: none; resize: none;
  overflow: hidden;
  font: 14px/1.6 "SF Mono","Menlo","Consolas",monospace;
  background: transparent; color: inherit;
  padding: 0;
}}
body.editing #preview {{ display: none; }}
body.editing #editor {{ display: block; padding: 16px 24px; }}
body.editing #app {{ max-width: none; padding: 0; }}
body.editing #btn-print {{ display: none; }}

@media print {{
  .toolbar, #editor {{ display: none !important; }}
  #preview {{ display: block !important; }}
  #app {{ max-width: none; padding: 0; }}
}}
</style></head><body class="{body_class}">
<div class="toolbar">
  <button id="btn-toggle" title="{btn_edit}" aria-label="{btn_edit}"></button>
  <button id="btn-print" title="{btn_print}" aria-label="{btn_print}"></button>
  <button id="btn-update" class="update-btn" hidden title="{btn_update}" aria-label="{btn_update}"></button>
</div>
<div id="app">
  <div id="preview">{preview_html}</div>
  <textarea id="editor" spellcheck="false">{raw_md_escaped}</textarea>
</div>
<script>
(function(){{
  var ICON_EDIT = '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/></svg>';
  var ICON_VIEW = '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>';
  var ICON_PRINT = '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 6 2 18 2 18 9"/><path d="M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2"/><rect x="6" y="14" width="12" height="8"/></svg>';
  var ICON_UPDATE = '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 3v12"/><path d="m7 10 5 5 5-5"/><path d="M5 21h14"/></svg>';
  var L_EDIT = '{btn_edit}', L_VIEW = '{btn_preview}';

  var btnToggle = document.getElementById('btn-toggle');
  var btnPrint = document.getElementById('btn-print');
  var btnUpdate = document.getElementById('btn-update');
  var ta = document.getElementById('editor');
  var dirty = false;

  btnToggle.innerHTML = ICON_EDIT;
  btnPrint.innerHTML = ICON_PRINT;
  btnUpdate.innerHTML = ICON_UPDATE;

  function inEdit() {{ return document.body.classList.contains('editing'); }}
  document.addEventListener('contextmenu', function(e) {{
    if (!inEdit() || e.target !== ta) e.preventDefault();
  }});
  function setDirty(d) {{
    if (dirty === d) return;
    dirty = d;
    window.ipc.postMessage(d ? 'dirty:1' : 'dirty:0');
  }}
  function save() {{
    window.ipc.postMessage('save:' + ta.value);
    setDirty(false);
  }}
  // Grow textarea height to its content so the page (html) owns the sole
  // scrollbar; avoids the double-scrollbar you see if textarea keeps its
  // own internal scroll.
  function autoResize() {{
    var x = window.scrollX || document.documentElement.scrollLeft || 0;
    var y = window.scrollY || document.documentElement.scrollTop || 0;
    ta.style.height = 'auto';
    var h = Math.max(ta.scrollHeight, window.innerHeight);
    ta.style.height = h + 'px';
    window.scrollTo(x, y);
  }}
  function enterEdit() {{
    document.body.classList.add('editing');
    btnToggle.innerHTML = ICON_VIEW;
    btnToggle.title = L_VIEW;
    btnToggle.setAttribute('aria-label', L_VIEW);
    autoResize();
    ta.focus();
  }}
  function leaveEdit() {{
    if (dirty) save();
    document.body.classList.remove('editing');
    btnToggle.innerHTML = ICON_EDIT;
    btnToggle.title = L_EDIT;
    btnToggle.setAttribute('aria-label', L_EDIT);
  }}

  btnToggle.addEventListener('click', function() {{
    if (inEdit()) leaveEdit(); else enterEdit();
  }});
  btnPrint.addEventListener('click', function() {{
    if (inEdit()) leaveEdit();
    // Route through Rust: WKWebView ignores window.print(); wry's
    // WebView::print() calls the right native API on each platform.
    setTimeout(function(){{ window.ipc.postMessage('print'); }}, 0);
  }});
  ta.addEventListener('input', function() {{ setDirty(true); autoResize(); }});
  window.addEventListener('resize', function() {{ if (inEdit()) autoResize(); }});

  document.addEventListener('keydown', function(e) {{
    if ((e.metaKey || e.ctrlKey) && (e.key === 'r' || e.key === 'R')) {{
      e.preventDefault();
      if (!inEdit()) window.ipc.postMessage('refresh');
      return;
    }}
    if ((e.metaKey || e.ctrlKey) && (e.key === 'o' || e.key === 'O')) {{
      e.preventDefault();
      window.ipc.postMessage('open');
      return;
    }}
    if ((e.metaKey || e.ctrlKey) && (e.key === 'e' || e.key === 'E')) {{
      e.preventDefault();
      if (inEdit()) leaveEdit(); else enterEdit();
      return;
    }}
    if ((e.metaKey || e.ctrlKey) && (e.key === 's' || e.key === 'S')) {{
      if (inEdit()) {{ e.preventDefault(); save(); }}
      return;
    }}
    if ((e.metaKey || e.ctrlKey) && (e.key === 'p' || e.key === 'P')) {{
      e.preventDefault();
      if (inEdit()) leaveEdit();
      setTimeout(function(){{ window.ipc.postMessage('print'); }}, 0);
      return;
    }}
    if (e.key === 'Escape' && inEdit()) {{ leaveEdit(); }}
  }});

  // Called by Rust after a save (only preview is refreshed) or after an
  // external file change (both preview + textarea are refreshed).
  window.__setPreview = function(previewHtml, needsMath, needsMermaid) {{
    if (arguments.length > 1 && window.__setFeatureFlags) {{
      window.__setFeatureFlags(needsMath, needsMermaid);
    }}
    document.getElementById('preview').innerHTML = previewHtml;
    (window.requestIdleCallback || function(fn){{ return setTimeout(fn, 0); }})(function() {{
      if (typeof hljs !== 'undefined') hljs.highlightAll();
      if (window.__enhancePreview) window.__enhancePreview();
    }});
  }};
  window.__setBaseHref = function(baseHref) {{
    var base = document.getElementById('base-href');
    if (!base) {{
      base = document.createElement('base');
      base.id = 'base-href';
      document.head.insertBefore(base, document.head.firstChild);
    }}
    if (baseHref) base.setAttribute('href', baseHref);
    else base.removeAttribute('href');
  }};
  window.__setContent = function(previewHtml, rawMd, baseHref, needsMath, needsMermaid) {{
    document.body.classList.remove('empty');
    window.__setBaseHref(baseHref);
    window.__setPreview(previewHtml, needsMath, needsMermaid);
    if (!inEdit() || !dirty) {{
      ta.value = rawMd;
      setDirty(false);
      if (inEdit()) autoResize();
    }}
  }};

  // Defer hljs parse + initial highlight to idle time.
  // hljs itself is NOT inlined in this page — Rust injects it via
  // evaluate_script once we tell it we're painted. Until that injection
  // runs, typeof hljs === 'undefined' and highlightAll is skipped; once
  // it lands, hljs.highlightAll() gets called by the injected bootstrap
  // and __setPreview.

  // Signal Rust after first paint (triggers hljs inject; bench mode exits).
  requestAnimationFrame(function() {{
    requestAnimationFrame(function() {{
      if (window.ipc) window.ipc.postMessage('ready');
    }});
  }});
}})();
window.__mdPreviewFeatureFlags = {{ math: {needs_math}, mermaid: {needs_mermaid} }};
{preview_enhance_js}
{update_check_js}
window.__mdPreviewInstallUpdateCheck({{
  currentVersion: '{app_version}',
  buttonLabel: '{btn_update_js}',
  apiUrl: 'https://api.github.com/repos/vorojar/md-preview/releases/latest',
  latestUrl: 'https://github.com/vorojar/md-preview/releases/latest'
}});
</script>
</body></html>"#,
        css_light = HLJS_LIGHT,
        css_dark = HLJS_DARK,
        base_tag = base_tag,
        preview_html = preview_html,
        raw_md_escaped = html_escape_ta(raw_md),
        btn_edit = s.btn_edit,
        btn_preview = s.btn_preview,
        btn_print = s.btn_print,
        btn_update = s.btn_update,
        btn_update_js = escape_js(s.btn_update),
        app_version = env!("CARGO_PKG_VERSION"),
        body_class = body_class,
        needs_math = flags.math,
        needs_mermaid = flags.mermaid,
        preview_enhance_js = PREVIEW_ENHANCE_JS,
        update_check_js = UPDATE_CHECK_JS,
    )
}

fn escape_js(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn is_allowed_update_url(url: &str) -> bool {
    url == "https://github.com/vorojar/md-preview/releases/latest"
        || url.starts_with("https://github.com/vorojar/md-preview/releases/tag/")
}

fn watch_scope_for_file(path: &Path) -> &Path {
    path.parent()
        .filter(|parent| !parent.as_os_str().is_empty())
        .unwrap_or(path)
}

fn event_path_matches_file(event_path: &Path, target: &Path) -> bool {
    event_path == target
        || (target.file_name().is_some()
            && event_path.file_name() == target.file_name()
            && event_path.parent() == target.parent())
}

fn event_should_reload_file(ev: &Event, target: &Path) -> bool {
    if ev.need_rescan() {
        return true;
    }

    if !(ev.kind.is_modify() || ev.kind.is_create() || ev.kind.is_remove()) {
        return false;
    }

    ev.paths
        .iter()
        .any(|event_path| event_path_matches_file(event_path, target))
}

#[cfg(test)]
mod tests {
    use super::*;
    use notify::EventKind;

    #[test]
    fn dollar_math_is_protected_from_markdown_escapes() {
        let html = md_to_html(r"$\{x\}$");

        assert!(html.contains(r#"<span class="math math-inline">\{x\}</span>"#));
    }

    #[test]
    fn dollar_math_is_protected_from_markdown_emphasis() {
        let html = md_to_html(r"$\bar{\mu}_{n}$ and $x_{n}$");

        assert!(html.contains(r#"<span class="math math-inline">\bar{\mu}_{n}</span>"#));
        assert!(html.contains(r#"<span class="math math-inline">x_{n}</span>"#));
        assert!(!html.contains("<em>"));
    }

    #[test]
    fn help_flags_are_recognized() {
        assert!(is_help_arg("-h"));
        assert!(is_help_arg("--help"));
        assert!(!is_help_arg("--edit"));
    }

    #[test]
    fn page_blocks_native_preview_reload_paths() {
        let strings = Strings::for_lang(Lang::En);
        let page = build_page(
            &md_to_html("# Hello"),
            "# Hello",
            None,
            EnhanceFlags::default(),
            &strings,
            false,
        );

        assert!(page.contains("document.addEventListener('contextmenu'"));
        assert!(page.contains("window.ipc.postMessage('refresh')"));
    }

    #[test]
    fn vim_style_target_rewrite_events_reload_current_file() {
        let target = PathBuf::from("/tmp/note.md");
        let ev = Event::new(EventKind::Create(notify::event::CreateKind::File))
            .add_path(PathBuf::from("/tmp/note.md"));

        assert!(event_should_reload_file(&ev, &target));
    }

    #[test]
    fn sibling_file_events_do_not_reload_current_file() {
        let target = PathBuf::from("/tmp/note.md");
        let ev = Event::new(EventKind::Modify(notify::event::ModifyKind::Data(
            notify::event::DataChange::Any,
        )))
        .add_path(PathBuf::from("/tmp/other.md"));

        assert!(!event_should_reload_file(&ev, &target));
    }

    #[test]
    fn file_watch_scope_is_parent_directory() {
        let target = PathBuf::from("/tmp/note.md");

        assert_eq!(watch_scope_for_file(&target), Path::new("/tmp"));
    }
}

/// Decode embedded icon.ico to an RGBA tao Icon for the window chrome.
fn load_window_icon() -> Option<tao::window::Icon> {
    let img = image::load_from_memory_with_format(ICON_BYTES, image::ImageFormat::Ico).ok()?;
    let rgba = img.to_rgba8();
    let (w, h) = rgba.dimensions();
    tao::window::Icon::from_rgba(rgba.into_raw(), w, h).ok()
}

#[cfg(target_os = "macos")]
fn register_as_default(_lang: Lang) {
    use std::process::Command;
    let marker = config_dir().join(".md-preview-registered");
    if marker.exists() {
        return;
    }
    let _ = Command::new("swift")
        .arg("-")
        .stdin(std::process::Stdio::piped())
        .spawn()
        .and_then(|mut child| {
            use std::io::Write;
            if let Some(ref mut stdin) = child.stdin {
                let _ = stdin.write_all(b"import Foundation\nimport CoreServices\nlet _ = LSSetDefaultRoleHandlerForContentType(\"net.daringfireball.markdown\" as NSString, .viewer, \"com.mdpreview.app\" as NSString)\n");
            }
            child.wait()
        });
    let _ = fs::create_dir_all(marker.parent().unwrap());
    let _ = fs::write(&marker, "");
}

/// Windows: write HKCU registry so .md shows up in the "Open with" list, then
/// prompt the user once to finish wiring the default app (Win8+ blocks silent
/// default-handler changes — only the Settings app can confirm it).
#[cfg(target_os = "windows")]
fn register_as_default(_lang: Lang) {
    use winreg::enums::HKEY_CURRENT_USER;
    use winreg::RegKey;

    let marker_dir = config_dir();
    let marker = marker_dir.join(".md-preview-registered");
    if marker.exists() {
        return;
    }

    let Ok(exe) = std::env::current_exe() else {
        return;
    };
    let exe_str = exe.to_string_lossy().to_string();
    let progid = "MDPreview.md";
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);

    // Advertise MD Preview as a choice for these extensions.
    for ext in [".md", ".markdown", ".mdown", ".mkd"] {
        let path = format!(r"Software\Classes\{ext}\OpenWithProgids");
        if let Ok((key, _)) = hkcu.create_subkey(&path) {
            let _ = key.set_value::<String, _>(progid, &String::new());
        }
    }

    // ProgID definition: description, icon, open command.
    let progid_root = format!(r"Software\Classes\{progid}");
    if let Ok((k, _)) = hkcu.create_subkey(&progid_root) {
        let _ = k.set_value("", &"Markdown Document".to_string());
        let _ = k.set_value("FriendlyTypeName", &"Markdown Document".to_string());
    }
    if let Ok((k, _)) = hkcu.create_subkey(format!(r"{progid_root}\DefaultIcon")) {
        let _ = k.set_value("", &format!("\"{exe_str}\",0"));
    }
    if let Ok((k, _)) = hkcu.create_subkey(format!(r"{progid_root}\shell\open\command")) {
        let _ = k.set_value("", &format!("\"{exe_str}\" \"%1\""));
    }

    // Applications\<exe-name> entry gives us a friendly label in the "Open with" menu.
    if let Some(exe_name) = exe.file_name().map(|n| n.to_string_lossy().to_string()) {
        let app_root = format!(r"Software\Classes\Applications\{exe_name}");
        if let Ok((k, _)) = hkcu.create_subkey(&app_root) {
            let _ = k.set_value("FriendlyAppName", &"MD Preview".to_string());
        }
        if let Ok((k, _)) = hkcu.create_subkey(format!(r"{app_root}\shell\open\command")) {
            let _ = k.set_value("", &format!("\"{exe_str}\" \"%1\""));
        }
        if let Ok((k, _)) = hkcu.create_subkey(format!(r"{app_root}\SupportedTypes")) {
            for ext in [".md", ".markdown", ".mdown", ".mkd"] {
                let _ = k.set_value::<String, _>(ext, &String::new());
            }
        }
    }

    let _ = fs::create_dir_all(&marker_dir);
    let _ = fs::write(&marker, "");
    // Intentionally no dialog: users can pick MD Preview via "Open with"
    // whenever they want, and Win10+ blocks silent default-handler changes
    // anyway — asking them to click through Settings on first launch is noise.
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn register_as_default(_lang: Lang) {}

#[cfg(target_os = "macos")]
fn install_macos_edit_menu() {
    use objc2::runtime::Sel;
    use objc2::sel;
    use objc2::MainThreadOnly;
    use objc2_app_kit::{NSApplication, NSEventModifierFlags, NSMenu, NSMenuItem};
    use objc2_foundation::{MainThreadMarker, NSString};

    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };

    fn menu(title: &str, mtm: MainThreadMarker) -> objc2::rc::Retained<NSMenu> {
        NSMenu::initWithTitle(NSMenu::alloc(mtm), &NSString::from_str(title))
    }

    fn item(
        title: &str,
        action: Option<Sel>,
        key: &str,
        modifiers: NSEventModifierFlags,
        mtm: MainThreadMarker,
    ) -> objc2::rc::Retained<NSMenuItem> {
        let item = unsafe {
            NSMenuItem::initWithTitle_action_keyEquivalent(
                NSMenuItem::alloc(mtm),
                &NSString::from_str(title),
                action,
                &NSString::from_str(key),
            )
        };
        item.setKeyEquivalentModifierMask(modifiers);
        item
    }

    let app = NSApplication::sharedApplication(mtm);
    let main_menu = menu("", mtm);

    let app_menu = menu("MD Preview", mtm);
    app_menu.addItem(&item(
        "Quit MD Preview",
        Some(sel!(terminate:)),
        "q",
        NSEventModifierFlags::Command,
        mtm,
    ));
    let app_menu_item = item("MD Preview", None, "", NSEventModifierFlags::empty(), mtm);
    app_menu_item.setSubmenu(Some(&app_menu));
    main_menu.addItem(&app_menu_item);

    let edit_menu = menu("Edit", mtm);
    edit_menu.addItem(&item(
        "Undo",
        Some(sel!(undo:)),
        "z",
        NSEventModifierFlags::Command,
        mtm,
    ));
    edit_menu.addItem(&item(
        "Redo",
        Some(sel!(redo:)),
        "z",
        NSEventModifierFlags::Command | NSEventModifierFlags::Shift,
        mtm,
    ));
    edit_menu.addItem(&NSMenuItem::separatorItem(mtm));
    edit_menu.addItem(&item(
        "Cut",
        Some(sel!(cut:)),
        "x",
        NSEventModifierFlags::Command,
        mtm,
    ));
    edit_menu.addItem(&item(
        "Copy",
        Some(sel!(copy:)),
        "c",
        NSEventModifierFlags::Command,
        mtm,
    ));
    edit_menu.addItem(&item(
        "Paste",
        Some(sel!(paste:)),
        "v",
        NSEventModifierFlags::Command,
        mtm,
    ));
    edit_menu.addItem(&item(
        "Select All",
        Some(sel!(selectAll:)),
        "a",
        NSEventModifierFlags::Command,
        mtm,
    ));
    let edit_menu_item = item("Edit", None, "", NSEventModifierFlags::empty(), mtm);
    edit_menu_item.setSubmenu(Some(&edit_menu));
    main_menu.addItem(&edit_menu_item);

    app.setMainMenu(Some(&main_menu));
}

#[cfg(not(target_os = "macos"))]
fn install_macos_edit_menu() {}

fn main() {
    // Bench instrumentation: MD_PREVIEW_BENCH=1 makes the app print
    // cold-start timings to stderr and exit as soon as the first paint
    // lands. Costs nothing outside bench mode.
    let bench = std::env::var("MD_PREVIEW_BENCH").is_ok();
    let t0 = Instant::now();
    let bench_log = |label: &str| {
        if bench {
            eprintln!("[bench] +{}ms {}", t0.elapsed().as_millis(), label);
        }
    };
    bench_log("main_start");

    // CLI: md-preview [file.md]
    let arg1 = std::env::args().nth(1);
    if arg1.as_deref().map(is_help_arg).unwrap_or(false) {
        print_help();
        return;
    }

    let lang = detect_lang();
    let strings = Strings::for_lang(lang);
    register_as_default(lang);
    bench_log("after_register");

    let initial_file: Option<PathBuf> = arg1.map(PathBuf::from).and_then(|p| {
        let p = if p.is_relative() {
            std::env::current_dir().unwrap_or_default().join(p)
        } else {
            p
        };
        if p.exists() {
            Some(p)
        } else {
            eprintln!("File not found: {}", p.display());
            None
        }
    });

    let event_loop: EventLoop<UserEvent> = EventLoopBuilder::with_user_event().build();
    install_macos_edit_menu();
    let proxy = event_loop.create_proxy();

    let title = initial_file
        .as_ref()
        .and_then(|p| p.file_name())
        .map(|n| format!("{} — MD Preview", n.to_string_lossy()))
        .unwrap_or_else(|| "MD Preview".to_string());

    let geom = load_window_geom()
        .filter(|g| geom_visible(g, &event_loop))
        .unwrap_or_else(|| centered_geom(&event_loop));

    let mut window_builder = WindowBuilder::new()
        .with_title(&title)
        .with_inner_size(LogicalSize::new(geom.w, geom.h))
        .with_position(LogicalPosition::new(geom.x, geom.y));
    if let Some(icon) = load_window_icon() {
        window_builder = window_builder.with_window_icon(Some(icon));
    }
    let window = window_builder
        .build(&event_loop)
        .expect("failed to build window");
    bench_log("window_built");

    let mut initial_flags = EnhanceFlags::default();
    let initial_page = match &initial_file {
        Some(path) => fs::read_to_string(path).ok().map_or_else(
            || {
                build_page(
                    &format!(
                        r#"<div class="empty"><div class="icon">#</div>{}</div>"#,
                        strings.cannot_read
                    ),
                    "",
                    None,
                    EnhanceFlags::default(),
                    &strings,
                    true,
                )
            },
            |raw| {
                let html_body = md_to_html(&raw);
                let base_href = base_href_for_file(path);
                initial_flags = enhance_flags_for(&raw);
                build_page(
                    &html_body,
                    &raw,
                    base_href.as_deref(),
                    initial_flags,
                    &strings,
                    false,
                )
            },
        ),
        None => build_page(
            &format!(
                r#"<div class="empty"><div class="icon">#</div>{}</div>"#,
                strings.drop_hint
            ),
            "",
            None,
            EnhanceFlags::default(),
            &strings,
            true,
        ),
    };

    let file_path: Arc<Mutex<Option<PathBuf>>> = Arc::new(Mutex::new(initial_file));
    let enhance_flags: Arc<Mutex<EnhanceFlags>> = Arc::new(Mutex::new(initial_flags));
    let last_self_write: Arc<Mutex<Option<Instant>>> = Arc::new(Mutex::new(None));
    let file_path_for_ipc = Arc::clone(&file_path);
    let last_self_write_for_ipc = Arc::clone(&last_self_write);
    let proxy_for_ipc = proxy.clone();

    // Windows: steer WebView2's cache/cookie tree into %LOCALAPPDATA% instead of
    // letting it drop next to the exe. Other platforms: use default (None).
    let data_dir: Option<PathBuf> = {
        #[cfg(target_os = "windows")]
        {
            let d = config_dir().join("WebView2");
            let _ = fs::create_dir_all(&d);
            Some(d)
        }
        #[cfg(not(target_os = "windows"))]
        {
            None
        }
    };
    let mut web_context = wry::WebContext::new(data_dir);

    let builder = WebViewBuilder::with_web_context(&mut web_context)
        .with_html(&initial_page)
        .with_navigation_handler(|url: String| {
            // Let wry load the initial in-memory document; route any real URL click
            // (http/https/mailto) to the system default handler.
            if url.starts_with("http://")
                || url.starts_with("https://")
                || url.starts_with("mailto:")
            {
                let _ = open::that(&url);
                false
            } else {
                true
            }
        })
        .with_ipc_handler(move |msg| {
            let body = msg.body();
            if body == "open" {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Markdown", &["md", "markdown", "mdown", "mkd", "txt"])
                    .pick_file()
                {
                    *file_path_for_ipc.lock().unwrap() = Some(path);
                    let _ = proxy_for_ipc.send_event(UserEvent::FileChanged);
                }
            } else if body == "dirty:1" {
                let _ = proxy_for_ipc.send_event(UserEvent::DirtyChanged(true));
            } else if body == "dirty:0" {
                let _ = proxy_for_ipc.send_event(UserEvent::DirtyChanged(false));
            } else if body == "print" {
                let _ = proxy_for_ipc.send_event(UserEvent::Print);
            } else if body == "ready" {
                let _ = proxy_for_ipc.send_event(UserEvent::Ready);
            } else if body == "refresh" {
                let _ = proxy_for_ipc.send_event(UserEvent::FileChanged);
            } else if let Some(url) = body.strip_prefix("open-url:") {
                if is_allowed_update_url(url) {
                    let _ = open::that(url);
                }
            } else if let Some(content) = body.strip_prefix("save:") {
                let fp = file_path_for_ipc.lock().unwrap().clone();
                if let Some(path) = fp {
                    *last_self_write_for_ipc.lock().unwrap() = Some(Instant::now());
                    if fs::write(&path, content).is_ok() {
                        let _ = proxy_for_ipc.send_event(UserEvent::FileSaved);
                    }
                }
            }
        })
        .with_drag_drop_handler({
            let file_path = Arc::clone(&file_path);
            let proxy = proxy.clone();
            move |event| {
                if let wry::DragDropEvent::Drop { paths, .. } = event {
                    if let Some(p) = paths.into_iter().find(|p| {
                        p.extension()
                            .map(|e| {
                                let e = e.to_string_lossy().to_lowercase();
                                e == "md" || e == "markdown" || e == "txt"
                            })
                            .unwrap_or(false)
                    }) {
                        *file_path.lock().unwrap() = Some(p);
                        let _ = proxy.send_event(UserEvent::FileChanged);
                    }
                }
                true
            }
        });

    #[cfg(target_os = "linux")]
    let webview = {
        use tao::platform::unix::WindowExtUnix;
        use wry::WebViewBuilderExtUnix;

        let vbox = window
            .default_vbox()
            .expect("failed to get default GTK container");
        builder.build_gtk(vbox).expect("failed to build webview")
    };
    #[cfg(not(target_os = "linux"))]
    let webview = builder.build(&window).expect("failed to build webview");
    bench_log("webview_built");

    // hljs + extra language packs aren't part of first-paint HTML anymore.
    // We push them in via evaluate_script the moment the webview tells us
    // it's painted (IPC 'ready'). Keeps ~125KB out of the HTML-parse critical
    // path so the app window shows content faster on cold start.
    let hljs_bootstrap = format!(
        "(function(){{{hljs_js};{hljs_extra};try{{window.hljs=hljs;}}catch(e){{}}if(typeof hljs!=='undefined'&&hljs.highlightAll){{hljs.highlightAll();}}}})();",
        hljs_js = HLJS_JS,
        hljs_extra = HLJS_EXTRA_LANGS,
    );

    // File watcher state
    let watcher_holder: Arc<Mutex<Option<notify::RecommendedWatcher>>> = Arc::new(Mutex::new(None));
    let file_path_for_event = Arc::clone(&file_path);
    let watcher_for_event = Arc::clone(&watcher_holder);

    // If opened with CLI arg, setup watcher immediately
    if file_path_for_event.lock().unwrap().is_some() {
        let proxy_init = proxy.clone();
        let last_self_write_init = Arc::clone(&last_self_write);
        let fp = file_path_for_event.lock().unwrap().clone();
        if let Some(ref path) = fp {
            let target_path = path.clone();
            if let Ok(mut watcher) = notify::recommended_watcher(move |res: Result<Event, _>| {
                if let Ok(ev) = res {
                    if event_should_reload_file(&ev, &target_path) {
                        let suppress = last_self_write_init
                            .lock()
                            .unwrap()
                            .map(|t| t.elapsed() < Duration::from_millis(500))
                            .unwrap_or(false);
                        if !suppress {
                            let _ = proxy_init.send_event(UserEvent::FileChanged);
                        }
                    }
                }
            }) {
                let _ = watcher.watch(watch_scope_for_file(path), RecursiveMode::NonRecursive);
                *watcher_holder.lock().unwrap() = Some(watcher);
            }
        }
    }

    let mut loaded_enhancers = EnhanceFlags::default();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            TaoEvent::UserEvent(UserEvent::FileChanged) => {
                let fp = file_path_for_event.lock().unwrap().clone();
                if let Some(ref path) = fp {
                    if let Ok(raw) = fs::read_to_string(path) {
                        let html = md_to_html(&raw);
                        let base_href = base_href_for_file(path).unwrap_or_default();
                        let flags = enhance_flags_for(&raw);
                        *enhance_flags.lock().unwrap() = flags;
                        let js = format!(
                            "if(window.__setContent)window.__setContent('{}', '{}', '{}', {}, {});",
                            escape_js(&html),
                            escape_js(&raw),
                            escape_js(&base_href),
                            flags.math,
                            flags.mermaid
                        );
                        let _ = webview.evaluate_script(&js);
                        for js in build_enhancer_bootstrap(flags, loaded_enhancers) {
                            let _ = webview.evaluate_script(&js);
                        }
                        loaded_enhancers.math |= flags.math;
                        loaded_enhancers.mermaid |= flags.mermaid;

                        let name = path
                            .file_name()
                            .map(|n| n.to_string_lossy().to_string())
                            .unwrap_or_default();
                        window.set_title(&format!("{} — MD Preview", name));
                    }

                    // Re-setup watcher for current file
                    let mut w = watcher_for_event.lock().unwrap();
                    *w = None;
                    let proxy_clone = proxy.clone();
                    let last_self_write_cb = Arc::clone(&last_self_write);
                    let target_path = path.clone();
                    if let Ok(mut new_watcher) =
                        notify::recommended_watcher(move |res: Result<Event, _>| {
                            if let Ok(ev) = res {
                                if event_should_reload_file(&ev, &target_path) {
                                    let suppress = last_self_write_cb
                                        .lock()
                                        .unwrap()
                                        .map(|t| t.elapsed() < Duration::from_millis(500))
                                        .unwrap_or(false);
                                    if !suppress {
                                        let _ = proxy_clone.send_event(UserEvent::FileChanged);
                                    }
                                }
                            }
                        })
                    {
                        let _ = new_watcher
                            .watch(watch_scope_for_file(path), RecursiveMode::NonRecursive);
                        *w = Some(new_watcher);
                    }
                }
            }
            TaoEvent::UserEvent(UserEvent::FileSaved) => {
                let fp = file_path_for_event.lock().unwrap().clone();
                if let Some(ref path) = fp {
                    if let Ok(raw) = fs::read_to_string(path) {
                        let html = md_to_html(&raw);
                        let flags = enhance_flags_for(&raw);
                        *enhance_flags.lock().unwrap() = flags;
                        let js = format!(
                            "if(window.__setPreview)window.__setPreview('{}', {}, {});",
                            escape_js(&html),
                            flags.math,
                            flags.mermaid
                        );
                        let _ = webview.evaluate_script(&js);
                        for js in build_enhancer_bootstrap(flags, loaded_enhancers) {
                            let _ = webview.evaluate_script(&js);
                        }
                        loaded_enhancers.math |= flags.math;
                        loaded_enhancers.mermaid |= flags.mermaid;
                    }
                }
            }
            TaoEvent::UserEvent(UserEvent::DirtyChanged(dirty)) => {
                let fp = file_path_for_event.lock().unwrap().clone();
                let name = fp
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "MD Preview".to_string());
                let prefix = if dirty { "• " } else { "" };
                window.set_title(&format!("{}{} — MD Preview", prefix, name));
            }
            TaoEvent::UserEvent(UserEvent::Print) => {
                let _ = webview.print();
            }
            TaoEvent::UserEvent(UserEvent::Ready) => {
                // First paint is on the screen; now push hljs into the page
                // (kept out of first-paint HTML to keep it slim). Always do
                // this, even in bench mode, so subsequent panes would still
                // highlight — bench just exits right after measuring.
                let _ = webview.evaluate_script(&hljs_bootstrap);
                let flags = *enhance_flags.lock().unwrap();
                for js in build_enhancer_bootstrap(flags, loaded_enhancers) {
                    let _ = webview.evaluate_script(&js);
                }
                loaded_enhancers.math |= flags.math;
                loaded_enhancers.mermaid |= flags.mermaid;
                if bench {
                    eprintln!("[bench] +{}ms ready", t0.elapsed().as_millis());
                    *control_flow = ControlFlow::Exit;
                }
            }
            // macOS: double-click .md file in Finder opens the app with this event
            TaoEvent::Opened { urls } => {
                for url in urls {
                    if let Ok(path) = url.to_file_path() {
                        *file_path_for_event.lock().unwrap() = Some(path);
                        let _ = proxy.send_event(UserEvent::FileChanged);
                        break;
                    }
                }
            }
            TaoEvent::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                save_window_geom(&window);
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}
