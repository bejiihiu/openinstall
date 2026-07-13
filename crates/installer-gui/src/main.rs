#[cfg(target_os = "linux")]
mod app;

#[cfg(target_os = "linux")]
fn main() {
    app::run();
}

#[cfg(not(target_os = "linux"))]
fn main() {
    eprintln!("installer-gui currently only has a GTK/LibAdwaita implementation on Linux.");
}
