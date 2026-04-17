#[cfg(target_os = "windows")]
fn main() {
    let mut res = winresource::WindowsResource::new();
    res.set_icon("assets/icon.ico");
    res.compile().expect("failed to compile Windows resources");
}

#[cfg(not(target_os = "windows"))]
fn main() {}
