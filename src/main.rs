use notify::{Event, EventKind, RecursiveMode, Watcher};
use pulldown_cmark::{Options, Parser, html};
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tao::event::{Event as TaoEvent, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop, EventLoopBuilder};
use tao::window::WindowBuilder;
use wry::WebViewBuilder;

#[derive(Debug)]
enum UserEvent {
    FileChanged,
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
<script>{hljs_js}</script>
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
</style></head><body>{body}</body>
<script>hljs.highlightAll();</script>
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

/// On first launch, register as default handler for .md files (macOS)
#[cfg(target_os = "macos")]
fn register_as_default() {
    use std::process::Command;
    // Check if already registered by looking for a marker file
    let marker = dirs_hint().join(".md-preview-registered");
    if marker.exists() {
        return;
    }
    // Use swift to call LSSetDefaultRoleHandlerForContentType
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
    // Create marker
    let _ = fs::create_dir_all(marker.parent().unwrap());
    let _ = fs::write(&marker, "");
}

#[cfg(target_os = "macos")]
fn dirs_hint() -> PathBuf {
    std::env::var("HOME")
        .map(PathBuf::from)
        .unwrap_or_default()
        .join(".config/md-preview")
}

#[cfg(not(target_os = "macos"))]
fn register_as_default() {}

fn main() {
    register_as_default();

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

    let window = WindowBuilder::new()
        .with_title(&title)
        .with_inner_size(tao::dpi::LogicalSize::new(900.0, 700.0))
        .build(&event_loop)
        .expect("failed to build window");

    let initial_page = match &initial_file {
        Some(path) => load_and_render(path).unwrap_or_else(|| {
            build_page(r#"<div class="empty"><div class="icon">#</div>无法读取文件</div>"#)
        }),
        None => build_page(
            r#"<div class="empty"><div class="icon">#</div>拖入 .md 文件 或按 Cmd/Ctrl+O 打开</div>"#,
        ),
    };

    let file_path: Arc<Mutex<Option<PathBuf>>> = Arc::new(Mutex::new(initial_file));
    let file_path_for_ipc = Arc::clone(&file_path);
    let proxy_for_ipc = proxy.clone();

    let webview = WebViewBuilder::new()
        .with_html(&initial_page)
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
        )
        .build(&window)
        .expect("failed to build webview");

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
                                    if(typeof hljs!=='undefined') hljs.highlightAll();
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
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}
