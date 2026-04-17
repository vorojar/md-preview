#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

use notify::{Event, EventKind, RecursiveMode, Watcher};
use pulldown_cmark::{Options, Parser, html};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
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
    FileChanged,
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

#[allow(dead_code)] // register_title / register_body used only on Windows
struct Strings {
    drop_hint: &'static str,
    cannot_read: &'static str,
    register_title: &'static str,
    register_body: &'static str,
}

impl Strings {
    fn for_lang(lang: Lang) -> Self {
        match lang {
            Lang::Zh => Strings {
                drop_hint: "拖入 .md 文件 或按 Cmd/Ctrl+O 打开",
                cannot_read: "无法读取文件",
                register_title: "MD Preview",
                register_body:
                    "MD Preview 已注册为 .md / .markdown 的可选打开方式。\n\n\
                     由于 Windows 限制，应用本身无法静默设为默认打开方式。\
                     是否现在打开「设置 › 默认应用」手动关联？",
            },
            Lang::En => Strings {
                drop_hint: "Drop a .md file here or press Cmd/Ctrl+O to open",
                cannot_read: "Cannot read file",
                register_title: "MD Preview",
                register_body:
                    "MD Preview is now listed as an option for .md / .markdown files.\n\n\
                     Windows does not allow apps to silently set themselves as the default. \
                     Open Settings › Default apps now to finish the association?",
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

fn build_page(body: &str) -> String {
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
body {{
  font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Helvetica, Arial, sans-serif;
  max-width: 820px; margin: 0 auto; padding: 24px;
  line-height: 1.6; font-size: 15px;
  color: #1a1a1a; background: #fff;
}}
@media (prefers-color-scheme: dark) {{
  body {{ color: #d4d4d4; background: #1e1e1e; }}
  a {{ color: #6cb6ff; }}
  pre {{ background: #2d2d2d !important; }}
  code:not(pre code) {{ background: #2d2d2d; }}
  blockquote {{ border-color: #444; color: #999; }}
  table th {{ background: #2d2d2d; }}
  table td, table th {{ border-color: #444; }}
  hr {{ border-color: #333; }}
}}
h1,h2,h3,h4 {{ margin-top: 1.4em; }}
h1 {{ border-bottom: 1px solid #e1e4e8; padding-bottom: .3em; }}
h2 {{ border-bottom: 1px solid #e1e4e8; padding-bottom: .2em; }}
code {{ background: #f0f0f0; padding: 2px 6px; border-radius: 4px; font-size: 90%; }}
pre {{ background: #f6f8fa; padding: 16px; border-radius: 8px; overflow-x: auto; }}
pre code {{ background: none; padding: 0; font-size: 14px; }}
blockquote {{ border-left: 4px solid #ddd; margin: 0; padding: 0 1em; color: #666; }}
table {{ border-collapse: collapse; width: 100%; }}
table th, table td {{ border: 1px solid #ddd; padding: 8px 12px; text-align: left; }}
table th {{ background: #f6f8fa; font-weight: 600; }}
img {{ max-width: 100%; }}
hr {{ border: none; border-top: 1px solid #e1e4e8; margin: 2em 0; }}
a {{ color: #0969da; text-decoration: none; }}
a:hover {{ text-decoration: underline; }}
ul, ol {{ padding-left: 2em; }}
input[type="checkbox"] {{ margin-right: 6px; }}
.empty {{ display: flex; flex-direction: column; align-items: center; justify-content: center;
  height: 60vh; color: #999; font-size: 18px; gap: 12px; }}
.empty .icon {{ font-size: 48px; opacity: 0.4; }}
</style></head><body>{body}
<script id="hljs-src" type="text/x-hljs">{hljs_js}</script>
<script>
(function(){{
  // First paint is not blocked by parsing/running the 119KB hljs bundle above —
  // the <script type="text/x-hljs"> tag is treated as inert text by the browser.
  // Defer eval + highlighting to idle time so users see content immediately.
  var run = function(){{
    var src = document.getElementById('hljs-src').textContent;
    (new Function(src))();
    if (typeof hljs !== 'undefined') hljs.highlightAll();
  }};
  (window.requestIdleCallback || function(fn){{ return setTimeout(fn, 0); }})(run);
}})();
</script>
</html>"#,
        css_light = HLJS_LIGHT,
        css_dark = HLJS_DARK,
        hljs_js = HLJS_JS,
        body = body
    )
}

fn escape_js(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('\'', "\\'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
}

fn load_and_render(path: &PathBuf) -> Option<String> {
    fs::read_to_string(path).ok().map(|content| {
        let html_body = md_to_html(&content);
        build_page(&html_body)
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
fn register_as_default(lang: Lang) {
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

    // Nudge user to finish the default-app assignment. Win10+ cannot set it silently.
    let s = Strings::for_lang(lang);
    let answer = rfd::MessageDialog::new()
        .set_title(s.register_title)
        .set_description(s.register_body)
        .set_buttons(rfd::MessageButtons::YesNo)
        .show();
    if matches!(answer, rfd::MessageDialogResult::Yes) {
        let _ = open::that("ms-settings:defaultapps");
    }
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
        Some(path) => load_and_render(path).unwrap_or_else(|| {
            build_page(&format!(
                r#"<div class="empty"><div class="icon">#</div>{}</div>"#,
                strings.cannot_read
            ))
        }),
        None => build_page(&format!(
            r#"<div class="empty"><div class="icon">#</div>{}</div>"#,
            strings.drop_hint
        )),
    };

    let file_path: Arc<Mutex<Option<PathBuf>>> = Arc::new(Mutex::new(initial_file));
    let file_path_for_ipc = Arc::clone(&file_path);
    let proxy_for_ipc = proxy.clone();

    let builder = WebViewBuilder::new()
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
        .with_initialization_script(
            r#"document.addEventListener('keydown', e => {
                if ((e.metaKey || e.ctrlKey) && e.key === 'o') {
                    e.preventDefault();
                    window.ipc.postMessage('open');
                }
            });"#,
        );

    // Windows: keep WebView2's cache/cookie tree out of the exe directory.
    #[cfg(target_os = "windows")]
    let builder = {
        use wry::WebViewBuilderExtWindows;
        let data_dir = config_dir().join("WebView2");
        let _ = fs::create_dir_all(&data_dir);
        builder.with_data_directory(data_dir)
    };

    let webview = builder.build(&window).expect("failed to build webview");

    // File watcher state
    let watcher_holder: Arc<Mutex<Option<notify::RecommendedWatcher>>> =
        Arc::new(Mutex::new(None));
    let file_path_for_event = Arc::clone(&file_path);
    let watcher_for_event = Arc::clone(&watcher_holder);

    // If opened with CLI arg, setup watcher immediately
    if file_path_for_event.lock().unwrap().is_some() {
        let proxy_init = proxy.clone();
        let fp = file_path_for_event.lock().unwrap().clone();
        if let Some(ref path) = fp {
            if let Ok(mut watcher) =
                notify::recommended_watcher(move |res: Result<Event, _>| {
                    if let Ok(ev) = res {
                        if matches!(ev.kind, EventKind::Modify(_) | EventKind::Create(_)) {
                            let _ = proxy_init.send_event(UserEvent::FileChanged);
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
                    if let Ok(content) = fs::read_to_string(path) {
                        let html_body = md_to_html(&content);
                        let page = build_page(&html_body);
                        let js = format!(
                            r#"(function(){{
                                var s = document.documentElement.scrollTop || document.body.scrollTop;
                                document.documentElement.innerHTML = '{}';
                                requestAnimationFrame(function(){{
                                    document.documentElement.scrollTop = s;
                                    document.body.scrollTop = s;
                                    var idle = window.requestIdleCallback || function(fn){{ return setTimeout(fn, 0); }};
                                    idle(function(){{
                                        if (typeof hljs === 'undefined') {{
                                            var el = document.getElementById('hljs-src');
                                            if (el) (new Function(el.textContent))();
                                        }}
                                        if (typeof hljs !== 'undefined') hljs.highlightAll();
                                    }});
                                }});
                            }})()"#,
                            escape_js(&page)
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
                    if let Ok(mut new_watcher) =
                        notify::recommended_watcher(move |res: Result<Event, _>| {
                            if let Ok(ev) = res {
                                if matches!(
                                    ev.kind,
                                    EventKind::Modify(_) | EventKind::Create(_)
                                ) {
                                    let _ = proxy_clone.send_event(UserEvent::FileChanged);
                                }
                            }
                        })
                    {
                        let _ = new_watcher.watch(path, RecursiveMode::NonRecursive);
                        *w = Some(new_watcher);
                    }
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
