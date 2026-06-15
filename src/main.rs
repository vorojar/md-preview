#![cfg_attr(
    all(target_os = "windows", not(debug_assertions)),
    windows_subsystem = "windows"
)]

use notify::{Event, RecursiveMode, Watcher};
use pulldown_cmark::{html, CowStr, Event as MdEvent, Options, Parser, Tag, TagEnd};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tao::dpi::{LogicalPosition, LogicalSize};
use tao::event::{Event as TaoEvent, WindowEvent};
use tao::event_loop::{ControlFlow, EventLoop, EventLoopBuilder, EventLoopProxy};
use tao::window::{Theme, Window, WindowBuilder};
use wry::WebViewBuilder;

const ICON_BYTES: &[u8] = include_bytes!("../assets/icon.ico");
const DEFAULT_W: f64 = 900.0;
const DEFAULT_H: f64 = 700.0;
static APP_DIRTY: AtomicBool = AtomicBool::new(false);

#[derive(Debug)]
enum UserEvent {
    OpenFile,
    FileChanged, // external change: refresh preview AND textarea
    FileSaved,   // our own save: refresh preview only, leave textarea cursor alone
    DirtyChanged(bool),
    ToggleEdit,
    ShowFind,
    Print, // route print through wry's native API (WKWebView ignores window.print())
    CheckUpdates,
    UpdateCheckResult(UpdateCheckResult),
    SetTheme(ThemeChoice),
    OpenUrl(&'static str),
    RecentChanged,
    Ready, // first paint landed: inject hljs now; if bench mode, also exit
}

#[derive(Debug)]
enum UpdateCheckResult {
    Available {
        tag: String,
        url: String,
        digest: Option<String>,
    },
    UpToDate,
    Failed,
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
enum ThemeChoice {
    #[default]
    System,
    Light,
    Dark,
}

impl ThemeChoice {
    fn as_str(self) -> &'static str {
        match self {
            ThemeChoice::System => "system",
            ThemeChoice::Light => "light",
            ThemeChoice::Dark => "dark",
        }
    }

    fn from_str(value: &str) -> Self {
        match value.trim().to_ascii_lowercase().as_str() {
            "light" => ThemeChoice::Light,
            "dark" => ThemeChoice::Dark,
            _ => ThemeChoice::System,
        }
    }

    fn tao_theme(self) -> Option<Theme> {
        match self {
            ThemeChoice::System => None,
            ThemeChoice::Light => Some(Theme::Light),
            ThemeChoice::Dark => Some(Theme::Dark),
        }
    }
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
    open_file: &'static str,
    recent_title: &'static str,
    btn_edit: &'static str,
    btn_preview: &'static str,
    btn_open: &'static str,
    btn_search: &'static str,
    btn_print: &'static str,
    btn_update: &'static str,
    search_placeholder: &'static str,
}

impl Strings {
    fn for_lang(lang: Lang) -> Self {
        match lang {
            Lang::Zh => Strings {
                drop_hint: "Drop a .md file here or press Cmd/Ctrl+O to open",
                cannot_read: "无法读取文件",
                open_file: "Open File",
                recent_title: "Recent",
                btn_edit: "编辑 (Cmd/Ctrl+E)",
                btn_preview: "预览 (Cmd/Ctrl+E)",
                btn_open: "Open File (Cmd/Ctrl+O)",
                btn_search: "搜索 (Cmd/Ctrl+F)",
                btn_print: "打印 (Cmd/Ctrl+P)",
                btn_update: "Update",
                search_placeholder: "搜索",
            },
            Lang::En => Strings {
                drop_hint: "Drop a .md file here or press Cmd/Ctrl+O to open",
                cannot_read: "Cannot read file",
                open_file: "Open File",
                recent_title: "Recent",
                btn_edit: "Edit (Cmd/Ctrl+E)",
                btn_preview: "Preview (Cmd/Ctrl+E)",
                btn_open: "Open File (Cmd/Ctrl+O)",
                btn_search: "Find (Cmd/Ctrl+F)",
                btn_print: "Print (Cmd/Ctrl+P)",
                btn_update: "Update",
                search_placeholder: "Find",
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

fn theme_path() -> PathBuf {
    config_dir().join("theme.txt")
}

fn load_theme_choice() -> ThemeChoice {
    fs::read_to_string(theme_path())
        .map(|raw| ThemeChoice::from_str(&raw))
        .unwrap_or_default()
}

fn save_theme_choice(choice: ThemeChoice) {
    let dir = config_dir();
    let _ = fs::create_dir_all(&dir);
    let _ = fs::write(dir.join("theme.txt"), choice.as_str());
}

fn show_info_dialog(title: &str, description: &str) {
    let _ = rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Info)
        .set_title(title)
        .set_description(description)
        .show();
}

fn show_warning_dialog(title: &str, description: &str) {
    let _ = rfd::MessageDialog::new()
        .set_level(rfd::MessageLevel::Warning)
        .set_title(title)
        .set_description(description)
        .show();
}

fn confirm_open_update(tag: &str) -> bool {
    matches!(
        rfd::MessageDialog::new()
            .set_level(rfd::MessageLevel::Info)
            .set_title("Update Available")
            .set_description(format!(
                "MD Preview {tag} is available. Open the release page to download it?"
            ))
            .set_buttons(rfd::MessageButtons::OkCancelCustom(
                "Open Release".to_string(),
                "Cancel".to_string(),
            ))
            .show(),
        rfd::MessageDialogResult::Custom(label) if label == "Open Release"
    )
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
        | Options::ENABLE_MATH
        | Options::ENABLE_GFM;
    let parser = Parser::new_ext(md, opts);
    let events = add_mark_highlights(add_heading_ids(parser.collect()));
    let mut html_out = String::new();
    html::push_html(&mut html_out, events.into_iter());
    html_out
}

fn add_mark_highlights<'a>(events: Vec<MdEvent<'a>>) -> Vec<MdEvent<'a>> {
    let mut out = Vec::with_capacity(events.len());

    for event in events {
        match event {
            MdEvent::Text(text) => {
                if text.contains("==") {
                    push_mark_highlight_events(text.as_ref(), &mut out);
                } else {
                    out.push(MdEvent::Text(text));
                }
            }
            _ => out.push(event),
        }
    }

    out
}

fn push_mark_highlight_events<'a>(text: &str, out: &mut Vec<MdEvent<'a>>) {
    let mut rest = text;

    while let Some(open) = rest.find("==") {
        let after_open = open + 2;
        let Some(close_rel) = rest[after_open..].find("==") else {
            break;
        };
        let close = after_open + close_rel;
        let body = &rest[after_open..close];
        if body.trim().is_empty() {
            break;
        }

        if open > 0 {
            out.push(MdEvent::Text(CowStr::Boxed(
                rest[..open].to_string().into_boxed_str(),
            )));
        }
        out.push(MdEvent::Html(CowStr::Borrowed(
            r#"<mark class="mdp-mark">"#,
        )));
        out.push(MdEvent::Text(CowStr::Boxed(
            body.to_string().into_boxed_str(),
        )));
        out.push(MdEvent::Html(CowStr::Borrowed("</mark>")));
        rest = &rest[close + 2..];
    }

    if !rest.is_empty() {
        out.push(MdEvent::Text(CowStr::Boxed(
            rest.to_string().into_boxed_str(),
        )));
    }
}

fn add_heading_ids<'a>(mut events: Vec<MdEvent<'a>>) -> Vec<MdEvent<'a>> {
    let mut seen: HashMap<String, usize> = HashMap::new();

    for i in 0..events.len() {
        let generate_id = match &events[i] {
            MdEvent::Start(Tag::Heading { id: Some(id), .. }) => {
                register_heading_id(id.as_ref(), &mut seen);
                false
            }
            MdEvent::Start(Tag::Heading { id: None, .. }) => true,
            _ => false,
        };

        if !generate_id {
            continue;
        }

        let text = collect_heading_text(&events, i);
        let base = heading_slug(&text);
        let id_value = unique_heading_id(base, &mut seen);
        if let MdEvent::Start(Tag::Heading { id, .. }) = &mut events[i] {
            *id = Some(CowStr::Boxed(id_value.into_boxed_str()));
        }
    }

    events
}

fn collect_heading_text(events: &[MdEvent<'_>], start: usize) -> String {
    let mut text = String::new();

    for event in events.iter().skip(start + 1) {
        match event {
            MdEvent::End(TagEnd::Heading(_)) => break,
            MdEvent::Text(value)
            | MdEvent::Code(value)
            | MdEvent::InlineMath(value)
            | MdEvent::DisplayMath(value) => text.push_str(value.as_ref()),
            MdEvent::SoftBreak | MdEvent::HardBreak => text.push(' '),
            _ => {}
        }
    }

    text
}

fn heading_slug(text: &str) -> String {
    let mut slug = String::new();
    let mut last_dash = false;

    for c in text.trim().chars().flat_map(char::to_lowercase) {
        if c.is_alphanumeric() || c == '_' || c == '-' {
            slug.push(c);
            last_dash = false;
        } else if c.is_whitespace() && !slug.is_empty() && !last_dash {
            slug.push('-');
            last_dash = true;
        }
    }

    while slug.ends_with('-') {
        slug.pop();
    }

    if slug.is_empty() {
        "section".to_string()
    } else {
        slug
    }
}

fn register_heading_id(id: &str, seen: &mut HashMap<String, usize>) {
    if !id.is_empty() {
        *seen.entry(id.to_string()).or_insert(0) += 1;
    }
}

fn unique_heading_id(base: String, seen: &mut HashMap<String, usize>) -> String {
    let count = seen.entry(base.clone()).or_insert(0);
    let id = if *count == 0 {
        base
    } else {
        format!("{base}-{count}")
    };
    *count += 1;
    id
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
const MAX_RECENT_FILES: usize = 8;

fn html_escape_ta(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;")
}

fn html_escape_attr(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('"', "&quot;")
        .replace('<', "&lt;")
}

fn html_escape_text(s: &str) -> String {
    s.replace('&', "&amp;").replace('<', "&lt;")
}

fn recent_files_path() -> PathBuf {
    config_dir().join("recent-files.txt")
}

fn load_recent_files() -> Vec<PathBuf> {
    let Ok(txt) = fs::read_to_string(recent_files_path()) else {
        return Vec::new();
    };
    let mut files = Vec::new();
    for line in txt.lines() {
        let path = PathBuf::from(line);
        if line.is_empty() || !path.exists() || files.iter().any(|p| p == &path) {
            continue;
        }
        files.push(path);
        if files.len() == MAX_RECENT_FILES {
            break;
        }
    }
    files
}

fn save_recent_files(files: &[PathBuf]) {
    let dir = config_dir();
    let _ = fs::create_dir_all(&dir);
    let body = files
        .iter()
        .take(MAX_RECENT_FILES)
        .map(|p| p.to_string_lossy())
        .collect::<Vec<_>>()
        .join("\n");
    let _ = fs::write(dir.join("recent-files.txt"), body);
}

fn remember_recent_file(files: &Arc<Mutex<Vec<PathBuf>>>, path: &Path) {
    let mut recent = files.lock().unwrap();
    recent.retain(|p| p != path);
    recent.insert(0, path.to_path_buf());
    recent.truncate(MAX_RECENT_FILES);
    save_recent_files(&recent);
}

fn forget_recent_file(files: &Arc<Mutex<Vec<PathBuf>>>, path: &Path) -> bool {
    let mut recent = files.lock().unwrap();
    let original_len = recent.len();
    recent.retain(|p| p != path);
    if recent.len() == original_len {
        return false;
    }
    save_recent_files(&recent);
    true
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

fn empty_preview_html(s: &Strings, recent_files: &[PathBuf]) -> String {
    let empty_class = if recent_files.is_empty() {
        "empty"
    } else {
        "empty has-recent"
    };
    let mut html = format!(
        r#"<div class="{empty_class}"><div class="icon">#</div><div>{}</div><button class="empty-open" type="button" data-open-file>{}</button>"#,
        html_escape_text(s.drop_hint),
        html_escape_text(s.open_file)
    );

    if !recent_files.is_empty() {
        html.push_str(&format!(
            r#"<div class="recent"><div class="recent-title">{}</div><div class="recent-list">"#,
            html_escape_text(s.recent_title)
        ));
        for (index, path) in recent_files.iter().take(MAX_RECENT_FILES).enumerate() {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy())
                .unwrap_or_else(|| path.to_string_lossy());
            let parent = path
                .parent()
                .map(|p| p.to_string_lossy())
                .unwrap_or_default();
            html.push_str(&format!(
                r#"<button class="recent-item" type="button" data-recent-index="{index}"><span class="recent-name">{}</span><span class="recent-path">{}</span></button>"#,
                html_escape_text(&name),
                html_escape_text(&parent)
            ));
        }
        html.push_str("</div></div>");
    }

    html.push_str("</div>");
    html
}

fn build_page(
    preview_html: &str,
    raw_md: &str,
    base_href: Option<&str>,
    flags: EnhanceFlags,
    s: &Strings,
    empty: bool,
    native_updater: bool,
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
  #preview .markdown-alert-note,
  #preview .markdown-alert-tip,
  #preview .markdown-alert-important,
  #preview .markdown-alert-warning,
  #preview .markdown-alert-caution {{ background: #161b22; }}
  #preview .markdown-alert-note {{ border-color: #2f81f7; }}
  #preview .markdown-alert-tip {{ border-color: #3fb950; }}
  #preview .markdown-alert-important {{ border-color: #a371f7; }}
  #preview .markdown-alert-warning {{ border-color: #d29922; }}
  #preview .markdown-alert-caution {{ border-color: #f85149; }}
  #preview .markdown-alert-note .markdown-alert-title {{ color: #2f81f7; }}
  #preview .markdown-alert-tip .markdown-alert-title {{ color: #3fb950; }}
  #preview .markdown-alert-important .markdown-alert-title {{ color: #a371f7; }}
  #preview .markdown-alert-warning .markdown-alert-title {{ color: #d29922; }}
  #preview .markdown-alert-caution .markdown-alert-title {{ color: #f85149; }}
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
#preview .markdown-alert-note,
#preview .markdown-alert-tip,
#preview .markdown-alert-important,
#preview .markdown-alert-warning,
#preview .markdown-alert-caution {{
  margin: 1em 0;
  padding: 0.75em 1em;
  border-radius: 6px;
  color: inherit;
}}
#preview .markdown-alert-title {{
  display: flex;
  align-items: center;
  gap: .35em;
  margin: 0 0 .45em;
  font-weight: 600;
  line-height: 1.25;
}}
#preview .markdown-alert-title + p {{ margin-top: 0; }}
#preview .markdown-alert-note {{ border-color: #0969da; background: #ddf4ff; }}
#preview .markdown-alert-tip {{ border-color: #1a7f37; background: #dafbe1; }}
#preview .markdown-alert-important {{ border-color: #8250df; background: #fbefff; }}
#preview .markdown-alert-warning {{ border-color: #9a6700; background: #fff8c5; }}
#preview .markdown-alert-caution {{ border-color: #cf222e; background: #ffebe9; }}
#preview .markdown-alert-note .markdown-alert-title {{ color: #0969da; }}
#preview .markdown-alert-tip .markdown-alert-title {{ color: #1a7f37; }}
#preview .markdown-alert-important .markdown-alert-title {{ color: #8250df; }}
#preview .markdown-alert-warning .markdown-alert-title {{ color: #9a6700; }}
#preview .markdown-alert-caution .markdown-alert-title {{ color: #cf222e; }}
#preview .mdp-mark {{ border-radius: 3px; padding: 0 0.12em; background: #fff2a8; color: inherit; }}
#preview table {{ border-collapse: collapse; width: 100%; }}
#preview .mdp-table-wrap {{
  width: min(calc(100vw - 64px), 1280px);
  margin: 1em 0 1em 50%;
  transform: translateX(-50%);
  overflow-x: auto;
  -webkit-overflow-scrolling: touch;
}}
#preview .mdp-table-wrap table {{ width: max-content; min-width: 100%; }}
#preview table th, #preview table td {{ border: 1px solid #ddd; padding: 8px 12px; text-align: left; }}
#preview table th {{ background: #f6f8fa; font-weight: 600; color: #1a1a1a; white-space: nowrap; }}
#preview table td {{ min-width: 64px; max-width: 360px; vertical-align: top; overflow-wrap: break-word; }}
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
	  min-height: 60vh; color: #999; font-size: 18px; gap: 12px; text-align: center; }}
	.empty.has-recent {{
	  justify-content: flex-start;
	  min-height: calc(100vh - 48px);
	  padding: clamp(56px, 10vh, 96px) 0 40px;
	  box-sizing: border-box;
	}}
	.empty .icon {{ font-size: 48px; opacity: 0.4; }}
	.empty-open {{
	  margin-top: 6px; min-height: 40px; padding: 0 16px;
	  border: 1px solid #ddd; border-radius: 8px; background: #fff;
	  color: #1a1a1a; font: inherit; font-size: 15px; cursor: pointer;
	}}
	.empty-open:hover {{ background: #f5f5f5; color: #000; }}
	.recent {{ width: min(480px, 100%); margin-top: 16px; text-align: left; }}
	.recent-title {{ margin: 0 0 8px; padding: 0; border: 0; font-size: 11px; font-weight: 600; letter-spacing: 0; color: #b6b6b6; text-transform: uppercase; }}
	.recent-list {{ display: grid; gap: 6px; }}
	.recent-item {{
	  width: 100%; min-height: 44px; padding: 7px 10px; border: 1px solid #eee;
	  border-radius: 8px; background: #fff; color: inherit; text-align: left; cursor: pointer;
	  display: grid; gap: 1px;
	}}
	.recent-item:hover {{ background: #f7f7f7; }}
	.recent-name {{ color: #555; font-weight: 600; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }}
	.recent-path {{ color: #aaa; font-size: 12px; overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }}

/* Floating toolbar (top-right) — hover-reveal, hidden in empty state unless an update exists */
.toolbar {{
  position: fixed; top: 10px; right: 12px;
  display: flex; gap: 6px; z-index: 100;
  opacity: 0; pointer-events: none;
  transition: opacity 0.18s ease;
}}
html:hover .toolbar {{ opacity: 1; pointer-events: auto; }}
body.empty .toolbar:not(.has-update) {{ display: none !important; }}
body.empty .toolbar.has-update {{ opacity: 1; pointer-events: auto; }}
body.empty .toolbar.has-update button:not(.update-btn) {{ display: none !important; }}
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
	.toolbar .update-btn {{
	  width: auto; min-width: 76px; padding: 0 11px; grid-auto-flow: column; gap: 5px;
	  font-size: 13px; font-weight: 600; color: #0969da;
	}}
	.toolbar .update-mark {{ font-size: 17px; line-height: 1; transform: translateY(-0.5px); }}
	.findbar {{
	  position: fixed; top: 10px; left: 50%; transform: translateX(-50%);
	  display: none; align-items: center; gap: 6px; z-index: 101;
	  padding: 6px; border: 1px solid rgba(0,0,0,0.08); border-radius: 10px;
	  background: rgba(255,255,255,0.96); box-shadow: 0 8px 24px rgba(0,0,0,0.12);
	  backdrop-filter: blur(8px); -webkit-backdrop-filter: blur(8px);
	}}
	body.finding .findbar {{ display: flex; }}
	.findbar input {{
	  width: min(42vw, 320px); height: 30px; box-sizing: border-box; border: 0;
	  outline: none; background: transparent; color: inherit; font: inherit;
	}}
	.findbar span {{ min-width: 14px; color: #8c959f; text-align: center; }}
	.findbar button {{
	  width: 30px; height: 30px; padding: 0; border: 0; border-radius: 7px;
	  display: grid; place-items: center; color: #555; background: transparent; cursor: pointer;
	}}
	.findbar button:hover {{ background: #f0f0f0; color: #111; }}
	@media (prefers-color-scheme: dark) {{
	  .toolbar button {{
	    background: rgba(40,40,40,0.8);
    border-color: rgba(255,255,255,0.1);
    color: #bbb;
	  }}
	  .toolbar button:hover {{ color: #fff; background: rgba(55,55,55,1); }}
	  .toolbar .update-btn {{ color: #6cb6ff; }}
		  .empty-open {{ background: #242424; border-color: #444; color: #ddd; }}
		  .empty-open:hover {{ background: #2d2d2d; color: #fff; }}
		  .recent-name {{ color: #ddd; }}
		  .recent-item {{ background: #242424; border-color: #333; }}
		  .recent-item:hover {{ background: #2d2d2d; }}
	  .findbar {{ background: rgba(34,34,34,0.96); border-color: rgba(255,255,255,0.1); }}
	  .findbar button:hover {{ background: #333; color: #fff; }}
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
body.editing #btn-open,
body.editing #btn-search,
body.editing #btn-print {{ display: none; }}

@media print {{
  .toolbar, #editor {{ display: none !important; }}
  #preview {{ display: block !important; }}
  #app {{ max-width: none; padding: 0; }}
  #preview .mdp-table-wrap {{ width: auto; margin: 1em 0; transform: none; overflow: visible; }}
}}
	</style></head><body class="{body_class}">
	<div class="toolbar">
	  <button id="btn-open" title="{btn_open}" aria-label="{btn_open}"></button>
	  <button id="btn-search" title="{btn_search}" aria-label="{btn_search}"></button>
	  <button id="btn-toggle" title="{btn_edit}" aria-label="{btn_edit}"></button>
	  <button id="btn-print" title="{btn_print}" aria-label="{btn_print}"></button>
	  <button id="btn-update" class="update-btn" hidden title="{btn_update}" aria-label="{btn_update}"></button>
	</div>
	<div class="findbar" role="search">
	  <input id="find-input" type="search" placeholder="{search_placeholder}" aria-label="{search_placeholder}">
	  <span id="find-state"></span>
	  <button id="find-prev" title="Previous" aria-label="Previous"></button>
	  <button id="find-next" title="Next" aria-label="Next"></button>
	  <button id="find-close" title="Close" aria-label="Close"></button>
	</div>
	<div id="app">
  <div id="preview">{preview_html}</div>
  <textarea id="editor" spellcheck="false">{raw_md_escaped}</textarea>
</div>
<script>
(function(){{
	  var ICON_EDIT = '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M12 20h9"/><path d="M16.5 3.5a2.121 2.121 0 0 1 3 3L7 19l-4 1 1-4L16.5 3.5z"/></svg>';
	  var ICON_VIEW = '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z"/><circle cx="12" cy="12" r="3"/></svg>';
	  var ICON_OPEN = '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m6 14 1.45-2.9A2 2 0 0 1 9.24 10H20a2 2 0 0 1 1.94 2.5l-1.55 6A2 2 0 0 1 18.45 20H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h3.9a2 2 0 0 1 1.69.9l.81 1.2a2 2 0 0 0 1.67.9H18a2 2 0 0 1 2 2v2"/></svg>';
	  var ICON_SEARCH = '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><circle cx="11" cy="11" r="8"/><path d="m21 21-4.3-4.3"/></svg>';
	  var ICON_PRINT = '<svg viewBox="0 0 24 24" width="18" height="18" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="6 9 6 2 18 2 18 9"/><path d="M6 18H4a2 2 0 0 1-2-2v-5a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v5a2 2 0 0 1-2 2h-2"/><rect x="6" y="14" width="12" height="8"/></svg>';
	  var ICON_UP = '<svg viewBox="0 0 24 24" width="17" height="17" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m18 15-6-6-6 6"/></svg>';
	  var ICON_DOWN = '<svg viewBox="0 0 24 24" width="17" height="17" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="m6 9 6 6 6-6"/></svg>';
	  var ICON_CLOSE = '<svg viewBox="0 0 24 24" width="17" height="17" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M18 6 6 18"/><path d="m6 6 12 12"/></svg>';
	  var L_EDIT = '{btn_edit}', L_VIEW = '{btn_preview}';

	  var btnOpen = document.getElementById('btn-open');
	  var btnSearch = document.getElementById('btn-search');
	  var btnToggle = document.getElementById('btn-toggle');
	  var btnPrint = document.getElementById('btn-print');
	  var btnUpdate = document.getElementById('btn-update');
	  var findInput = document.getElementById('find-input');
	  var findState = document.getElementById('find-state');
	  var findPrev = document.getElementById('find-prev');
	  var findNext = document.getElementById('find-next');
	  var findClose = document.getElementById('find-close');
	  var ta = document.getElementById('editor');
	  var dirty = false;
	  var composingFind = false;
	  var pendingFindTimer = 0;
	  var FIND_DEBOUNCE_MS = 300;

	  btnOpen.innerHTML = ICON_OPEN;
	  btnSearch.innerHTML = ICON_SEARCH;
	  btnToggle.innerHTML = ICON_EDIT;
	  btnPrint.innerHTML = ICON_PRINT;
	  btnUpdate.innerHTML = '<span class="update-mark">↻</span><span class="update-label">{btn_update}</span>';
	  findPrev.innerHTML = ICON_UP;
	  findNext.innerHTML = ICON_DOWN;
	  findClose.innerHTML = ICON_CLOSE;

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
	  function openFile() {{
	    if (inEdit()) leaveEdit();
	    window.ipc.postMessage('open');
	  }}
	  window.__mdPreviewOpenFile = openFile;
	  function showFind() {{
	    if (document.body.classList.contains('empty')) return;
	    if (inEdit()) return;
	    document.body.classList.add('finding');
	    setTimeout(function(){{ findInput.focus(); findInput.select(); }}, 0);
	  }}
	  window.__mdPreviewShowFind = showFind;
	  function hideFind() {{
	    document.body.classList.remove('finding');
	    findInput.value = '';
	    findState.textContent = '';
	    if (pendingFindTimer) {{ clearTimeout(pendingFindTimer); pendingFindTimer = 0; }}
	    var sel = window.getSelection && window.getSelection();
	    if (sel && sel.removeAllRanges) sel.removeAllRanges();
	  }}
	  function focusFindInput(selectionStart, selectionEnd) {{
	    if (!document.body.classList.contains('finding')) return;
	    try {{
	      findInput.focus({{ preventScroll: true }});
	    }} catch (_) {{
	      findInput.focus();
	    }}
	    if (typeof selectionStart === 'number' && typeof selectionEnd === 'number') {{
	      try {{ findInput.setSelectionRange(selectionStart, selectionEnd); }} catch (_) {{}}
	    }}
	  }}
	  function restoreFindInput(selectionStart, selectionEnd) {{
	    setTimeout(function() {{ focusFindInput(selectionStart, selectionEnd); }}, 0);
	    requestAnimationFrame(function() {{
	      focusFindInput(selectionStart, selectionEnd);
	      setTimeout(function() {{ focusFindInput(selectionStart, selectionEnd); }}, 80);
	    }});
	  }}
	  function runFind(backward) {{
	    var q = findInput.value;
	    if (!q) {{ findState.textContent = ''; return; }}
	    var hadFocus = document.activeElement === findInput;
	    var selectionStart = findInput.selectionStart;
	    var selectionEnd = findInput.selectionEnd;
	    var found = window.find ? window.find(q, false, !!backward, true, false, false, false) : false;
	    findState.textContent = found ? '' : '0';
	    if (hadFocus) restoreFindInput(selectionStart, selectionEnd);
	  }}
	  function scheduleFind() {{
	    if (pendingFindTimer) clearTimeout(pendingFindTimer);
	    pendingFindTimer = setTimeout(function() {{
	      pendingFindTimer = 0;
	      if (!composingFind) runFind(false);
	    }}, FIND_DEBOUNCE_MS);
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
	    var x = window.scrollX || document.documentElement.scrollLeft || 0;
	    var y = window.scrollY || document.documentElement.scrollTop || 0;
	    document.body.classList.add('editing');
	    btnToggle.innerHTML = ICON_VIEW;
	    btnToggle.title = L_VIEW;
	    btnToggle.setAttribute('aria-label', L_VIEW);
	    autoResize();
	    try {{
	      ta.focus({{ preventScroll: true }});
	    }} catch (_) {{
	      ta.focus();
	    }}
	    window.scrollTo(x, y);
	    requestAnimationFrame(function() {{ window.scrollTo(x, y); }});
	  }}
  function leaveEdit() {{
    if (dirty) save();
    document.body.classList.remove('editing');
    btnToggle.innerHTML = ICON_EDIT;
    btnToggle.title = L_EDIT;
    btnToggle.setAttribute('aria-label', L_EDIT);
  }}
  window.__mdPreviewToggleEdit = function() {{
    if (inEdit()) leaveEdit(); else enterEdit();
  }};
  window.__mdPreviewCheckUpdates = function() {{
    if (btnUpdate) btnUpdate.click();
  }};

	  btnOpen.addEventListener('click', openFile);
	  btnSearch.addEventListener('click', showFind);
	  document.addEventListener('click', function(e) {{
	    var openBtn = e.target && e.target.closest ? e.target.closest('[data-open-file]') : null;
	    if (openBtn) {{
	      e.preventDefault();
	      openFile();
	      return;
	    }}
	    var recentBtn = e.target && e.target.closest ? e.target.closest('[data-recent-index]') : null;
	    if (recentBtn) {{
	      e.preventDefault();
	      window.ipc.postMessage('open-recent:' + recentBtn.getAttribute('data-recent-index'));
	    }}
	  }});
	  findInput.addEventListener('compositionstart', function() {{ composingFind = true; }});
	  findInput.addEventListener('compositionend', function() {{ composingFind = false; scheduleFind(); }});
	  findInput.addEventListener('input', function(e) {{
	    if (composingFind || e.isComposing) return;
	    scheduleFind();
	  }});
	  findInput.addEventListener('keydown', function(e) {{
	    if (composingFind || e.isComposing) return;
	    if (e.key === 'Enter') {{ e.preventDefault(); runFind(e.shiftKey); }}
	    if (e.key === 'Escape') {{ e.preventDefault(); hideFind(); }}
	  }});
	  findPrev.addEventListener('click', function() {{ runFind(true); }});
	  findNext.addEventListener('click', function() {{ runFind(false); }});
	  findClose.addEventListener('click', hideFind);

	  btnToggle.addEventListener('click', function() {{
	    window.__mdPreviewToggleEdit();
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
	      openFile();
	      return;
	    }}
	    if ((e.metaKey || e.ctrlKey) && (e.key === 'f' || e.key === 'F')) {{
	      if (inEdit()) return;
	      e.preventDefault();
	      showFind();
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
	    if (e.key === 'Escape' && document.body.classList.contains('finding')) {{ hideFind(); return; }}
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
	    hideFind();
	    window.__setBaseHref(baseHref);
    window.__setPreview(previewHtml, needsMath, needsMermaid);
    if (!inEdit() || !dirty) {{
      ta.value = rawMd;
      setDirty(false);
      if (inEdit()) autoResize();
    }}
  }};
	  window.__setEmptyPreview = function(previewHtml) {{
	    document.body.classList.add('empty');
	    hideFind();
	    window.__setBaseHref('');
	    document.getElementById('preview').innerHTML = previewHtml;
	    ta.value = '';
	    setDirty(false);
	    window.scrollTo(0, 0);
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
if(window.__enhancePreview)window.__enhancePreview();
{update_check_js}
window.__mdPreviewInstallUpdateCheck({{
  currentVersion: '{app_version}',
  buttonLabel: '{btn_update_js}',
  nativeUpdater: {native_updater},
  apiUrl: 'https://api.github.com/repos/vorojar/md-preview/releases?per_page=20',
  latestUrl: 'https://github.com/vorojar/md-preview/releases/latest'
}});
{test_update_release_js}
</script>
</body></html>"#,
        css_light = HLJS_LIGHT,
        css_dark = HLJS_DARK,
        base_tag = base_tag,
        preview_html = preview_html,
        raw_md_escaped = html_escape_ta(raw_md),
        btn_open = s.btn_open,
        btn_search = s.btn_search,
        btn_edit = s.btn_edit,
        btn_preview = s.btn_preview,
        btn_print = s.btn_print,
        btn_update = s.btn_update,
        search_placeholder = s.search_placeholder,
        btn_update_js = escape_js(s.btn_update),
        app_version = update_current_version(),
        test_update_release_js = test_update_release_js(),
        native_updater = native_updater,
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
        || url.starts_with("https://github.com/vorojar/md-preview/releases/download/")
}

fn update_current_version() -> String {
    #[cfg(debug_assertions)]
    {
        if let Ok(version) = std::env::var("MD_PREVIEW_TEST_CURRENT_VERSION") {
            return version;
        }
    }

    env!("CARGO_PKG_VERSION").to_string()
}

#[derive(Debug, PartialEq, Eq)]
struct UpdateRelease {
    tag: String,
    url: String,
    digest: Option<String>,
}

fn parse_version(value: &str) -> Option<Vec<u64>> {
    let cleaned = value
        .trim()
        .trim_start_matches(['v', 'V'])
        .split(['+', '-'])
        .next()?;
    if cleaned.is_empty() {
        return None;
    }
    let mut parts = Vec::new();
    for part in cleaned.split('.') {
        if part.is_empty() || !part.chars().all(|ch| ch.is_ascii_digit()) {
            return None;
        }
        parts.push(part.parse().ok()?);
    }
    Some(parts)
}

fn is_newer_version(candidate: &str, current: &str) -> bool {
    let Some(next) = parse_version(candidate) else {
        return false;
    };
    let Some(now) = parse_version(current) else {
        return false;
    };
    let len = next.len().max(now.len());
    for index in 0..len {
        let a = *next.get(index).unwrap_or(&0);
        let b = *now.get(index).unwrap_or(&0);
        if a > b {
            return true;
        }
        if a < b {
            return false;
        }
    }
    false
}

fn is_desktop_release_tag(tag: &str) -> bool {
    let Some(version) = tag.trim().strip_prefix('v') else {
        return false;
    };
    version.contains('.')
        && version.chars().all(|ch| ch.is_ascii_digit() || ch == '.')
        && parse_version(tag).is_some()
}

fn preferred_update_asset_name() -> &'static str {
    if cfg!(target_os = "macos") {
        "MD-Preview-macOS-universal.dmg"
    } else if cfg!(target_os = "windows") {
        "MD-Preview-windows-x64.exe"
    } else {
        "MD-Preview-linux-x64.tar.gz"
    }
}

fn select_update_release(payload: &str, current_version: &str) -> Option<UpdateRelease> {
    let releases: serde_json::Value = serde_json::from_str(payload).ok()?;
    let releases = releases.as_array()?;
    let asset_name = preferred_update_asset_name();
    let mut best: Option<UpdateRelease> = None;

    for release in releases {
        if release
            .get("draft")
            .and_then(serde_json::Value::as_bool)
            .unwrap_or(false)
            || release
                .get("prerelease")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false)
        {
            continue;
        }
        let Some(tag) = release.get("tag_name").and_then(serde_json::Value::as_str) else {
            continue;
        };
        if !is_desktop_release_tag(tag) || !is_newer_version(tag, current_version) {
            continue;
        }

        let html_url = release
            .get("html_url")
            .and_then(serde_json::Value::as_str)
            .unwrap_or(GITHUB_URL);
        let mut url = html_url;
        let mut digest = None;
        if let Some(assets) = release.get("assets").and_then(serde_json::Value::as_array) {
            for asset in assets {
                if asset.get("name").and_then(serde_json::Value::as_str) == Some(asset_name) {
                    if let Some(download_url) = asset
                        .get("browser_download_url")
                        .and_then(serde_json::Value::as_str)
                    {
                        url = download_url;
                    }
                    digest = asset
                        .get("digest")
                        .and_then(serde_json::Value::as_str)
                        .map(str::to_string);
                    break;
                }
            }
        }

        let candidate = UpdateRelease {
            tag: tag.to_string(),
            url: url.to_string(),
            digest,
        };
        if best
            .as_ref()
            .map(|current| is_newer_version(&candidate.tag, &current.tag))
            .unwrap_or(true)
        {
            best = Some(candidate);
        }
    }

    best
}

fn check_github_updates() -> UpdateCheckResult {
    let output = std::process::Command::new("curl")
        .args([
            "-fsSL",
            "--connect-timeout",
            "5",
            "--max-time",
            "10",
            "-H",
            "Accept: application/vnd.github+json",
            "https://api.github.com/repos/vorojar/md-preview/releases?per_page=20",
        ])
        .output();
    let Ok(output) = output else {
        return UpdateCheckResult::Failed;
    };
    if !output.status.success() {
        return UpdateCheckResult::Failed;
    }
    let Ok(payload) = String::from_utf8(output.stdout) else {
        return UpdateCheckResult::Failed;
    };
    match select_update_release(&payload, env!("CARGO_PKG_VERSION")) {
        Some(release) => UpdateCheckResult::Available {
            tag: release.tag,
            url: release.url,
            digest: release.digest,
        },
        None => UpdateCheckResult::UpToDate,
    }
}

fn test_update_release_js() -> String {
    #[cfg(debug_assertions)]
    {
        let Ok(tag) = std::env::var("MD_PREVIEW_TEST_UPDATE_TAG") else {
            return String::new();
        };
        let tag = tag.trim();
        if tag.is_empty() {
            return String::new();
        }
        let escaped_tag = escape_js(tag);
        return format!(
            r#"if(window.__mdPreviewApplyUpdateRelease)window.__mdPreviewApplyUpdateRelease({{
  tag_name: '{tag}',
  html_url: 'https://github.com/vorojar/md-preview/releases/tag/{tag}',
  download_url: 'https://github.com/vorojar/md-preview/releases/download/{tag}/MD-Preview-macOS-universal.dmg'
}});"#,
            tag = escaped_tag
        );
    }

    #[cfg(not(debug_assertions))]
    {
        String::new()
    }
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
    fn github_alerts_render_as_markdown_alert_blockquotes() {
        let html = md_to_html("> [!IMPORTANT]\n> This is an alert");

        assert!(html.contains(r#"<blockquote class="markdown-alert-important">"#));
        assert!(html.contains("<p>This is an alert</p>"));
        assert!(!html.contains("[!IMPORTANT]"));
    }

    #[test]
    fn double_equals_highlight_renders_mark_without_touching_code() {
        let html = md_to_html("Use ==highlight & tag== here and `==literal==` there.");

        assert!(html.contains(r#"Use <mark class="mdp-mark">highlight &amp; tag</mark> here"#));
        assert!(html.contains("<code>==literal==</code>"));
    }

    #[test]
    fn linux_nvidia_compat_env_only_sets_dmabuf_when_unconfigured() {
        assert_eq!(
            linux_webkit_compat_env(None, None, true),
            Some(("WEBKIT_DISABLE_DMABUF_RENDERER", "1"))
        );
        assert_eq!(linux_webkit_compat_env(Some("0"), None, true), None);
        assert_eq!(linux_webkit_compat_env(None, Some("1"), true), None);
        assert_eq!(linux_webkit_compat_env(None, None, false), None);
    }

    #[test]
    fn generated_heading_ids_support_cjk_anchor_links() {
        let html = md_to_html("1. [需求概述](#需求概述)\n\n## 需求概述");

        assert!(html.contains(r##"<a href="#%E9%9C%80%E6%B1%82%E6%A6%82%E8%BF%B0">需求概述</a>"##));
        assert!(html.contains(r#"<h2 id="需求概述">需求概述</h2>"#));
    }

    #[test]
    fn generated_heading_ids_are_unique_and_keep_explicit_ids() {
        let html = md_to_html("## Intro\n## Intro\n## Custom {#fixed}\n## Fixed");

        assert!(html.contains(r#"<h2 id="intro">Intro</h2>"#));
        assert!(html.contains(r#"<h2 id="intro-1">Intro</h2>"#));
        assert!(html.contains(r#"<h2 id="fixed">Custom</h2>"#));
        assert!(html.contains(r#"<h2 id="fixed-1">Fixed</h2>"#));
    }

    #[test]
    fn help_flags_are_recognized() {
        assert!(is_help_arg("-h"));
        assert!(is_help_arg("--help"));
        assert!(!is_help_arg("--edit"));
    }

    #[test]
    fn theme_choice_parses_menu_values() {
        assert_eq!(ThemeChoice::from_str("system"), ThemeChoice::System);
        assert_eq!(ThemeChoice::from_str("light"), ThemeChoice::Light);
        assert_eq!(ThemeChoice::from_str("dark"), ThemeChoice::Dark);
        assert_eq!(ThemeChoice::from_str("unexpected"), ThemeChoice::System);
        assert_eq!(ThemeChoice::Dark.as_str(), "dark");
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
            true,
        );

        assert!(page.contains("document.addEventListener('contextmenu'"));
        assert!(page.contains("window.ipc.postMessage('refresh')"));
        assert!(page.contains("id=\"btn-open\""));
        assert!(page.contains("id=\"btn-search\""));
        assert!(page.contains("window.ipc.postMessage('open')"));
        assert!(page.contains("window.__mdPreviewOpenFile = openFile"));
        assert!(page.contains("window.__mdPreviewShowFind = showFind"));
        assert!(page.contains("window.__mdPreviewToggleEdit"));
        assert!(page.contains("window.__mdPreviewCheckUpdates"));
        assert!(page.contains("update-check-result:available"));
        assert!(page.contains("update-check-result:"));
        assert!(page.contains("Cmd/Ctrl+F"));
        assert!(page.contains("if (inEdit()) return;\n\t      e.preventDefault();\n\t      showFind();"));
        assert!(page.contains("body.editing #btn-open"));
        assert!(page.contains("ta.focus({ preventScroll: true })"));
        assert!(page.contains("window.__setEmptyPreview"));
        assert!(page.contains("releases?per_page=20"));
        assert!(page.contains("nativeUpdater: true"));
        assert!(page.contains("compositionstart"));
        assert!(page.contains("e.isComposing"));
        assert!(page.contains("focusFindInput"));
        assert!(page.contains("FIND_DEBOUNCE_MS = 300"));
        assert!(page.contains("restoreFindInput(selectionStart, selectionEnd)"));
        assert!(page.contains("findInput.setSelectionRange(selectionStart, selectionEnd)"));
        assert!(page.contains("body.empty .toolbar.has-update"));
        assert!(page.contains("bindAnchorNavigation"));
        assert!(page.contains("event.target.closest('#preview a[href]')"));
        assert!(page.contains(".markdown-alert-important"));
        assert!(page.contains(".markdown-alert-title"));
        assert!(page.contains("background: #161b22"));
        assert!(page.contains(".mdp-mark"));
        assert!(!page.contains("Local-first Markdown preview for AI-generated docs"));
    }

    #[test]
    fn page_expands_multi_column_tables() {
        let strings = Strings::for_lang(Lang::En);
        let page = build_page(
            &md_to_html("| A | B | C | D |\n|---|---|---|---|\n| 1 | 2 | 3 | 4 |"),
            "",
            None,
            EnhanceFlags::default(),
            &strings,
            false,
            false,
        );

        assert!(page.contains("mdp-table-wrap"));
        assert!(page.contains("width: min(calc(100vw - 64px), 1280px)"));
        assert!(page.contains("if(window.__enhancePreview)window.__enhancePreview();"));
        assert!(page.contains("nativeUpdater: false"));
    }

    #[test]
    fn update_download_urls_are_allowed() {
        assert!(is_allowed_update_url(
            "https://github.com/vorojar/md-preview/releases/download/v1.1.9/MD-Preview-macOS-universal.dmg"
        ));
        assert!(!is_allowed_update_url(
            "https://github.com/other/project/releases/download/v1.0.0/app.dmg"
        ));
    }

    #[test]
    fn update_versions_compare_semver_tags() {
        assert!(is_newer_version("v1.1.21", "1.1.20"));
        assert!(is_newer_version("v1.2.0", "1.1.99"));
        assert!(!is_newer_version("v1.1.20", "1.1.21"));
        assert!(!is_newer_version("v1.1.21", "1.1.21"));
        assert!(!is_newer_version("not-a-version", "1.1.21"));
    }

    #[test]
    fn update_release_selection_ignores_older_and_prerelease_versions() {
        let payload = r#"[
          {"tag_name":"v1.1.22-beta","draft":false,"prerelease":true,"html_url":"https://example.invalid/beta","assets":[]},
          {"tag_name":"v1.1.20","draft":false,"prerelease":false,"html_url":"https://example.invalid/old","assets":[]},
          {"tag_name":"v1.1.22","draft":false,"prerelease":false,"html_url":"https://example.invalid/new","assets":[
            {"name":"MD-Preview-macOS-universal.dmg","browser_download_url":"https://example.invalid/app.dmg","digest":"sha256:abc"}
          ]}
        ]"#;

        let release = select_update_release(payload, "1.1.21").unwrap();
        assert_eq!(release.tag, "v1.1.22");
        if cfg!(target_os = "macos") {
            assert_eq!(release.url, "https://example.invalid/app.dmg");
            assert_eq!(release.digest.as_deref(), Some("sha256:abc"));
        } else {
            assert_eq!(release.url, "https://example.invalid/new");
        }
        assert!(select_update_release(payload, "1.1.22").is_none());
    }

    #[test]
    fn empty_state_exposes_open_and_recent_files() {
        let strings = Strings::for_lang(Lang::En);
        let recent = vec![PathBuf::from("/tmp/example.md")];
        let html = empty_preview_html(&strings, &recent);

        assert!(html.contains("Open File"));
        assert!(html.contains("Recent"));
        assert!(html.contains("example.md"));
        assert!(html.contains("data-recent-index=\"0\""));
        assert!(html.contains(r#"<div class="empty has-recent">"#));
        assert!(html.contains(r#"<div class="icon">#</div>"#));
        assert!(!html.contains("empty-mark"));

        let page = build_page(
            &html,
            "",
            None,
            EnhanceFlags::default(),
            &strings,
            true,
            false,
        );
        assert!(page.contains(".empty.has-recent"));
        assert!(!page.contains(".empty.has-recent .recent { max-height"));
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

const WEBSITE_URL: &str = "https://vorojar.github.io/md-preview/";
const GITHUB_URL: &str = "https://github.com/vorojar/md-preview";

#[cfg(target_os = "macos")]
thread_local! {
    static MACOS_MENU_PROXY: std::cell::RefCell<Option<EventLoopProxy<UserEvent>>> =
        const { std::cell::RefCell::new(None) };
}

#[cfg(target_os = "macos")]
fn send_macos_menu_event(event: UserEvent) {
    MACOS_MENU_PROXY.with(|cell| {
        if let Some(proxy) = cell.borrow().as_ref() {
            let _ = proxy.send_event(event);
        }
    });
}

#[cfg(target_os = "macos")]
fn macos_menu_controller_class() -> &'static objc2::runtime::AnyClass {
    use objc2::runtime::{AnyClass, AnyObject, ClassBuilder, NSObject, Sel};
    use objc2::{sel, ClassType, MainThreadOnly};
    use objc2_app_kit::{
        NSAlert, NSAlertStyle, NSButton, NSControlStateValueOff, NSControlStateValueOn, NSImage,
        NSMenuItem, NSView,
    };
    use objc2_foundation::{MainThreadMarker, NSPoint, NSRect, NSSize, NSString};
    use std::sync::Once;

    extern "C" fn open_file(_: &AnyObject, _: Sel, _: &AnyObject) {
        send_macos_menu_event(UserEvent::OpenFile);
    }

    extern "C" fn show_find(_: &AnyObject, _: Sel, _: &AnyObject) {
        send_macos_menu_event(UserEvent::ShowFind);
    }

    extern "C" fn toggle_edit(_: &AnyObject, _: Sel, _: &AnyObject) {
        send_macos_menu_event(UserEvent::ToggleEdit);
    }

    extern "C" fn print(_: &AnyObject, _: Sel, _: &AnyObject) {
        send_macos_menu_event(UserEvent::Print);
    }

    extern "C" fn check_updates(_: &AnyObject, _: Sel, _: &AnyObject) {
        send_macos_menu_event(UserEvent::CheckUpdates);
    }

    extern "C" fn open_website(_: &AnyObject, _: Sel, _: &AnyObject) {
        send_macos_menu_event(UserEvent::OpenUrl(WEBSITE_URL));
    }

    extern "C" fn open_github(_: &AnyObject, _: Sel, _: &AnyObject) {
        send_macos_menu_event(UserEvent::OpenUrl(GITHUB_URL));
    }

    extern "C" fn set_theme(_: &AnyObject, _: Sel, sender: &NSMenuItem) {
        let choice = match sender.tag() {
            102 => ThemeChoice::Light,
            103 => ThemeChoice::Dark,
            _ => ThemeChoice::System,
        };
        let selected = sender.tag();
        if let Some(menu) = unsafe { sender.menu() } {
            for index in 0..menu.numberOfItems() {
                if let Some(item) = menu.itemAtIndex(index) {
                    let tag = item.tag();
                    if (101..=103).contains(&tag) {
                        item.setState(if tag == selected {
                            NSControlStateValueOn
                        } else {
                            NSControlStateValueOff
                        });
                    }
                }
            }
        }
        send_macos_menu_event(UserEvent::SetTheme(choice));
    }

    fn rect(x: f64, y: f64, width: f64, height: f64) -> NSRect {
        NSRect::new(NSPoint::new(x, y), NSSize::new(width, height))
    }

    fn symbol_button(
        symbol: &str,
        fallback_title: &str,
        tooltip: &str,
        action: Sel,
        target: &AnyObject,
        mtm: MainThreadMarker,
    ) -> objc2::rc::Retained<NSButton> {
        let accessibility = NSString::from_str(tooltip);
        let symbol_name = NSString::from_str(symbol);
        let button = if let Some(image) =
            NSImage::imageWithSystemSymbolName_accessibilityDescription(
                &symbol_name,
                Some(&accessibility),
            ) {
            unsafe {
                NSButton::buttonWithImage_target_action(&image, Some(target), Some(action), mtm)
            }
        } else {
            unsafe {
                NSButton::buttonWithTitle_target_action(
                    &NSString::from_str(fallback_title),
                    Some(target),
                    Some(action),
                    mtm,
                )
            }
        };
        button.setBordered(false);
        button.setToolTip(Some(&accessibility));
        button
    }

    extern "C" fn show_about(controller: &AnyObject, _: Sel, _: &AnyObject) {
        let Some(mtm) = MainThreadMarker::new() else {
            return;
        };

        let alert = NSAlert::new(mtm);
        alert.setAlertStyle(NSAlertStyle::Informational);
        alert.setMessageText(&NSString::from_str("MD Preview"));
        alert.setInformativeText(&NSString::from_str(&format!(
            "Version {}",
            env!("CARGO_PKG_VERSION")
        )));

        let accessory = NSView::initWithFrame(NSView::alloc(mtm), rect(0.0, 0.0, 64.0, 28.0));
        let home = symbol_button(
            "house",
            "Home",
            "Home",
            sel!(mdPreviewOpenWebsite:),
            controller,
            mtm,
        );
        home.setFrame(rect(4.0, 1.0, 26.0, 26.0));
        accessory.addSubview(&home);

        let github = symbol_button(
            "chevron.left.forwardslash.chevron.right",
            "GitHub",
            "GitHub",
            sel!(mdPreviewOpenGitHub:),
            controller,
            mtm,
        );
        github.setFrame(rect(34.0, 1.0, 26.0, 26.0));
        accessory.addSubview(&github);
        alert.setAccessoryView(Some(&accessory));
        alert.addButtonWithTitle(&NSString::from_str("OK"));

        alert.runModal();
    }

    static REGISTER_CLASS: Once = Once::new();
    REGISTER_CLASS.call_once(|| {
        let mut builder = ClassBuilder::new(c"MDPreviewMenuController", NSObject::class()).unwrap();
        unsafe {
            builder.add_method(
                sel!(mdPreviewOpenFile:),
                open_file as extern "C" fn(_, _, _),
            );
            builder.add_method(
                sel!(mdPreviewShowFind:),
                show_find as extern "C" fn(_, _, _),
            );
            builder.add_method(
                sel!(mdPreviewToggleEdit:),
                toggle_edit as extern "C" fn(_, _, _),
            );
            builder.add_method(sel!(mdPreviewPrint:), print as extern "C" fn(_, _, _));
            builder.add_method(
                sel!(mdPreviewCheckUpdates:),
                check_updates as extern "C" fn(_, _, _),
            );
            builder.add_method(
                sel!(mdPreviewOpenWebsite:),
                open_website as extern "C" fn(_, _, _),
            );
            builder.add_method(
                sel!(mdPreviewOpenGitHub:),
                open_github as extern "C" fn(_, _, _),
            );
            builder.add_method(
                sel!(mdPreviewSetTheme:),
                set_theme as extern "C" fn(_, _, _),
            );
            builder.add_method(
                sel!(mdPreviewShowAbout:),
                show_about as extern "C" fn(_, _, _),
            );
        }
        let _ = builder.register();
    });

    AnyClass::get(c"MDPreviewMenuController").unwrap()
}

#[cfg(target_os = "macos")]
fn install_macos_menu(proxy: EventLoopProxy<UserEvent>, theme: ThemeChoice) {
    use objc2::rc::Retained;
    use objc2::runtime::{AnyObject, Sel};
    use objc2::{msg_send, sel, MainThreadOnly};
    use objc2_app_kit::{
        NSApplication, NSControlStateValueOff, NSControlStateValueOn, NSEventModifierFlags, NSMenu,
        NSMenuItem,
    };
    use objc2_foundation::{MainThreadMarker, NSString};

    let Some(mtm) = MainThreadMarker::new() else {
        return;
    };
    MACOS_MENU_PROXY.with(|cell| {
        *cell.borrow_mut() = Some(proxy);
    });

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

    fn command_item(
        title: &str,
        action: Sel,
        key: &str,
        modifiers: NSEventModifierFlags,
        target: &AnyObject,
        mtm: MainThreadMarker,
    ) -> objc2::rc::Retained<NSMenuItem> {
        let item = item(title, Some(action), key, modifiers, mtm);
        unsafe {
            item.setTarget(Some(target));
        }
        item
    }

    let app = NSApplication::sharedApplication(mtm);
    let main_menu = menu("", mtm);
    let controller: Retained<AnyObject> = unsafe { msg_send![macos_menu_controller_class(), new] };
    let controller_ptr = Retained::into_raw(controller);
    let controller = unsafe { &*controller_ptr };

    let app_menu = menu("MD Preview", mtm);
    app_menu.setAutoenablesItems(false);
    app_menu.addItem(&command_item(
        "About MD Preview",
        sel!(mdPreviewShowAbout:),
        "",
        NSEventModifierFlags::empty(),
        controller,
        mtm,
    ));
    app_menu.addItem(&NSMenuItem::separatorItem(mtm));
    app_menu.addItem(&command_item(
        "MD Preview Website",
        sel!(mdPreviewOpenWebsite:),
        "",
        NSEventModifierFlags::empty(),
        controller,
        mtm,
    ));
    app_menu.addItem(&command_item(
        "GitHub Repository",
        sel!(mdPreviewOpenGitHub:),
        "",
        NSEventModifierFlags::empty(),
        controller,
        mtm,
    ));
    app_menu.addItem(&command_item(
        "Check for Updates...",
        sel!(mdPreviewCheckUpdates:),
        "",
        NSEventModifierFlags::empty(),
        controller,
        mtm,
    ));
    app_menu.addItem(&NSMenuItem::separatorItem(mtm));
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

    let file_menu = menu("File", mtm);
    file_menu.setAutoenablesItems(false);
    file_menu.addItem(&command_item(
        "Open...",
        sel!(mdPreviewOpenFile:),
        "o",
        NSEventModifierFlags::Command,
        controller,
        mtm,
    ));
    file_menu.addItem(&command_item(
        "Print...",
        sel!(mdPreviewPrint:),
        "p",
        NSEventModifierFlags::Command,
        controller,
        mtm,
    ));
    let file_menu_item = item("File", None, "", NSEventModifierFlags::empty(), mtm);
    file_menu_item.setSubmenu(Some(&file_menu));
    main_menu.addItem(&file_menu_item);

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

    let view_menu = menu("View", mtm);
    view_menu.setAutoenablesItems(false);
    view_menu.addItem(&command_item(
        "Find",
        sel!(mdPreviewShowFind:),
        "",
        NSEventModifierFlags::empty(),
        controller,
        mtm,
    ));
    view_menu.addItem(&command_item(
        "Toggle Edit Mode",
        sel!(mdPreviewToggleEdit:),
        "e",
        NSEventModifierFlags::Command,
        controller,
        mtm,
    ));
    view_menu.addItem(&NSMenuItem::separatorItem(mtm));
    let theme_menu = menu("Theme", mtm);
    theme_menu.setAutoenablesItems(false);
    for (label, choice, tag) in [
        ("System", ThemeChoice::System, 101),
        ("Light", ThemeChoice::Light, 102),
        ("Dark", ThemeChoice::Dark, 103),
    ] {
        let theme_item = command_item(
            label,
            sel!(mdPreviewSetTheme:),
            "",
            NSEventModifierFlags::empty(),
            controller,
            mtm,
        );
        theme_item.setTag(tag);
        theme_item.setState(if choice == theme {
            NSControlStateValueOn
        } else {
            NSControlStateValueOff
        });
        theme_menu.addItem(&theme_item);
    }
    let theme_menu_item = item("Theme", None, "", NSEventModifierFlags::empty(), mtm);
    theme_menu_item.setSubmenu(Some(&theme_menu));
    view_menu.addItem(&theme_menu_item);
    let view_menu_item = item("View", None, "", NSEventModifierFlags::empty(), mtm);
    view_menu_item.setSubmenu(Some(&view_menu));
    main_menu.addItem(&view_menu_item);

    app.setMainMenu(Some(&main_menu));
}

#[cfg(not(target_os = "macos"))]
fn install_macos_menu(_proxy: EventLoopProxy<UserEvent>, _theme: ThemeChoice) {}

#[cfg(target_os = "macos")]
mod macos_updater {
    use objc2::msg_send;
    use objc2::runtime::{AnyClass, AnyObject, Bool};
    use std::ffi::{c_char, c_void, CStr, CString};
    use std::path::{Path, PathBuf};
    use std::sync::OnceLock;

    static CONTROLLER: OnceLock<usize> = OnceLock::new();
    static FRAMEWORK_HANDLE: OnceLock<usize> = OnceLock::new();

    const RTLD_NOW: i32 = 0x2;
    const RTLD_GLOBAL: i32 = 0x8;

    unsafe extern "C" {
        fn dlopen(path: *const c_char, mode: i32) -> *mut c_void;
    }

    fn bundled_framework_path() -> Option<CString> {
        let exe = std::env::current_exe().ok()?;
        let path: PathBuf = exe
            .parent()?
            .join("../Frameworks/Sparkle.framework/Sparkle");
        if !path.exists() {
            return None;
        }
        CString::new(path.to_string_lossy().as_bytes()).ok()
    }

    fn app_bundle_for_exe_path(exe: &Path) -> Option<PathBuf> {
        exe.ancestors()
            .find(|path| {
                path.extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext.eq_ignore_ascii_case("app"))
                    .unwrap_or(false)
            })
            .map(Path::to_path_buf)
    }

    fn is_applications_bundle(bundle: &Path, home: Option<&Path>) -> bool {
        let Some(parent) = bundle.parent() else {
            return false;
        };
        if parent == Path::new("/Applications") {
            return true;
        }
        home.map(|home| parent == home.join("Applications"))
            .unwrap_or(false)
    }

    fn allow_non_applications_updater() -> bool {
        std::env::var("MD_PREVIEW_ALLOW_NON_APPLICATIONS_UPDATER")
            .map(|value| value == "1")
            .unwrap_or(false)
    }

    pub fn installer_enabled() -> bool {
        std::env::var("MD_PREVIEW_ENABLE_SPARKLE_INSTALLER")
            .map(|value| value == "1")
            .unwrap_or(false)
    }

    pub fn can_install_updates() -> bool {
        if !installer_enabled() {
            return false;
        }
        if bundled_framework_path().is_none() {
            return false;
        }
        if allow_non_applications_updater() {
            return true;
        }

        let Some(exe) = std::env::current_exe().ok() else {
            return false;
        };
        let Some(bundle) = app_bundle_for_exe_path(&exe) else {
            return false;
        };
        let home = std::env::var_os("HOME").map(PathBuf::from);
        is_applications_bundle(&bundle, home.as_deref())
    }

    fn load_framework() -> bool {
        if FRAMEWORK_HANDLE.get().is_some() {
            return true;
        }
        let Some(path) = bundled_framework_path() else {
            return false;
        };
        let handle = unsafe { dlopen(path.as_ptr(), RTLD_NOW | RTLD_GLOBAL) };
        if handle.is_null() {
            return false;
        }
        let _ = FRAMEWORK_HANDLE.set(handle as usize);
        true
    }

    pub fn start() -> bool {
        if CONTROLLER.get().is_some() {
            return true;
        }
        if !can_install_updates() {
            return false;
        }
        if !load_framework() {
            return false;
        }

        let Some(controller_class) =
            AnyClass::get(CStr::from_bytes_with_nul(b"SPUStandardUpdaterController\0").unwrap())
        else {
            return false;
        };

        let controller: *mut AnyObject = unsafe {
            let allocated: *mut AnyObject = msg_send![controller_class, alloc];
            msg_send![
                allocated,
                initWithStartingUpdater: Bool::YES,
                updaterDelegate: Option::<&AnyObject>::None,
                userDriverDelegate: Option::<&AnyObject>::None
            ]
        };
        if controller.is_null() {
            return false;
        }
        let _ = CONTROLLER.set(controller as usize);
        true
    }

    pub fn check_for_updates() -> bool {
        if !start() {
            return false;
        }
        let Some(ptr) = CONTROLLER.get().copied() else {
            return false;
        };
        let controller = ptr as *mut AnyObject;
        unsafe {
            let _: () = msg_send![controller, checkForUpdates: Option::<&AnyObject>::None];
        }
        true
    }

    #[cfg(test)]
    mod tests {
        use super::*;
        use std::path::Path;

        #[test]
        fn finds_bundle_from_executable_path() {
            let bundle = app_bundle_for_exe_path(Path::new(
                "/Applications/MD Preview.app/Contents/MacOS/md-preview",
            ));

            assert_eq!(bundle, Some(PathBuf::from("/Applications/MD Preview.app")));
        }

        #[test]
        fn allows_system_and_user_applications_locations() {
            assert!(is_applications_bundle(
                Path::new("/Applications/MD Preview.app"),
                None,
            ));
            assert!(is_applications_bundle(
                Path::new("/Users/me/Applications/MD Preview.app"),
                Some(Path::new("/Users/me")),
            ));
            assert!(!is_applications_bundle(
                Path::new("/Volumes/MD Preview/MD Preview.app"),
                Some(Path::new("/Users/me")),
            ));
            assert!(!is_applications_bundle(
                Path::new("/Users/me/Downloads/MD Preview.app"),
                Some(Path::new("/Users/me")),
            ));
        }
    }
}

#[cfg(target_os = "windows")]
mod windows_updater {
    use super::{config_dir, is_allowed_update_url, APP_DIRTY};
    use std::fs;
    use std::path::PathBuf;
    use std::process::Command;
    use std::sync::atomic::Ordering;

    pub fn start() -> bool {
        true
    }

    fn ps_quote(value: &str) -> String {
        format!("'{}'", value.replace('\'', "''"))
    }

    fn valid_digest(digest: &str) -> bool {
        digest
            .strip_prefix("sha256:")
            .map(|hash| hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit()))
            .unwrap_or(false)
    }

    fn relaunch_args(file: Option<PathBuf>) -> String {
        let Some(path) = file else {
            return "@()".to_string();
        };
        format!("@({})", ps_quote(&path.to_string_lossy()))
    }

    pub fn check_for_updates(
        download_url: Option<&str>,
        digest: Option<&str>,
        relaunch_file: Option<PathBuf>,
    ) -> bool {
        if APP_DIRTY.load(Ordering::SeqCst) {
            return false;
        }

        let Some(url) = download_url.filter(|url| is_allowed_update_url(url)) else {
            return false;
        };
        if !url.ends_with("/MD-Preview-windows-x64.exe") {
            return false;
        }
        let Some(expected_digest) = digest.filter(|digest| valid_digest(digest)) else {
            return false;
        };

        let Ok(target) = std::env::current_exe() else {
            return false;
        };
        let pid = std::process::id();
        let update_dir = config_dir().join("updates");
        if fs::create_dir_all(&update_dir).is_err() {
            return false;
        }
        let script_path = update_dir.join(format!("update-{pid}.ps1"));
        let script_path_s = script_path.to_string_lossy();
        let target_s = target.to_string_lossy();
        let args = relaunch_args(relaunch_file);
        let script = format!(
            r#"$ErrorActionPreference = 'Stop'
$target = {target}
$url = {url}
$expected = {expected}
$script = {script}
$pidToWait = {pid}
$tmp = Join-Path ([IO.Path]::GetTempPath()) ('md-preview-update-' + [guid]::NewGuid().ToString() + '.exe')
Invoke-WebRequest -Uri $url -OutFile $tmp -UseBasicParsing
$actual = 'sha256:' + (Get-FileHash -LiteralPath $tmp -Algorithm SHA256).Hash.ToLowerInvariant()
if ($actual -ne $expected.ToLowerInvariant()) {{
  Remove-Item -LiteralPath $tmp -Force -ErrorAction SilentlyContinue
  exit 2
}}
Wait-Process -Id $pidToWait -Timeout 30 -ErrorAction SilentlyContinue
$done = $false
for ($i = 0; $i -lt 80; $i++) {{
  try {{
    Copy-Item -LiteralPath $tmp -Destination $target -Force
    $done = $true
    break
  }} catch {{
    Start-Sleep -Milliseconds 250
  }}
}}
Remove-Item -LiteralPath $tmp -Force -ErrorAction SilentlyContinue
if (-not $done) {{ exit 3 }}
Start-Process -FilePath $target -ArgumentList {args}
Remove-Item -LiteralPath $script -Force -ErrorAction SilentlyContinue
"#,
            target = ps_quote(&target_s),
            url = ps_quote(url),
            expected = ps_quote(expected_digest),
            script = ps_quote(&script_path_s),
            pid = pid,
            args = args,
        );
        if fs::write(&script_path, script).is_err() {
            return false;
        }

        let spawned = Command::new("powershell.exe")
            .arg("-NoProfile")
            .arg("-ExecutionPolicy")
            .arg("Bypass")
            .arg("-WindowStyle")
            .arg("Hidden")
            .arg("-File")
            .arg(&script_path)
            .spawn()
            .is_ok();
        if spawned {
            std::process::exit(0);
        }
        false
    }
}

#[cfg(target_os = "macos")]
fn start_native_updater() -> bool {
    macos_updater::start()
}

#[cfg(target_os = "windows")]
fn start_native_updater() -> bool {
    windows_updater::start()
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn start_native_updater() -> bool {
    false
}

#[cfg(target_os = "macos")]
fn native_updater_enabled() -> bool {
    macos_updater::can_install_updates()
}

#[cfg(target_os = "windows")]
fn native_updater_enabled() -> bool {
    true
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn native_updater_enabled() -> bool {
    false
}

#[cfg(target_os = "macos")]
fn check_native_updates(
    _download_url: Option<&str>,
    _digest: Option<&str>,
    _relaunch_file: Option<PathBuf>,
) -> bool {
    macos_updater::check_for_updates()
}

#[cfg(target_os = "windows")]
fn check_native_updates(
    download_url: Option<&str>,
    digest: Option<&str>,
    relaunch_file: Option<PathBuf>,
) -> bool {
    windows_updater::check_for_updates(download_url, digest, relaunch_file)
}

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
fn check_native_updates(
    _download_url: Option<&str>,
    _digest: Option<&str>,
    _relaunch_file: Option<PathBuf>,
) -> bool {
    false
}

#[cfg(any(target_os = "linux", test))]
fn linux_webkit_compat_env(
    disable_dmabuf: Option<&str>,
    disable_compositing: Option<&str>,
    nvidia_driver_present: bool,
) -> Option<(&'static str, &'static str)> {
    if disable_dmabuf.is_some() || disable_compositing.is_some() || !nvidia_driver_present {
        return None;
    }

    Some(("WEBKIT_DISABLE_DMABUF_RENDERER", "1"))
}

#[cfg(target_os = "linux")]
fn apply_linux_webkit_compat_env() {
    let nvidia_driver_present = Path::new("/proc/driver/nvidia/version").exists();
    if let Some((key, value)) = linux_webkit_compat_env(
        std::env::var("WEBKIT_DISABLE_DMABUF_RENDERER")
            .ok()
            .as_deref(),
        std::env::var("WEBKIT_DISABLE_COMPOSITING_MODE")
            .ok()
            .as_deref(),
        nvidia_driver_present,
    ) {
        std::env::set_var(key, value);
    }
}

#[cfg(not(target_os = "linux"))]
fn apply_linux_webkit_compat_env() {}

fn main() {
    apply_linux_webkit_compat_env();

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
    let proxy = event_loop.create_proxy();
    let initial_theme = load_theme_choice();
    install_macos_menu(proxy.clone(), initial_theme);
    let native_updater_enabled = native_updater_enabled();

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
        .with_position(LogicalPosition::new(geom.x, geom.y))
        .with_theme(initial_theme.tao_theme());
    if let Some(icon) = load_window_icon() {
        window_builder = window_builder.with_window_icon(Some(icon));
    }
    let window = window_builder
        .build(&event_loop)
        .expect("failed to build window");
    bench_log("window_built");
    let native_updater_available = if native_updater_enabled {
        start_native_updater()
    } else {
        false
    };
    if bench && native_updater_available {
        bench_log("native_updater_started");
    }

    let recent_files: Arc<Mutex<Vec<PathBuf>>> = Arc::new(Mutex::new(load_recent_files()));
    if let Some(path) = &initial_file {
        remember_recent_file(&recent_files, path);
    }

    let mut initial_flags = EnhanceFlags::default();
    let initial_page = match &initial_file {
        Some(path) => fs::read_to_string(path).ok().map_or_else(
            || {
                build_page(
                    &format!(
                        r#"<div class="empty"><div class="icon">#</div><div>{}</div><button class="empty-open" type="button" data-open-file>{}</button></div>"#,
                        html_escape_text(strings.cannot_read),
                        html_escape_text(strings.open_file)
                    ),
                    "",
                    None,
                    EnhanceFlags::default(),
                    &strings,
                    true,
                    native_updater_enabled,
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
                    native_updater_enabled,
                )
            },
        ),
        None => build_page(
            &empty_preview_html(&strings, &recent_files.lock().unwrap()),
            "",
            None,
            EnhanceFlags::default(),
            &strings,
            true,
            native_updater_enabled,
        ),
    };

    let file_path: Arc<Mutex<Option<PathBuf>>> = Arc::new(Mutex::new(initial_file));
    let enhance_flags: Arc<Mutex<EnhanceFlags>> = Arc::new(Mutex::new(initial_flags));
    let last_self_write: Arc<Mutex<Option<Instant>>> = Arc::new(Mutex::new(None));
    let file_path_for_ipc = Arc::clone(&file_path);
    let recent_files_for_ipc = Arc::clone(&recent_files);
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
            } else if let Some(index) = body.strip_prefix("open-recent:") {
                if let Ok(index) = index.parse::<usize>() {
                    let path = recent_files_for_ipc.lock().unwrap().get(index).cloned();
                    if let Some(path) = path {
                        if path.exists() {
                            *file_path_for_ipc.lock().unwrap() = Some(path);
                            let _ = proxy_for_ipc.send_event(UserEvent::FileChanged);
                        } else if forget_recent_file(&recent_files_for_ipc, &path) {
                            let _ = proxy_for_ipc.send_event(UserEvent::RecentChanged);
                        }
                    }
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
            } else if body == "check-updates" || body.starts_with("check-updates:\n") {
                let payload = body.strip_prefix("check-updates:\n").unwrap_or("");
                let mut parts = payload.splitn(4, '\n');
                let download_url = parts.next().filter(|value| !value.is_empty());
                let digest = parts.next().filter(|value| !value.is_empty());
                let tag = parts.next().filter(|value| !value.is_empty());
                let relaunch_file = file_path_for_ipc.lock().unwrap().clone();
                if !check_native_updates(download_url, digest, relaunch_file) {
                    if let Some(url) = download_url.filter(|url| is_allowed_update_url(url)) {
                        if confirm_open_update(tag.unwrap_or("update")) {
                            let _ = open::that(url);
                        }
                    } else {
                        show_warning_dialog(
                            "Update Unavailable",
                            "MD Preview could not start the updater for this release.",
                        );
                    }
                }
            } else if let Some(result) = body.strip_prefix("update-check-result:") {
                let mut parts = result.splitn(4, '\n');
                match parts.next().unwrap_or("") {
                    "available" => {
                        let tag = parts.next().unwrap_or("update").to_string();
                        let url = parts.next().unwrap_or("").to_string();
                        let digest = parts
                            .next()
                            .filter(|value| !value.is_empty())
                            .map(str::to_string);
                        let _ = proxy_for_ipc.send_event(UserEvent::UpdateCheckResult(
                            UpdateCheckResult::Available { tag, url, digest },
                        ));
                    }
                    "none" => {
                        let _ = proxy_for_ipc
                            .send_event(UserEvent::UpdateCheckResult(UpdateCheckResult::UpToDate));
                    }
                    _ => {
                        let _ = proxy_for_ipc
                            .send_event(UserEvent::UpdateCheckResult(UpdateCheckResult::Failed));
                    }
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
            TaoEvent::UserEvent(UserEvent::OpenFile) => {
                if let Some(path) = rfd::FileDialog::new()
                    .add_filter("Markdown", &["md", "markdown", "mdown", "mkd", "txt"])
                    .pick_file()
                {
                    *file_path_for_event.lock().unwrap() = Some(path);
                    let _ = proxy.send_event(UserEvent::FileChanged);
                }
            }
            TaoEvent::UserEvent(UserEvent::FileChanged) => {
                let fp = file_path_for_event.lock().unwrap().clone();
                if let Some(ref path) = fp {
                    if let Ok(raw) = fs::read_to_string(path) {
                        remember_recent_file(&recent_files, path);
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
                APP_DIRTY.store(dirty, Ordering::SeqCst);
                let fp = file_path_for_event.lock().unwrap().clone();
                let name = fp
                    .as_ref()
                    .and_then(|p| p.file_name())
                    .map(|n| n.to_string_lossy().to_string())
                    .unwrap_or_else(|| "MD Preview".to_string());
                let prefix = if dirty { "• " } else { "" };
                window.set_title(&format!("{}{} — MD Preview", prefix, name));
            }
            TaoEvent::UserEvent(UserEvent::ToggleEdit) => {
                let _ = webview.evaluate_script(
                    "if(window.__mdPreviewToggleEdit)window.__mdPreviewToggleEdit();",
                );
            }
            TaoEvent::UserEvent(UserEvent::ShowFind) => {
                let _ = webview
                    .evaluate_script("if(window.__mdPreviewShowFind)window.__mdPreviewShowFind();");
            }
            TaoEvent::UserEvent(UserEvent::RecentChanged) => {
                let html = empty_preview_html(&strings, &recent_files.lock().unwrap());
                let js = format!(
                    "if(window.__setEmptyPreview)window.__setEmptyPreview('{}');",
                    escape_js(&html)
                );
                let _ = webview.evaluate_script(&js);
            }
            TaoEvent::UserEvent(UserEvent::Print) => {
                let _ = webview.print();
            }
            TaoEvent::UserEvent(UserEvent::CheckUpdates) => {
                let proxy = proxy.clone();
                std::thread::spawn(move || {
                    let _ = proxy.send_event(UserEvent::UpdateCheckResult(check_github_updates()));
                });
            }
            TaoEvent::UserEvent(UserEvent::UpdateCheckResult(result)) => match result {
                UpdateCheckResult::Available { tag, url, digest } => {
                    if is_allowed_update_url(&url) && confirm_open_update(&tag) {
                        let relaunch_file = file_path_for_event.lock().unwrap().clone();
                        if !check_native_updates(
                            Some(url.as_str()),
                            digest.as_deref(),
                            relaunch_file,
                        ) {
                            let _ = open::that(url);
                        }
                    }
                }
                UpdateCheckResult::UpToDate => {
                    show_info_dialog(
                        "MD Preview Is Up to Date",
                        &format!(
                            "You are using the latest version: {}.",
                            env!("CARGO_PKG_VERSION")
                        ),
                    );
                }
                UpdateCheckResult::Failed => {
                    show_warning_dialog(
                        "Could Not Check for Updates",
                        "MD Preview could not reach the update service. Please try again later.",
                    );
                }
            },
            TaoEvent::UserEvent(UserEvent::SetTheme(choice)) => {
                save_theme_choice(choice);
                window.set_theme(choice.tao_theme());
            }
            TaoEvent::UserEvent(UserEvent::OpenUrl(url)) => {
                let _ = open::that(url);
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
