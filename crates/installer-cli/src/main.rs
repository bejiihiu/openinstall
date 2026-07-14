use std::fs;
use std::iter::Peekable;
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

use installer_core::runtime::Installer;
use installer_core::{
    desktop_entry_for_install_uri, serve_latest_app, Environment, InstallUri, InstallationState,
    Manifest, PackageMatrix, PublishSpec, SignatureSpec,
};

#[cfg(all(feature = "gui", target_os = "linux"))]
use installer_gui;

fn main() -> ExitCode {
    match run(std::env::args().skip(1)) {
        Ok(()) => ExitCode::SUCCESS,
        Err(message) => {
            eprintln!("{message}");
            ExitCode::from(1)
        }
    }
}

fn run(mut args: impl Iterator<Item = String>) -> Result<(), String> {
    let command = args.next();

    // Direct install URI as top-level command (e.g. from OS custom URI handler)
    if let Some(ref arg) = command {
        if arg.contains("://") {
            return handle_install_uri(arg);
        }
    }

    match command.as_deref() {
        None | Some("help") | Some("--help") | Some("-h") => {
            print_help();
            Ok(())
        }
        Some("detect") => {
            let environment = Environment::detect();
            println!("distro: {}", environment.distro);
            println!("architecture: {}", environment.architecture);
            println!("package manager: {}", environment.package_manager);
            Ok(())
        }
        Some("validate") => {
            let manifest = load_manifest(args.next(), "validate")?;
            manifest.validate().map_err(|error| error.to_string())?;
            println!("manifest is valid");
            Ok(())
        }
        Some("select") => {
            let manifest = load_manifest(args.next(), "select")?;
            let environment = Environment::detect();
            let selection = manifest
                .package_for_environment(&environment)
                .ok_or_else(|| {
                    "manifest does not contain a package for this environment".to_string()
                })?;
            println!(
                "environment: {} / {}",
                environment.distro, environment.package_manager
            );
            println!("selected slot: {}", selection.slot);
            println!("package manager: {}", selection.package_manager);
            println!("reference: {}", selection.reference);
            Ok(())
        }
        Some("show") => {
            let manifest = load_manifest(args.next(), "show")?;
            let environment = Environment::detect();
            let installer = Installer::default();
            print_manifest(&manifest, &environment, &installer);
            Ok(())
        }
        Some("verify") => {
            let manifest = load_manifest(args.next(), "verify")?;
            let installer = Installer::default();
            let environment = Environment::detect();
            let outcome = installer
                .verify(&manifest, &environment)
                .map_err(|error| error.to_string())?;
            println!("verified: {}", outcome.staged_path.display());
            println!("sha256 checked: {}", outcome.sha256_ok);
            println!("signature present: {}", outcome.signature_present);
            match outcome.signature_ok {
                Some(true) => println!("signature checked: yes"),
                Some(false) => println!("signature checked: no"),
                None => println!("signature checked: not provided"),
            }
            Ok(())
        }
        Some("install") => {
            let manifest = load_manifest(args.next(), "install")?;
            let installer = Installer::default();
            let environment = Environment::detect();
            let outcome = installer
                .install(&manifest, &environment)
                .map_err(|error| error.to_string())?;
            println!("install command: {}", outcome.command);
            println!("staged path: {}", outcome.staged_path.display());
            Ok(())
        }
        Some("remove") => {
            let manifest = load_manifest(args.next(), "remove")?;
            let installer = Installer::default();
            let environment = Environment::detect();
            let outcome = installer
                .remove(&manifest, &environment)
                .map_err(|error| error.to_string())?;
            println!("remove command: {}", outcome.command);
            Ok(())
        }
        Some("update") => {
            let manifest = load_manifest(args.next(), "update")?;
            let installer = Installer::default();
            let environment = Environment::detect();
            let outcome = installer
                .update(&manifest, &environment)
                .map_err(|error| error.to_string())?;
            println!("update command: {}", outcome.command);
            println!("staged path: {}", outcome.staged_path.display());
            Ok(())
        }
        Some("rollback") => {
            let manifest = load_manifest(args.next(), "rollback")?;
            let installer = Installer::default();
            let environment = Environment::detect();
            let outcome = installer
                .rollback(&manifest, &environment)
                .map_err(|error| error.to_string())?;
            println!("rollback command: {}", outcome.command);
            println!("staged path: {}", outcome.staged_path.display());
            Ok(())
        }
        Some("cache") => match args.next().as_deref() {
            Some("clear") => {
                let installer = Installer::default();
                installer.clear_cache().map_err(|error| error.to_string())?;
                println!("cache cleared");
                Ok(())
            }
            Some("info") => {
                let installer = Installer::default();
                let info = installer.cache_info().map_err(|error| error.to_string())?;
                println!("files: {}", info.file_count);
                println!("size: {} bytes", info.total_bytes);
                Ok(())
            }
            Some(other) => Err(format!("unknown cache subcommand: {other}\n")),
            None => Err("cache requires a subcommand: clear|info".to_string()),
        },
        Some("history") => {
            let installer = Installer::default();
            let entries = installer.get_history().map_err(|error| error.to_string())?;
            if entries.is_empty() {
                println!("no history for this package");
                return Ok(());
            }
            for (i, entry) in entries.iter().enumerate() {
                let instant = std::time::UNIX_EPOCH
                    + std::time::Duration::from_secs(entry.installed_at_unix_secs);
                println!(
                    "{}. {} v{} via {} at {:?}",
                    i + 1,
                    entry.package_id,
                    entry.version,
                    entry.package_manager,
                    instant,
                );
                println!("   staged: {}", entry.staged_path);
                if let Some(hash) = &entry.sha256 {
                    println!("   sha256: {hash}");
                }
            }
            Ok(())
        }
        Some("reinstall") => {
            let manifest = load_manifest(args.next(), "reinstall")?;
            let installer = Installer::default();
            let environment = Environment::detect();
            let outcome = installer
                .reinstall(&manifest, &environment)
                .map_err(|error| error.to_string())?;
            println!("reinstall command: {}", outcome.command);
            println!("staged path: {}", outcome.staged_path.display());
            Ok(())
        }
        Some("uri") => {
            let sub = args.next();
            if matches!(sub.as_deref(), Some("help") | Some("--help") | Some("-h")) {
                println!(
                    "uri commands:\n\
                     \n\
                     uri <scheme://app>                          Parse URI, print scheme + app id\n\
                     uri <scheme://app?m=manifest_url>           Parse and install from manifest URL\n\
                     uri desktop-entry <name> <path> [scheme]    Generate .desktop file\n\
                     uri register <name> <path> [scheme]         Register URI handler\n\
                     \n\
                     Supported schemes: openinstall, linuxinstall\n\
                     Query params: m=<manifest_url>, manifest=<manifest_url>\n\
                     \n\
                     Examples:\n\
                       installer uri openinstall://cursor\n\
                       installer uri openinstall://cursor?m=https://example.com/manifest.json\n\
                       installer uri desktop-entry Cursor /usr/bin/cursor\n\
                       installer uri register Cursor /usr/bin/cursor openinstall\n\
                       installer openinstall://cursor?m=https://example.com/manifest.json"
                );
                return Ok(());
            }
            match sub.as_deref() {
                Some("desktop-entry") => {
                    let app_name = next_required(
                        &mut args,
                        "uri desktop-entry requires <app_name> <exec_path>",
                    )?;
                    let exec_path = next_required(
                        &mut args,
                        "uri desktop-entry requires <app_name> <exec_path>",
                    )?;
                    let scheme = args.next().unwrap_or_else(|| "openinstall".to_string());
                    let app_id = args.next().unwrap_or_else(|| "cursor".to_string());
                    let uri = InstallUri::parse(&format!("{scheme}://{app_id}"))
                        .map_err(|error| error.to_string())?;
                    print!(
                        "{}",
                        desktop_entry_for_install_uri(&app_name, &exec_path, &uri)
                    );
                    Ok(())
                }
                Some("register") => {
                    let app_name = next_required(
                        &mut args,
                        "uri register requires <app_name> <exec_path> [scheme]",
                    )?;
                    let exec_path = next_required(
                        &mut args,
                        "uri register requires <app_name> <exec_path> [scheme]",
                    )?;
                    let scheme = args.next().unwrap_or_else(|| "openinstall".to_string());
                    let uri = InstallUri::parse(&format!("{scheme}://{app_name}"))
                        .map_err(|error| error.to_string())?;
                    let desktop_content =
                        desktop_entry_for_install_uri(&app_name, &exec_path, &uri);
                    let home = std::env::var("HOME").map_err(|_| "$HOME is not set".to_string())?;
                    let apps_dir = PathBuf::from(&home).join(".local/share/applications");
                    fs::create_dir_all(&apps_dir)
                        .map_err(|e| format!("failed to create {:?}: {e}", apps_dir))?;
                    let desktop_path = apps_dir.join(format!("{app_name}.desktop"));
                    fs::write(&desktop_path, &desktop_content)
                        .map_err(|e| format!("failed to write {:?}: {e}", desktop_path))?;
                    println!("wrote: {}", desktop_path.display());

                    if Command::new("xdg-mime").arg("--version").output().is_ok() {
                        let desktop_file = format!("{app_name}.desktop");
                        let mime_type = format!("x-scheme-handler/{scheme}");
                        if let Err(e) = Command::new("xdg-mime")
                            .args(["default", &desktop_file, &mime_type])
                            .status()
                        {
                            eprintln!("warning: xdg-mime failed: {e}");
                        }
                    }

                    if Command::new("update-desktop-database")
                        .arg("--version")
                        .output()
                        .is_ok()
                    {
                        if let Err(e) = Command::new("update-desktop-database")
                            .arg(&apps_dir)
                            .status()
                        {
                            eprintln!("warning: update-desktop-database failed: {e}");
                        }
                    }

                    Ok(())
                }
                Some(uri) => handle_install_uri(uri),
                None => Err("uri requires a linuxinstall:// or openinstall:// value".to_string()),
            }
        }
        Some("publish") => publish_command(args),
        Some("serve") => serve_command(args),
        Some("signature") => signature_command(args),
        #[cfg(all(feature = "gui", target_os = "linux"))]
        Some("gui") => gui_command(args),
        Some("self-update") => self_update_command(),
        Some(command) => Err(format!("unknown command: {command}\n")),
    }
}

fn print_manifest(manifest: &Manifest, environment: &Environment, installer: &Installer) {
    println!("name: {}", manifest.name);
    println!("publisher: {}", manifest.publisher);
    println!("version: {}", manifest.version);
    println!("description: {}", manifest.description);
    if let Some(homepage) = &manifest.homepage {
        println!("homepage: {}", homepage);
    }
    if let Some(license) = &manifest.license {
        println!("license: {}", license);
    }
    if let Some(changelog) = &manifest.changelog {
        println!("changelog: {}", changelog);
    }
    println!(
        "environment: {} / {}",
        environment.distro, environment.package_manager
    );
    match installer.inspect(manifest, environment) {
        Ok(InstallationState::NotInstalled) => println!("state: not installed"),
        Ok(InstallationState::SameVersion { version }) => {
            println!("state: already installed ({version})")
        }
        Ok(InstallationState::DifferentVersion {
            current_version,
            available_version,
        }) => {
            println!("state: update available ({current_version} -> {available_version})")
        }
        Err(error) => println!("state: unavailable ({error})"),
    }
    if let Some(selection) = manifest.package_for_environment(environment) {
        println!("selected slot: {}", selection.slot);
        println!("selected package: {}", selection.reference);
    }
}

fn handle_install_uri(uri_str: &str) -> Result<(), String> {
    let uri = InstallUri::parse(uri_str).map_err(|e| e.to_string())?;

    if let Some(manifest_url) = &uri.manifest_url {
        println!("app: {}", uri.app_id);
        println!("manifest: {}", manifest_url);
        let manifest = Manifest::from_url(manifest_url).map_err(|e| e.to_string())?;
        let installer = Installer::default();
        let environment = Environment::detect();
        let outcome = installer
            .install(&manifest, &environment)
            .map_err(|e| e.to_string())?;
        println!("install command: {}", outcome.command);
        println!("staged path: {}", outcome.staged_path.display());
        Ok(())
    } else {
        println!("scheme: {}", uri.scheme);
        println!("app id: {}", uri.app_id);
        if !uri.has_manifest() {
            println!("hint: add ?m=<manifest_url> to the URI for direct installation");
        }
        Ok(())
    }
}

fn load_manifest(path: Option<String>, command: &str) -> Result<Manifest, String> {
    let path = path.ok_or_else(|| format!("{command} requires a manifest path"))?;
    let path = PathBuf::from(path);
    Manifest::from_path(&path).map_err(|error| error.to_string())
}

fn print_help() {
    println!(
        "installer-cli\n\n\
         Commands:\n  \
         detect\n  \
         validate <manifest.json>\n  \
         select <manifest.json>\n  \
         show <manifest.json>\n  \
         verify <manifest.json>\n  \
         install <manifest.json>\n  \
         remove <manifest.json>\n  \
         update <manifest.json>\n  \
         reinstall <manifest.json>\n  \
         rollback <manifest.json>\n  \
         history\n  \
         cache clear\n  cache info\n  \
         uri <scheme://app>\n  \
         uri <scheme://app?m=manifest_url>     parse and install from manifest URL\n  \
         uri desktop-entry <app_name> <exec_path>\n  \
         uri register <app_name> <exec_path> [scheme]\n  \
         publish --name ... --publisher ... --version ... --description ... [--arch ...] [--ubuntu ...] [--fedora ...] [--opensuse ...] [--output ...]\n  \
         serve <manifest.json> [addr]\n  \
         gui [manifest]                          launch graphical installer (Linux only)\n  \
         gui --register-desktop                  add OpenInstall to application menu\n  \
         self-update                             download and replace itself\n  \
         signature verify <signature> <file>\n  \
         help\n\n  \
         <scheme://app[?m=manifest_url]>    also accepted as top-level command"
    );
}

fn publish_command(args: impl Iterator<Item = String>) -> Result<(), String> {
    let mut args = args.peekable();
    let preinstall = optional_flag(&mut args, "--preinstall");
    let postinstall = optional_flag(&mut args, "--postinstall");
    let preremove = optional_flag(&mut args, "--preremove");
    let postremove = optional_flag(&mut args, "--postremove");
    let scripts = if preinstall.is_some()
        || postinstall.is_some()
        || preremove.is_some()
        || postremove.is_some()
    {
        Some(installer_core::Scripts {
            preinstall,
            postinstall,
            preremove,
            postremove,
        })
    } else {
        None
    };

    let spec = PublishSpec {
        name: required_flag(&mut args, "--name")?,
        publisher: required_flag(&mut args, "--publisher")?,
        version: required_flag(&mut args, "--version")?,
        description: required_flag(&mut args, "--description")?,
        homepage: optional_flag(&mut args, "--homepage"),
        license: optional_flag(&mut args, "--license"),
        changelog: optional_flag(&mut args, "--changelog"),
        image: optional_flag(&mut args, "--image"),
        packages: PackageMatrix {
            arch: optional_flag(&mut args, "--arch"),
            ubuntu: optional_flag(&mut args, "--ubuntu"),
            fedora: optional_flag(&mut args, "--fedora"),
            opensuse: optional_flag(&mut args, "--opensuse"),
            flatpak: optional_flag(&mut args, "--flatpak"),
            appimage: optional_flag(&mut args, "--appimage").or(optional_flag(&mut args, "--fallback")),
            windows: optional_flag(&mut args, "--windows"),
            macos: optional_flag(&mut args, "--macos"),
        },
        sha256: optional_flag(&mut args, "--sha256"),
        signature: optional_flag(&mut args, "--signature"),
        scripts,
    };

    let output = optional_flag(&mut args, "--output").unwrap_or_else(|| "-".to_string());
    if output == "-" {
        let json =
            serde_json::to_string_pretty(&spec.to_manifest()).map_err(|error| error.to_string())?;
        println!("{json}");
        return Ok(());
    }

    spec.write_manifest(Path::new(&output))
        .map_err(|error| error.to_string())?;
    println!("manifest written: {output}");
    Ok(())
}

fn serve_command(mut args: impl Iterator<Item = String>) -> Result<(), String> {
    let manifest_path = args
        .next()
        .ok_or_else(|| "serve requires <manifest.json>".to_string())?;
    let address = args.next().unwrap_or_else(|| "127.0.0.1:3000".to_string());
    let manifest = load_manifest(Some(manifest_path), "serve")?;
    let _server = serve_latest_app(&address, &manifest).map_err(|error| error.to_string())?;
    println!("serving /app/latest on http://{address}");
    loop {
        std::thread::park();
    }
}

#[cfg(all(feature = "gui", target_os = "linux"))]
fn gui_command(args: impl Iterator<Item = String>) -> Result<(), String> {
    let args: Vec<String> = args.collect();
    if args.iter().any(|a| a == "--register-desktop") {
        return register_gui_desktop();
    }
    installer_gui::app::run();
    Ok(())
}

#[cfg(all(feature = "gui", target_os = "linux"))]
fn register_gui_desktop() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("failed to get exe path: {e}"))?;
    let desktop = format!(
        "[Desktop Entry]\n\
         Type=Application\n\
         Name=OpenInstall\n\
         Comment=Linux Application Installer\n\
         Exec={} gui %f\n\
         Icon=system-software-install\n\
         Terminal=false\n\
         Categories=Utility;\n\
         MimeType=x-scheme-handler/openinstall;x-scheme-handler/openinstaller;x-scheme-handler/linuxinstall;\n",
        exe.display()
    );
    let home = std::env::var("HOME").map_err(|_| "$HOME is not set".to_string())?;
    let apps_dir = PathBuf::from(&home).join(".local/share/applications");
    std::fs::create_dir_all(&apps_dir)
        .map_err(|e| format!("failed to create {apps_dir:?}: {e}"))?;
    let desktop_path = apps_dir.join("openinstall.desktop");
    std::fs::write(&desktop_path, &desktop)
        .map_err(|e| format!("failed to write desktop file: {e}"))?;
    println!("wrote: {}", desktop_path.display());

    if Command::new("xdg-mime").arg("--version").output().is_ok() {
        for scheme in &["openinstall", "openinstaller", "linuxinstall"] {
            let _ = Command::new("xdg-mime")
                .args([
                    "default",
                    "openinstall.desktop",
                    &format!("x-scheme-handler/{scheme}"),
                ])
                .status();
        }
    }
    if Command::new("update-desktop-database")
        .arg("--version")
        .output()
        .is_ok()
    {
        let _ = Command::new("update-desktop-database")
            .arg(&apps_dir)
            .status();
    }
    println!("OpenInstall registered in application menu");
    println!("URI schemes registered: openinstall://, openinstaller://, linuxinstall://");
    Ok(())
}

fn self_update_command() -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("failed to get exe path: {e}"))?;

    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;
    let target = match (os, arch) {
        ("linux", "x86_64") => "x86_64-unknown-linux-gnu",
        ("linux", "aarch64") => "aarch64-unknown-linux-gnu",
        _ => return Err("unsupported platform for self-update".to_string()),
    };
    let url = format!(
        "https://github.com/bejiihiu/openinstall/releases/latest/download/installer-{target}"
    );

    println!("Downloading OpenInstall {target}...");
    let client = reqwest::blocking::Client::builder()
        .build()
        .map_err(|e| format!("http client: {e}"))?;
    let response = client
        .get(&url)
        .send()
        .and_then(|r| r.error_for_status())
        .map_err(|e| format!("download failed: {e}"))?;
    let bytes = response
        .bytes()
        .map_err(|e| format!("read response: {e}"))?;

    if bytes.len() < 4096 {
        return Err("downloaded file is too small — not a valid binary".to_string());
    }

    let temp = std::env::temp_dir().join("openinstall-update");
    std::fs::write(&temp, &bytes).map_err(|e| format!("write temp: {e}"))?;

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&temp, std::fs::Permissions::from_mode(0o755))
            .map_err(|e| format!("chmod temp: {e}"))?;
    }

    std::fs::rename(&temp, &exe).map_err(|e| format!("replace binary failed: {e}"))?;

    println!("OpenInstall updated to the latest version ({})", url);
    println!("Restart to use the new version");
    Ok(())
}

fn signature_command(mut args: impl Iterator<Item = String>) -> Result<(), String> {
    match args.next().as_deref() {
        Some("verify") => {
            let signature = args
                .next()
                .ok_or_else(|| "signature verify requires <signature> <file>".to_string())?;
            let file = args
                .next()
                .ok_or_else(|| "signature verify requires <signature> <file>".to_string())?;
            let spec = SignatureSpec::parse(&signature).map_err(|error| error.to_string())?;
            spec.verify_file(Path::new(&file))
                .map_err(|error| error.to_string())?;
            println!("signature valid");
            Ok(())
        }
        Some(other) => Err(format!("unknown signature subcommand: {other}")),
        None => Err("signature requires a subcommand: verify".to_string()),
    }
}

fn required_flag(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    match args.next().as_deref() {
        Some(value) if value == flag => args
            .next()
            .ok_or_else(|| format!("{flag} requires a value")),
        Some(other) => Err(format!("expected {flag}, got {other}")),
        None => Err(format!("missing required flag {flag}")),
    }
}

fn optional_flag(args: &mut Peekable<impl Iterator<Item = String>>, flag: &str) -> Option<String> {
    if args.peek()? == flag {
        args.next();
        args.next()
    } else {
        None
    }
}

fn next_required(args: &mut impl Iterator<Item = String>, message: &str) -> Result<String, String> {
    args.next().ok_or_else(|| message.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn help_returns_ok() {
        let args = vec!["help".to_string()];
        assert!(run(args.into_iter()).is_ok());
    }

    #[test]
    fn empty_args_defaults_to_help() {
        let args: Vec<String> = vec![];
        assert!(run(args.into_iter()).is_ok());
    }

    #[test]
    fn detect_returns_ok() {
        let args = vec!["detect".to_string()];
        assert!(run(args.into_iter()).is_ok());
    }

    #[test]
    fn unknown_command_returns_err() {
        let args = vec!["nonexistent".to_string()];
        let result = run(args.into_iter());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("unknown command"));
    }

    #[test]
    fn validate_without_path_returns_err() {
        let args = vec!["validate".to_string()];
        let result = run(args.into_iter());
        assert!(result.is_err());
    }

    #[test]
    fn required_flag_returns_value() {
        let args = vec!["--name".to_string(), "test".to_string()];
        let result = required_flag(&mut args.into_iter(), "--name");
        assert_eq!(result, Ok("test".to_string()));
    }

    #[test]
    fn required_flag_missing_value_returns_err() {
        let args = vec!["--name".to_string()];
        let result = required_flag(&mut args.into_iter(), "--name");
        assert!(result.is_err());
    }

    #[test]
    fn required_flag_wrong_flag_returns_err() {
        let args = vec!["--other".to_string(), "val".to_string()];
        let result = required_flag(&mut args.into_iter(), "--name");
        assert!(result.is_err());
    }

    #[test]
    fn optional_flag_returns_some_when_present() {
        let args = vec!["--arch".to_string(), "x86_64".to_string()];
        let result = optional_flag(&mut args.into_iter().peekable(), "--arch");
        assert_eq!(result, Some("x86_64".to_string()));
    }

    #[test]
    fn optional_flag_returns_none_when_missing() {
        let args = vec!["--other".to_string()];
        let result = optional_flag(&mut args.into_iter().peekable(), "--arch");
        assert_eq!(result, None);
    }

    #[test]
    fn next_required_returns_value() {
        let args = vec!["val".to_string()];
        let result = next_required(&mut args.into_iter(), "msg");
        assert_eq!(result, Ok("val".to_string()));
    }

    #[test]
    fn next_required_empty_returns_err() {
        let args: Vec<String> = vec![];
        let result = next_required(&mut args.into_iter(), "test message");
        assert_eq!(result, Err("test message".to_string()));
    }

    #[test]
    fn cache_without_subcommand_returns_err() {
        let args = vec!["cache".to_string()];
        let result = run(args.into_iter());
        assert!(result.is_err());
    }

    #[test]
    fn uri_without_value_returns_err() {
        let args = vec!["uri".to_string()];
        let result = run(args.into_iter());
        assert!(result.is_err());
    }

    #[test]
    fn signature_without_subcommand_returns_err() {
        let args = vec!["signature".to_string()];
        let result = run(args.into_iter());
        assert!(result.is_err());
    }
}
