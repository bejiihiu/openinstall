#[cfg(target_os = "linux")]
mod app;

#[cfg(target_os = "linux")]
mod i18n;

#[cfg(target_os = "linux")]
fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    app::run(args);
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("installer-gui currently only has a GTK/LibAdwaita implementation on Linux.");
}
