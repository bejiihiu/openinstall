use std::fmt;
use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::matrix::PackageSlot;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageManager {
    Apt,
    Dnf,
    Pacman,
    Zypper,
    Flatpak,
    PackageKit,
    Unknown,
}

impl fmt::Display for PackageManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            PackageManager::Apt => "apt",
            PackageManager::Dnf => "dnf",
            PackageManager::Pacman => "pacman",
            PackageManager::Zypper => "zypper",
            PackageManager::Flatpak => "flatpak",
            PackageManager::PackageKit => "packagekit",
            PackageManager::Unknown => "unknown",
        };

        f.write_str(name)
    }
}

impl PackageManager {
    pub fn is_system_package_manager(self) -> bool {
        !matches!(self, PackageManager::Flatpak | PackageManager::PackageKit | PackageManager::Unknown)
    }

    pub fn install_command(self) -> Option<&'static str> {
        match self {
            PackageManager::Apt => Some("apt-get"),
            PackageManager::Dnf => Some("dnf"),
            PackageManager::Pacman => Some("pacman"),
            PackageManager::Zypper => Some("zypper"),
            PackageManager::Flatpak => Some("flatpak"),
            PackageManager::PackageKit | PackageManager::Unknown => None,
        }
    }

    pub fn remove_command(self) -> Option<&'static str> {
        self.install_command()
    }

    pub fn query_command(self) -> Option<&'static str> {
        match self {
            PackageManager::Apt => Some("dpkg-query"),
            PackageManager::Dnf => Some("rpm"),
            PackageManager::Pacman => Some("pacman"),
            PackageManager::Zypper => Some("rpm"),
            PackageManager::Flatpak => Some("flatpak"),
            PackageManager::PackageKit | PackageManager::Unknown => None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Environment {
    pub distro: String,
    pub architecture: String,
    pub package_manager: PackageManager,
}

impl Environment {
    pub fn unknown() -> Self {
        Self {
            distro: "unknown".to_string(),
            architecture: "unknown".to_string(),
            package_manager: PackageManager::Unknown,
        }
    }

    pub fn detect() -> Self {
        let architecture = normalize_architecture(std::env::consts::ARCH);
        let os_release = read_os_release();
        let distro = os_release
            .as_ref()
            .and_then(|os_release| os_release.id.clone())
            .or_else(|| {
                os_release
                    .as_ref()
                    .and_then(|os_release| os_release.pretty_name.clone())
            })
            .unwrap_or_else(|| "unknown".to_string());
        let package_manager = detect_package_manager(os_release.as_ref());

        Self {
            distro,
            architecture,
            package_manager,
        }
    }

    pub fn from_os_release(contents: &str) -> Self {
        let os_release = OsRelease::parse(contents);
        let distro = os_release
            .id
            .clone()
            .or_else(|| os_release.pretty_name.clone())
            .unwrap_or_else(|| "unknown".to_string());
        let package_manager = detect_package_manager(Some(&os_release));

        Self {
            distro,
            architecture: normalize_architecture(std::env::consts::ARCH),
            package_manager,
        }
    }
}

#[derive(Debug, Clone)]
struct OsRelease {
    id: Option<String>,
    id_like: Vec<String>,
    pretty_name: Option<String>,
}

impl OsRelease {
    fn parse(contents: &str) -> Self {
        let mut id = None;
        let mut id_like = Vec::new();
        let mut pretty_name = None;

        for line in contents.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }

            let Some((key, raw_value)) = line.split_once('=') else {
                continue;
            };

            let value = parse_os_release_value(raw_value.trim());
            match key.trim() {
                "ID" => id = value.filter(|value| !value.is_empty()),
                "ID_LIKE" => {
                    id_like = value
                        .map(|value| {
                            value
                                .split_whitespace()
                                .map(|token| token.to_string())
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default();
                }
                "PRETTY_NAME" => pretty_name = value.filter(|value| !value.is_empty()),
                _ => {}
            }
        }

        Self {
            id,
            id_like,
            pretty_name,
        }
    }
}

fn parse_os_release_value(raw_value: &str) -> Option<String> {
    let value = raw_value.trim();
    if value.is_empty() {
        return None;
    }

    let value = value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value);

    let mut parsed = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => parsed.push('\n'),
                Some('t') => parsed.push('\t'),
                Some('"') => parsed.push('"'),
                Some('\\') => parsed.push('\\'),
                Some(other) => {
                    parsed.push(other);
                }
                None => break,
            }
        } else {
            parsed.push(ch);
        }
    }

    Some(parsed.trim().to_string())
}

fn read_os_release() -> Option<OsRelease> {
    for path in ["/etc/os-release", "/usr/lib/os-release"] {
        if let Ok(contents) = fs::read_to_string(path) {
            return Some(OsRelease::parse(&contents));
        }
    }

    None
}

fn normalize_architecture(input: &str) -> String {
    let arch = input.trim().to_lowercase();

    match arch.as_str() {
        "x86_64" | "amd64" => "x86_64".to_string(),
        "aarch64" | "arm64" => "aarch64".to_string(),
        "armv7l" | "armv7" => "armv7".to_string(),
        "armv6l" | "armv6" => "armv6".to_string(),
        "i386" | "i486" | "i586" | "i686" => "i686".to_string(),
        "" => "unknown".to_string(),
        other => other.to_string(),
    }
}

fn detect_package_manager(os_release: Option<&OsRelease>) -> PackageManager {
    if let Some(os_release) = os_release {
        for token in os_release.id.iter().chain(os_release.id_like.iter()) {
            let token = token.to_lowercase();

            if token.contains("arch") || token.contains("manjaro") || token.contains("endeavouros")
            {
                return PackageManager::Pacman;
            }

            if token.contains("ubuntu")
                || token.contains("debian")
                || token.contains("linuxmint")
                || token.contains("pop")
                || token.contains("elementary")
            {
                return PackageManager::Apt;
            }

            if token.contains("fedora")
                || token.contains("rhel")
                || token.contains("centos")
                || token.contains("rocky")
                || token.contains("almalinux")
            {
                return PackageManager::Dnf;
            }

            if token.contains("opensuse") || token.contains("suse") {
                return PackageManager::Zypper;
            }
        }
    }

    for (binary, manager) in [
        ("pacman", PackageManager::Pacman),
        ("apt-get", PackageManager::Apt),
        ("apt", PackageManager::Apt),
        ("dnf", PackageManager::Dnf),
        ("zypper", PackageManager::Zypper),
        ("pkcon", PackageManager::PackageKit),
    ] {
        if command_exists(binary) {
            return manager;
        }
    }

    PackageManager::Unknown
}

pub fn command_exists(binary: &str) -> bool {
    let path = match std::env::var_os("PATH") {
        Some(path) => path,
        None => return false,
    };

    let candidates = command_candidates(binary);
    for directory in std::env::split_paths(&path) {
        for candidate in &candidates {
            if directory.join(candidate).is_file() {
                return true;
            }
        }
    }

    false
}

fn command_candidates(binary: &str) -> Vec<String> {
    #[allow(unused_mut)]
    let mut candidates = vec![binary.to_string()];

    #[cfg(windows)]
    {
        let has_extension = Path::new(binary).extension().is_some();
        if !has_extension {
            if let Some(exts) = std::env::var_os("PATHEXT") {
                for ext in exts
                    .to_string_lossy()
                    .split(';')
                    .filter(|value| !value.is_empty())
                {
                    let ext = ext.trim_start_matches('.');
                    candidates.push(format!("{binary}.{ext}"));
                }
            }
        }
    }

    candidates
}

pub(crate) fn preferred_slot(environment: &Environment) -> PackageSlot {
    let distro = environment.distro.to_lowercase();

    if distro.contains("arch") || environment.package_manager == PackageManager::Pacman {
        return PackageSlot::Arch;
    }

    if distro.contains("ubuntu")
        || distro.contains("debian")
        || environment.package_manager == PackageManager::Apt
    {
        return PackageSlot::Ubuntu;
    }

    if distro.contains("fedora")
        || distro.contains("rhel")
        || distro.contains("centos")
        || distro.contains("rocky")
        || distro.contains("almalinux")
        || environment.package_manager == PackageManager::Dnf
    {
        return PackageSlot::Fedora;
    }

    if distro.contains("opensuse")
        || distro.contains("suse")
        || environment.package_manager == PackageManager::Zypper
    {
        return PackageSlot::OpenSuse;
    }

    if command_exists("flatpak") {
        return PackageSlot::Flatpak;
    }

    PackageSlot::AppImage
}

pub(crate) fn package_manager_for_slot(slot: PackageSlot) -> PackageManager {
    match slot {
        PackageSlot::Arch => PackageManager::Pacman,
        PackageSlot::Ubuntu => PackageManager::Apt,
        PackageSlot::Fedora => PackageManager::Dnf,
        PackageSlot::OpenSuse => PackageManager::Zypper,
        PackageSlot::Flatpak => PackageManager::Flatpak,
        PackageSlot::AppImage => PackageManager::Unknown,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_architecture_maps_x86_64() {
        assert_eq!(normalize_architecture("x86_64"), "x86_64");
        assert_eq!(normalize_architecture("amd64"), "x86_64");
    }

    #[test]
    fn normalize_architecture_maps_aarch64() {
        assert_eq!(normalize_architecture("aarch64"), "aarch64");
        assert_eq!(normalize_architecture("arm64"), "aarch64");
    }

    #[test]
    fn normalize_architecture_empty_returns_unknown() {
        assert_eq!(normalize_architecture(""), "unknown");
    }

    #[test]
    fn parses_os_release_simple_id() {
        let os = OsRelease::parse("ID=ubuntu\nPRETTY_NAME=\"Ubuntu 22.04\"\n");
        assert_eq!(os.id.as_deref(), Some("ubuntu"));
        assert_eq!(os.pretty_name.as_deref(), Some("Ubuntu 22.04"));
    }

    #[test]
    fn parses_os_release_with_id_like() {
        let os = OsRelease::parse("ID=debian\nID_LIKE=debian\nPRETTY_NAME=\"Debian GNU/Linux\"\n");
        assert_eq!(os.id_like, vec!["debian"]);
    }

    #[test]
    fn parses_os_release_skips_comments_and_empty() {
        let os = OsRelease::parse("# comment\n\nID=arch\n");
        assert_eq!(os.id.as_deref(), Some("arch"));
    }

    #[test]
    fn os_release_parse_escaped_values() {
        let os = OsRelease::parse("PRETTY_NAME=\"Ubuntu \\\"22.04\\\" LTS\"\n");
        assert_eq!(os.pretty_name.as_deref(), Some("Ubuntu \"22.04\" LTS"));
    }

    #[test]
    fn preferred_slot_arch_by_pacman() {
        let env = Environment {
            distro: "unknown".into(),
            architecture: "x86_64".into(),
            package_manager: PackageManager::Pacman,
        };
        assert_eq!(preferred_slot(&env), PackageSlot::Arch);
    }

    #[test]
    fn preferred_slot_arch_by_distro() {
        let env = Environment {
            distro: "arch linux".into(),
            architecture: "x86_64".into(),
            package_manager: PackageManager::Unknown,
        };
        assert_eq!(preferred_slot(&env), PackageSlot::Arch);
    }

    #[test]
    fn preferred_slot_ubuntu_by_apt() {
        let env = Environment {
            distro: "unknown".into(),
            architecture: "x86_64".into(),
            package_manager: PackageManager::Apt,
        };
        assert_eq!(preferred_slot(&env), PackageSlot::Ubuntu);
    }

    #[test]
    fn preferred_slot_fedora_by_dnf() {
        let env = Environment {
            distro: "unknown".into(),
            architecture: "x86_64".into(),
            package_manager: PackageManager::Dnf,
        };
        assert_eq!(preferred_slot(&env), PackageSlot::Fedora);
    }

    #[test]
    fn preferred_slot_opensuse_by_zypper() {
        let env = Environment {
            distro: "opensuse tumbleweed".into(),
            architecture: "x86_64".into(),
            package_manager: PackageManager::Zypper,
        };
        assert_eq!(preferred_slot(&env), PackageSlot::OpenSuse);
    }

    #[test]
    fn preferred_slot_appimage_by_default() {
        let env = Environment::unknown();
        assert_eq!(preferred_slot(&env), PackageSlot::AppImage);
    }

    #[test]
    fn preferred_slot_appimage_by_packagekit() {
        let env = Environment {
            distro: "unknown".into(),
            architecture: "x86_64".into(),
            package_manager: PackageManager::PackageKit,
        };
        assert_eq!(preferred_slot(&env), PackageSlot::AppImage);
    }

    #[test]
    fn preferred_slot_centos_maps_fedora() {
        let env = Environment {
            distro: "centos stream 9".into(),
            architecture: "x86_64".into(),
            package_manager: PackageManager::Unknown,
        };
        assert_eq!(preferred_slot(&env), PackageSlot::Fedora);
    }

    #[test]
    fn preferred_slot_debian_maps_ubuntu() {
        let env = Environment {
            distro: "debian bookworm".into(),
            architecture: "x86_64".into(),
            package_manager: PackageManager::Unknown,
        };
        assert_eq!(preferred_slot(&env), PackageSlot::Ubuntu);
    }

    #[test]
    fn package_manager_display_all_variants() {
        assert_eq!(format!("{}", PackageManager::Apt), "apt");
        assert_eq!(format!("{}", PackageManager::Dnf), "dnf");
        assert_eq!(format!("{}", PackageManager::Pacman), "pacman");
        assert_eq!(format!("{}", PackageManager::Zypper), "zypper");
        assert_eq!(format!("{}", PackageManager::Flatpak), "flatpak");
        assert_eq!(format!("{}", PackageManager::PackageKit), "packagekit");
        assert_eq!(format!("{}", PackageManager::Unknown), "unknown");
    }

    #[test]
    fn package_manager_is_system_pm() {
        assert!(PackageManager::Apt.is_system_package_manager());
        assert!(!PackageManager::Flatpak.is_system_package_manager());
        assert!(!PackageManager::PackageKit.is_system_package_manager());
        assert!(!PackageManager::Unknown.is_system_package_manager());
    }

    #[test]
    fn package_manager_install_commands() {
        assert_eq!(PackageManager::Apt.install_command(), Some("apt-get"));
        assert_eq!(PackageManager::Flatpak.install_command(), Some("flatpak"));
        assert!(PackageManager::PackageKit.install_command().is_none());
        assert!(PackageManager::Unknown.install_command().is_none());
    }

    #[test]
    fn package_manager_query_commands() {
        assert_eq!(PackageManager::Apt.query_command(), Some("dpkg-query"));
        assert_eq!(PackageManager::Pacman.query_command(), Some("pacman"));
        assert_eq!(PackageManager::Flatpak.query_command(), Some("flatpak"));
        assert_eq!(PackageManager::PackageKit.query_command(), None);
    }

    #[test]
    fn package_manager_for_slot_matches() {
        assert_eq!(
            package_manager_for_slot(PackageSlot::Arch),
            PackageManager::Pacman
        );
        assert_eq!(
            package_manager_for_slot(PackageSlot::Ubuntu),
            PackageManager::Apt
        );
        assert_eq!(
            package_manager_for_slot(PackageSlot::Fedora),
            PackageManager::Dnf
        );
        assert_eq!(
            package_manager_for_slot(PackageSlot::OpenSuse),
            PackageManager::Zypper
        );
        assert_eq!(
            package_manager_for_slot(PackageSlot::Flatpak),
            PackageManager::Flatpak
        );
        assert_eq!(
            package_manager_for_slot(PackageSlot::AppImage),
            PackageManager::Unknown
        );
    }

    #[test]
    fn environment_from_os_release_ubuntu() {
        let env = Environment::from_os_release("ID=ubuntu\nID_LIKE=debian\n");
        assert_eq!(env.distro, "ubuntu");
        assert_eq!(env.package_manager, PackageManager::Apt);
    }

    #[test]
    fn environment_from_os_release_arch() {
        let env = Environment::from_os_release("ID=arch\nID_LIKE=arch\n");
        assert_eq!(env.distro, "arch");
        assert_eq!(env.package_manager, PackageManager::Pacman);
    }

    #[test]
    fn detect_package_manager_falls_back_to_path() {
        let os = OsRelease::parse("ID=unknown\n");
        let pm = detect_package_manager(Some(&os));
        assert!(matches!(
            pm,
            PackageManager::Apt
                | PackageManager::Dnf
                | PackageManager::Pacman
                | PackageManager::Zypper
                | PackageManager::PackageKit
                | PackageManager::Unknown
        ));
    }
}
