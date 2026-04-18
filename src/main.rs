#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

use notify::{Event, EventKind, RecursiveMode, Watcher};
use pulldown_cmark::{Options, Parser, html};
use std::fs;
use std::path::PathBuf;
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
}

#[derive(Copy, Clone)]
enum Lang {
    Zh,
    En,
}

fn detect_lang() -> Lang {
    sys_locale::get_locale()
        .map(|l| if l.to_lowercase().starts_with("zh") { Lang::Zh } else { Lang::En })
        .unwrap_or(Lang::En)
}

struct Strings {
    drop_hint: &'static str,
    cannot_read: &'static str,
    btn_edit: &'static str,
    btn_preview: &'static str,
    btn_print: &'static str,
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
            },
            Lang::En => Strings {
                drop_hint: "Drop a .md file here or press Cmd/Ctrl+O to open",
                cannot_read: "Cannot read file",
                btn_edit: "Edit (Cmd/Ctrl+E)",
                btn_preview: "Preview (Cmd/Ctrl+E)",
                btn_print: "Print (Cmd/Ctrl+P)",
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
        | Options::ENABLE_HEADING_ATTRIBUTES;
    let parser = Parser::new_ext(md, opts);
    let mut html_out = String::new();
    html::push_html(&mut html_out, parser);
    html_out
}

// Embedded highlight.js + themes (offline)
const HLJS_JS: &str = include_str!("../assets/hljs/highlight.min.js");
const HLJS_LIGHT: &str = include_str!("../assets/hljs/github.min.css");
const HLJS_DARK: &str = include_str!("../assets/hljs/github-dark.min.css");

fn html_escape_ta(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;")
}

fn build_page(preview_html: &str, raw_md: &str, s: &Strings, empty: bool) -> String {
    let body_class = if empty { "empty" } else { "" };
    format!(
        r#"<!DOCTYPE html><html><head><meta charset="utf-8">
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
  #preview table th {{ background: #2d2d2d; }}
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
#preview table th {{ background: #f6f8fa; font-weight: 600; }}
#preview img {{ max-width: 100%; }}
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
@media (prefers-color-scheme: dark) {{
  .toolbar button {{
    background: rgba(40,40,40,0.8);
    border-color: rgba(255,255,255,0.1);
    color: #bbb;
  }}
  .toolbar button:hover {{ color: #fff; background: rgba(55,55,55,1); }}
}}

/* Source editor textarea */
#editor {{
  display: none;
  width: 100%;
  min-height: calc(100vh - 48px);
  box-sizing: border-box;
  border: none; outline: none; resize: none;
  font: 14px/1.6 "SF Mono","Menlo","Consolas",monospace;
  background: transparent; color: inherit;
  padding: 0;
}}
body.editing #preview {{ display: none; }}
body.editing #editor {{ display: block; padding: 16px 24px; min-height: 100vh; }}
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
</div>
<div id="app">
  <div id="preview">{preview_html}</div>
  <textarea id="editor" spellcheck="false">{raw_md_escaped}</textarea>
</div>
<script id="hljs-src" type="text/x-hljs">{hljs_js}</script>
<script>
(function(){{
  var ICON_EDIT = '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/></svg>';
  var ICON_VIEW = '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>';
  var ICON_PRINT = '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 6 2 18 2 18 9"/><path d="M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2"/><rect x="6" y="14" width="12" height="8"/></svg>';
  var L_EDIT = '{btn_edit}', L_VIEW = '{btn_preview}';

  var btnToggle = document.getElementById('btn-toggle');
  var btnPrint = document.getElementById('btn-print');
  var ta = document.getElementById('editor');
  var dirty = false;

  btnToggle.innerHTML = ICON_EDIT;
  btnPrint.innerHTML = ICON_PRINT;

  function inEdit() {{ return document.body.classList.contains('editing'); }}
  function setDirty(d) {{
    if (dirty === d) return;
    dirty = d;
    window.ipc.postMessage(d ? 'dirty:1' : 'dirty:0');
  }}
  function save() {{
    window.ipc.postMessage('save:' + ta.value);
    setDirty(false);
  }}
  function enterEdit() {{
    document.body.classList.add('editing');
    btnToggle.innerHTML = ICON_VIEW;
    btnToggle.title = L_VIEW;
    btnToggle.setAttribute('aria-label', L_VIEW);
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
    setTimeout(function(){{ window.print(); }}, 0);
  }});
  ta.addEventListener('input', function() {{ setDirty(true); }});

  document.addEventListener('keydown', function(e) {{
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
    if (e.key === 'Escape' && inEdit()) {{ leaveEdit(); }}
  }});

  // Called by Rust after a save (only preview is refreshed) or after an
  // external file change (both preview + textarea are refreshed).
  window.__setPreview = function(previewHtml) {{
    document.getElementById('preview').innerHTML = previewHtml;
    (window.requestIdleCallback || function(fn){{ return setTimeout(fn, 0); }})(function() {{
      if (typeof hljs !== 'undefined') hljs.highlightAll();
    }});
  }};
  window.__setContent = function(previewHtml, rawMd) {{
    document.body.classList.remove('empty');
    window.__setPreview(previewHtml);
    if (!inEdit() || !dirty) {{
      ta.value = rawMd;
      setDirty(false);
    }}
  }};

  // Defer hljs parse + initial highlight to idle time.
  var run = function(){{
    var src = document.getElementById('hljs-src').textContent;
    (new Function(src))();
    if (typeof hljs !== 'undefined') hljs.highlightAll();
  }};
  (window.requestIdleCallback || function(fn){{ return setTimeout(fn, 0); }})(run);
}})();
</script>
</body></html>"#,
        css_light = HLJS_LIGHT,
        css_dark = HLJS_DARK,
        hljs_js = HLJS_JS,
        preview_html = preview_html,
        raw_md_escaped = html_escape_ta(raw_md),
        btn_edit = s.btn_edit,
        btn_preview = s.btn_preview,
        btn_print = s.btn_print,
        body_class = body_class,
    )
}

fn escape_js(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn load_and_render(path: &PathBuf, s: &Strings) -> Option<String> {
    fs::read_to_string(path).ok().map(|raw| {
        let html_body = md_to_html(&raw);
        build_page(&html_body, &raw, s, false)
    })
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
    use winreg::RegKey;
    use winreg::enums::HKEY_CURRENT_USER;

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

fn main() {
    let lang = detect_lang();
    let strings = Strings::for_lang(lang);
    register_as_default(lang);

    // CLI: md-preview [file.md]
    let initial_file: Option<PathBuf> = std::env::args().nth(1).map(PathBuf::from).and_then(|p| {
        let p = if p.is_relative() {
            std::env::current_dir().unwrap_or_default().join(p)
        } else {
            p
        };
        if p.exists() { Some(p) } else {
            eprintln!("File not found: {}", p.display());
            None
        }
    });

    let event_loop: EventLoop<UserEvent> = EventLoopBuilder::with_user_event().build();
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

    let initial_page = match &initial_file {
        Some(path) => load_and_render(path, &strings).unwrap_or_else(|| {
            build_page(
                &format!(
                    r#"<div class="empty"><div class="icon">#</div>{}</div>"#,
                    strings.cannot_read
                ),
                "",
                &strings,
                true,
            )
        }),
        None => build_page(
            &format!(
                r#"<div class="empty"><div class="icon">#</div>{}</div>"#,
                strings.drop_hint
            ),
            "",
            &strings,
            true,
        ),
    };

    let file_path: Arc<Mutex<Option<PathBuf>>> = Arc::new(Mutex::new(initial_file));
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
        })
        ;

    let webview = builder.build(&window).expect("failed to build webview");

    // File watcher state
    let watcher_holder: Arc<Mutex<Option<notify::RecommendedWatcher>>> =
        Arc::new(Mutex::new(None));
    let file_path_for_event = Arc::clone(&file_path);
    let watcher_for_event = Arc::clone(&watcher_holder);

    // If opened with CLI arg, setup watcher immediately
    if file_path_for_event.lock().unwrap().is_some() {
        let proxy_init = proxy.clone();
        let last_self_write_init = Arc::clone(&last_self_write);
        let fp = file_path_for_event.lock().unwrap().clone();
        if let Some(ref path) = fp {
            if let Ok(mut watcher) =
                notify::recommended_watcher(move |res: Result<Event, _>| {
                    if let Ok(ev) = res {
                        if matches!(ev.kind, EventKind::Modify(_) | EventKind::Create(_)) {
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
                })
            {
                let _ = watcher.watch(path, RecursiveMode::NonRecursive);
                *watcher_holder.lock().unwrap() = Some(watcher);
            }
        }
    }

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            TaoEvent::UserEvent(UserEvent::FileChanged) => {
                let fp = file_path_for_event.lock().unwrap().clone();
                if let Some(ref path) = fp {
                    if let Ok(raw) = fs::read_to_string(path) {
                        let html = md_to_html(&raw);
                        let js = format!(
                            "if(window.__setContent)window.__setContent('{}', '{}');",
                            escape_js(&html),
                            escape_js(&raw)
                        );
                        let _ = webview.evaluate_script(&js);

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
                    if let Ok(mut new_watcher) =
                        notify::recommended_watcher(move |res: Result<Event, _>| {
                            if let Ok(ev) = res {
                                if matches!(
                                    ev.kind,
                                    EventKind::Modify(_) | EventKind::Create(_)
                                ) {
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
                        let _ = new_watcher.watch(path, RecursiveMode::NonRecursive);
                        *w = Some(new_watcher);
                    }
                }
            }
            TaoEvent::UserEvent(UserEvent::FileSaved) => {
                let fp = file_path_for_event.lock().unwrap().clone();
                if let Some(ref path) = fp {
                    if let Ok(raw) = fs::read_to_string(path) {
                        let html = md_to_html(&raw);
                        let js = format!(
                            "if(window.__setPreview)window.__setPreview('{}');",
                            escape_js(&html)
                        );
                        let _ = webview.evaluate_script(&js);
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
