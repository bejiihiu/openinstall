use std::env;
use std::path::PathBuf;
use std::process::Command;

use installer_core::{Environment, Manifest};

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();

    if args.is_empty() || args[0] == "--help" || args[0] == "-h" {
        eprintln!("Usage: installer-bootstrapper <manifest-url> [--gui|--headless]");
        std::process::exit(1);
    }

    let manifest_url = &args[0];
    let headless = args.iter().any(|a| a == "--headless");

    match run(manifest_url, headless) {
        Ok(()) => {}
        Err(e) => {
            eprintln!("Bootstrapper failed: {e}");
            std::process::exit(1);
        }
    }
}

fn run(manifest_url: &str, headless: bool) -> Result<(), String> {
    let manifest: Manifest = fetch_manifest(manifest_url)?;
    let environment = Environment::detect();
    let package = manifest
        .package_for_environment(&environment)
        .ok_or_else(|| "no package available for this environment".to_string())?;

    let cache_dir = default_cache_dir();
    std::fs::create_dir_all(&cache_dir).map_err(|e| format!("failed to create cache dir: {e}"))?;

    let dest = cache_dir.join(package_file_name(&manifest, package.reference));
    let client = reqwest::blocking::Client::builder()
        .build()
        .map_err(|e| e.to_string())?;
    let response = client
        .get(package.reference)
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("download failed: {e}"))?;
    let bytes = response.bytes().map_err(|e| format!("read failed: {e}"))?;
    std::fs::write(&dest, &bytes).map_err(|e| format!("write failed: {e}"))?;

    if headless {
        let output = Command::new("installer")
            .arg("install")
            .arg(dest.to_str().unwrap_or(""))
            .output()
            .map_err(|e| format!("installer exec failed: {e}"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!("install failed: {stderr}"));
        }
    } else {
        let (installer_bin, needs_gui_subcommand) = find_installer_gui().ok_or_else(|| {
            "installer-gui not found, use --headless or place installer-gui in PATH".to_string()
        })?;
        let mut cmd = Command::new(&installer_bin);
        if needs_gui_subcommand {
            cmd.arg("gui");
        }
        cmd.arg(dest.to_str().unwrap_or(""));
        let status = cmd
            .status()
            .map_err(|e| format!("launch installer-gui failed: {e}"))?;
        if !status.success() {
            return Err("installer-gui exited with error".to_string());
        }
    }

    Ok(())
}

fn fetch_manifest(url: &str) -> Result<Manifest, String> {
    let client = reqwest::blocking::Client::builder()
        .build()
        .map_err(|e| e.to_string())?;
    let text = client
        .get(url)
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("failed to fetch manifest: {e}"))?
        .text()
        .map_err(|e| format!("failed to read manifest: {e}"))?;
    Manifest::from_json_str(&text).map_err(|e| format!("invalid manifest: {e}"))
}

fn find_installer_gui() -> Option<(PathBuf, bool)> {
    let candidates = ["installer-gui", "installer"];
    for name in &candidates {
        if let Ok(path) = which(name) {
            return Some((path, *name == "installer"));
        }
    }
    None
}

fn which(name: &str) -> Result<PathBuf, ()> {
    let path = env::var_os("PATH").ok_or(())?;
    which_on_path(name, &path)
}

fn which_on_path(name: &str, path: &std::ffi::OsStr) -> Result<PathBuf, ()> {
    for dir in env::split_paths(path) {
        let candidate = dir.join(name);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }
    Err(())
}

fn package_file_name(_manifest: &Manifest, reference: &str) -> String {
    reference
        .rsplit('/')
        .next()
        .unwrap_or(reference)
        .split('?')
        .next()
        .unwrap_or(reference)
        .to_string()
}

fn default_cache_dir() -> PathBuf {
    default_cache_dir_for(env::var_os("XDG_CACHE_HOME"))
}

fn default_cache_dir_for(xdg: Option<impl AsRef<std::path::Path>>) -> PathBuf {
    match xdg {
        Some(cache_home) => PathBuf::from(cache_home.as_ref()).join("openinstall"),
        None => env::temp_dir().join("openinstall"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn package_file_name_extracts_from_url() {
        let manifest = Manifest::from_json_str(
            r#"{"name":"Test","publisher":"Pub","version":"1.0","description":"desc"}"#,
        )
        .unwrap();
        let name = package_file_name(&manifest, "https://example.com/app.deb");
        assert_eq!(name, "app.deb");
    }

    #[test]
    fn package_file_name_strips_query_string() {
        let manifest = Manifest::from_json_str(
            r#"{"name":"Test","publisher":"Pub","version":"1.0","description":"desc"}"#,
        )
        .unwrap();
        let name = package_file_name(&manifest, "https://example.com/app.deb?download=1");
        assert_eq!(name, "app.deb");
    }

    #[test]
    fn package_file_name_fallback() {
        let manifest = Manifest::from_json_str(
            r#"{"name":"Test","publisher":"Pub","version":"1.0","description":"desc"}"#,
        )
        .unwrap();
        let name = package_file_name(&manifest, "no-slash");
        assert_eq!(name, "no-slash");
    }

    #[test]
    fn default_cache_dir_uses_xdg() {
        let dir = default_cache_dir_for(Some("/tmp/xdg-cache"));
        assert_eq!(dir, PathBuf::from("/tmp/xdg-cache").join("openinstall"));
    }

    #[test]
    fn default_cache_dir_fallback() {
        let dir = default_cache_dir_for(None::<&std::path::Path>);
        assert!(dir.ends_with("openinstall"));
    }

    #[test]
    fn which_finds_on_path() {
        let current = std::env::current_exe().unwrap();
        let parent = current.parent().unwrap();
        let exe_name = current.file_name().unwrap().to_string_lossy().to_string();
        let result = which_on_path(&exe_name, parent.as_os_str());
        assert!(result.is_ok());
    }

    #[test]
    fn which_returns_err_for_missing() {
        let result = which("nonexistent-binary-12345");
        assert!(result.is_err());
    }
}
