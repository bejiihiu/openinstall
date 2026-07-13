use std::fmt;
use std::fs;
use std::path::{Path, PathBuf};

use serde::de::{self, Deserializer};
use serde::{Deserialize, Serialize};
use thiserror::Error;

pub mod adapters;
pub mod api;
pub mod desktop;
pub mod github;
pub mod publish;
pub mod runtime;
pub mod signature;
pub mod uri;

pub use adapters::{adapter_for, PackageAdapter};
pub use api::{serve_latest_app, ApiError, LatestAppResponse};
pub use desktop::{desktop_entry, desktop_entry_for_install_uri};
pub use github::{resolve_latest_release_asset, GitHubReleaseAsset};
pub use publish::{PublishError, PublishSpec};
pub use runtime::{
    CacheInfo, HistoryEntry, InstallOutcome, InstallProgress, InstallStage, InstallationState,
    Installer, InstallerError, VerificationOutcome,
};
pub use signature::{SignatureError, SignatureSpec};
pub use uri::{InstallUri, InstallUriError};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Manifest {
    pub name: String,
    pub publisher: String,
    pub version: String,
    pub description: String,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub changelog: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub packages: PackageMatrix,
    #[serde(default)]
    pub sha256: Option<String>,
    #[serde(default)]
    pub signature: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PackageMatrix {
    #[serde(default, deserialize_with = "deserialize_package_ref")]
    pub arch: Option<String>,
    #[serde(default, deserialize_with = "deserialize_package_ref")]
    pub ubuntu: Option<String>,
    #[serde(default, deserialize_with = "deserialize_package_ref")]
    pub fedora: Option<String>,
    #[serde(default, deserialize_with = "deserialize_package_ref")]
    pub opensuse: Option<String>,
    #[serde(default, deserialize_with = "deserialize_package_ref")]
    pub fallback: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PackageManager {
    Apt,
    Dnf,
    Pacman,
    Zypper,
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
            PackageManager::PackageKit => "packagekit",
            PackageManager::Unknown => "unknown",
        };

        f.write_str(name)
    }
}

impl PackageManager {
    pub fn is_system_package_manager(self) -> bool {
        !matches!(self, PackageManager::Unknown)
    }

    pub fn install_command(self) -> Option<&'static str> {
        match self {
            PackageManager::Apt => Some("apt-get"),
            PackageManager::Dnf => Some("dnf"),
            PackageManager::Pacman => Some("pacman"),
            PackageManager::Zypper => Some("zypper"),
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
            PackageManager::PackageKit | PackageManager::Unknown => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageSlot {
    Arch,
    Ubuntu,
    Fedora,
    OpenSuse,
    Fallback,
}

impl fmt::Display for PackageSlot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            PackageSlot::Arch => "arch",
            PackageSlot::Ubuntu => "ubuntu",
            PackageSlot::Fedora => "fedora",
            PackageSlot::OpenSuse => "opensuse",
            PackageSlot::Fallback => "fallback",
        };

        f.write_str(name)
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedPackage<'a> {
    pub slot: PackageSlot,
    pub package_manager: PackageManager,
    pub reference: &'a str,
}

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("{0}")]
    Validation(String),
    #[error("failed to read manifest from {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse manifest: {0}")]
    Parse(#[from] serde_json::Error),
}

impl Manifest {
    pub fn from_json_str(input: &str) -> Result<Self, ManifestError> {
        serde_json::from_str(input).map_err(ManifestError::from)
    }

    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, ManifestError> {
        let path = path.as_ref();
        let contents = fs::read_to_string(path).map_err(|source| ManifestError::Io {
            path: path.to_path_buf(),
            source,
        })?;

        Self::from_json_str(&contents)
    }

    pub fn validate(&self) -> Result<(), ManifestError> {
        if self.name.trim().is_empty() {
            return Err(ManifestError::Validation(
                "manifest.name cannot be empty".to_string(),
            ));
        }

        if self.publisher.trim().is_empty() {
            return Err(ManifestError::Validation(
                "manifest.publisher cannot be empty".to_string(),
            ));
        }

        if self.version.trim().is_empty() {
            return Err(ManifestError::Validation(
                "manifest.version cannot be empty".to_string(),
            ));
        }

        if self.description.trim().is_empty() {
            return Err(ManifestError::Validation(
                "manifest.description cannot be empty".to_string(),
            ));
        }

        if !self.has_any_package() {
            return Err(ManifestError::Validation(
                "manifest.packages must contain at least one package reference".to_string(),
            ));
        }

        Ok(())
    }

    pub fn has_any_package(&self) -> bool {
        self.packages.arch.is_some()
            || self.packages.ubuntu.is_some()
            || self.packages.fedora.is_some()
            || self.packages.opensuse.is_some()
            || self.packages.fallback.is_some()
    }

    pub fn package_for_environment<'a>(
        &'a self,
        environment: &Environment,
    ) -> Option<ResolvedPackage<'a>> {
        let preferred_slot = preferred_slot(environment);
        self.package_for_slot(preferred_slot)
            .or_else(|| self.first_available_package())
    }

    pub fn package_for_slot<'a>(&'a self, slot: PackageSlot) -> Option<ResolvedPackage<'a>> {
        let reference = match slot {
            PackageSlot::Arch => self.packages.arch.as_deref(),
            PackageSlot::Ubuntu => self.packages.ubuntu.as_deref(),
            PackageSlot::Fedora => self.packages.fedora.as_deref(),
            PackageSlot::OpenSuse => self.packages.opensuse.as_deref(),
            PackageSlot::Fallback => self.packages.fallback.as_deref(),
        }?;

        Some(ResolvedPackage {
            slot,
            package_manager: package_manager_for_slot(slot),
            reference,
        })
    }

    pub fn first_available_package<'a>(&'a self) -> Option<ResolvedPackage<'a>> {
        [
            PackageSlot::Arch,
            PackageSlot::Ubuntu,
            PackageSlot::Fedora,
            PackageSlot::OpenSuse,
            PackageSlot::Fallback,
        ]
        .into_iter()
        .find_map(|slot| self.package_for_slot(slot))
    }
}

fn preferred_slot(environment: &Environment) -> PackageSlot {
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

    match environment.package_manager {
        PackageManager::Pacman => PackageSlot::Arch,
        PackageManager::Apt => PackageSlot::Ubuntu,
        PackageManager::Dnf => PackageSlot::Fedora,
        PackageManager::Zypper => PackageSlot::OpenSuse,
        PackageManager::PackageKit | PackageManager::Unknown => PackageSlot::Fallback,
    }
}

fn package_manager_for_slot(slot: PackageSlot) -> PackageManager {
    match slot {
        PackageSlot::Arch => PackageManager::Pacman,
        PackageSlot::Ubuntu => PackageManager::Apt,
        PackageSlot::Fedora => PackageManager::Dnf,
        PackageSlot::OpenSuse => PackageManager::Zypper,
        PackageSlot::Fallback => PackageManager::PackageKit,
    }
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

fn command_exists(binary: &str) -> bool {
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

#[allow(unused_mut)]
fn command_candidates(binary: &str) -> Vec<String> {
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

fn deserialize_package_ref<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<serde_json::Value>::deserialize(deserializer)?;

    match value {
        None | Some(serde_json::Value::Null) => Ok(None),
        Some(serde_json::Value::String(value)) => {
            let value = value.trim().to_string();
            if value.is_empty() {
                Ok(None)
            } else {
                Ok(Some(value))
            }
        }
        Some(serde_json::Value::Object(map)) => {
            if map.is_empty() {
                return Ok(None);
            }

            for key in ["url", "href", "uri", "download", "path", "file"] {
                if let Some(serde_json::Value::String(value)) = map.get(key) {
                    let value = value.trim().to_string();
                    if !value.is_empty() {
                        return Ok(Some(value));
                    }
                }
            }

            Err(de::Error::custom(
                "package objects must contain a string field named url, href, uri, download, path, or file",
            ))
        }
        Some(other) => Err(de::Error::custom(format!(
            "expected a string or object for package reference, got {other}"
        ))),
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
    fn preferred_slot_arch_by_pacman() {
        let env = Environment { distro: "unknown".into(), architecture: "x86_64".into(), package_manager: PackageManager::Pacman };
        assert_eq!(preferred_slot(&env), PackageSlot::Arch);
    }

    #[test]
    fn preferred_slot_arch_by_distro() {
        let env = Environment { distro: "arch linux".into(), architecture: "x86_64".into(), package_manager: PackageManager::Unknown };
        assert_eq!(preferred_slot(&env), PackageSlot::Arch);
    }

    #[test]
    fn preferred_slot_ubuntu_by_apt() {
        let env = Environment { distro: "unknown".into(), architecture: "x86_64".into(), package_manager: PackageManager::Apt };
        assert_eq!(preferred_slot(&env), PackageSlot::Ubuntu);
    }

    #[test]
    fn preferred_slot_fedora_by_dnf() {
        let env = Environment { distro: "unknown".into(), architecture: "x86_64".into(), package_manager: PackageManager::Dnf };
        assert_eq!(preferred_slot(&env), PackageSlot::Fedora);
    }

    #[test]
    fn preferred_slot_opensuse_by_zypper() {
        let env = Environment { distro: "opensuse tumbleweed".into(), architecture: "x86_64".into(), package_manager: PackageManager::Zypper };
        assert_eq!(preferred_slot(&env), PackageSlot::OpenSuse);
    }

    #[test]
    fn preferred_slot_fallback_unknown() {
        let env = Environment::unknown();
        assert_eq!(preferred_slot(&env), PackageSlot::Fallback);
    }

    #[test]
    fn preferred_slot_fallback_by_packagekit() {
        let env = Environment { distro: "unknown".into(), architecture: "x86_64".into(), package_manager: PackageManager::PackageKit };
        assert_eq!(preferred_slot(&env), PackageSlot::Fallback);
    }

    #[test]
    fn preferred_slot_centos_maps_fedora() {
        let env = Environment { distro: "centos stream 9".into(), architecture: "x86_64".into(), package_manager: PackageManager::Unknown };
        assert_eq!(preferred_slot(&env), PackageSlot::Fedora);
    }

    #[test]
    fn preferred_slot_debian_maps_ubuntu() {
        let env = Environment { distro: "debian bookworm".into(), architecture: "x86_64".into(), package_manager: PackageManager::Unknown };
        assert_eq!(preferred_slot(&env), PackageSlot::Ubuntu);
    }

    #[test]
    fn package_manager_display_all_variants() {
        assert_eq!(format!("{}", PackageManager::Apt), "apt");
        assert_eq!(format!("{}", PackageManager::Dnf), "dnf");
        assert_eq!(format!("{}", PackageManager::Pacman), "pacman");
        assert_eq!(format!("{}", PackageManager::Zypper), "zypper");
        assert_eq!(format!("{}", PackageManager::PackageKit), "packagekit");
        assert_eq!(format!("{}", PackageManager::Unknown), "unknown");
    }

    #[test]
    fn package_manager_is_system_pm() {
        assert!(PackageManager::Apt.is_system_package_manager());
        assert!(!PackageManager::Unknown.is_system_package_manager());
    }

    #[test]
    fn package_manager_install_commands() {
        assert_eq!(PackageManager::Apt.install_command(), Some("apt-get"));
        assert!(PackageManager::PackageKit.install_command().is_none());
        assert!(PackageManager::Unknown.install_command().is_none());
    }

    #[test]
    fn package_manager_query_commands() {
        assert_eq!(PackageManager::Apt.query_command(), Some("dpkg-query"));
        assert_eq!(PackageManager::Pacman.query_command(), Some("pacman"));
        assert_eq!(PackageManager::PackageKit.query_command(), None);
    }

    #[test]
    fn package_slot_display() {
        assert_eq!(format!("{}", PackageSlot::Arch), "arch");
        assert_eq!(format!("{}", PackageSlot::Ubuntu), "ubuntu");
        assert_eq!(format!("{}", PackageSlot::Fedora), "fedora");
        assert_eq!(format!("{}", PackageSlot::OpenSuse), "opensuse");
        assert_eq!(format!("{}", PackageSlot::Fallback), "fallback");
    }

    #[test]
    fn package_manager_for_slot_matches() {
        assert_eq!(package_manager_for_slot(PackageSlot::Arch), PackageManager::Pacman);
        assert_eq!(package_manager_for_slot(PackageSlot::Ubuntu), PackageManager::Apt);
        assert_eq!(package_manager_for_slot(PackageSlot::Fedora), PackageManager::Dnf);
        assert_eq!(package_manager_for_slot(PackageSlot::OpenSuse), PackageManager::Zypper);
        assert_eq!(package_manager_for_slot(PackageSlot::Fallback), PackageManager::PackageKit);
    }

    #[test]
    fn manifest_first_available_respects_order() {
        let manifest = Manifest {
            name: "Test".to_string(), publisher: "Pub".to_string(), version: "1".to_string(),
            description: "desc".to_string(), homepage: None, license: None, changelog: None,
            image: None,
            packages: PackageMatrix { arch: None, ubuntu: Some("u.deb".into()), fedora: None, opensuse: None, fallback: Some("f.AppImage".into()) },
            sha256: None, signature: None,
        };
        let pkg = manifest.first_available_package().unwrap();
        assert_eq!(pkg.slot, PackageSlot::Ubuntu);
    }

    #[test]
    fn manifest_validate_empty_name_rejected() {
        let manifest = Manifest {
            name: "".to_string(), publisher: "Pub".to_string(), version: "1".to_string(),
            description: "desc".to_string(), homepage: None, license: None, changelog: None,
            image: None, packages: PackageMatrix { arch: Some("x".into()), ..PackageMatrix::default() },
            sha256: None, signature: None,
        };
        assert!(matches!(manifest.validate(), Err(ManifestError::Validation(_))));
    }

    #[test]
    fn manifest_from_json_str_parse_error() {
        let err = Manifest::from_json_str("{invalid json}");
        assert!(err.is_err());
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
    fn os_release_parse_escaped_values() {
        let os = OsRelease::parse("PRETTY_NAME=\"Ubuntu \\\"22.04\\\" LTS\"\n");
        assert_eq!(os.pretty_name.as_deref(), Some("Ubuntu \"22.04\" LTS"));
    }

    #[test]
    fn detect_package_manager_falls_back_to_path() {
        let os = OsRelease::parse("ID=unknown\n");
        let pm = detect_package_manager(Some(&os));
        // on the CI runner this will detect whatever is on PATH
        // just checking it doesn't panic and returns a valid value
        assert!(matches!(pm, PackageManager::Apt | PackageManager::Dnf | PackageManager::Pacman | PackageManager::Zypper | PackageManager::PackageKit | PackageManager::Unknown));
    }

    #[test]
    fn deserialize_package_ref_rejects_bad_object() {
        let json = r#"{"packages":{"ubuntu":{"bad":"value"}}}"#;
        let result: Result<Manifest, _> = serde_json::from_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn package_matrix_default_is_empty() {
        let m = PackageMatrix::default();
        assert!(m.arch.is_none());
        assert!(m.ubuntu.is_none());
        assert!(m.fedora.is_none());
        assert!(m.opensuse.is_none());
        assert!(m.fallback.is_none());
    }

    #[test]
    fn parses_manifest_with_object_packages() {
        let manifest = Manifest::from_json_str(
            r#"
            {
                "name": "Cursor",
                "publisher": "Anysphere",
                "version": "1.5.0",
                "description": "AI Code Editor",
                "packages": {
                    "ubuntu": { "url": "https://example.com/ilovearina.deb" },
                    "arch": { "url": "https://example.com/ilovearina.pkg.tar.zst" }
                }
            }
            "#,
        )
        .expect("manifest should parse");

        assert_eq!(
            manifest.packages.ubuntu.as_deref(),
            Some("https://example.com/ilovearina.deb")
        );
        assert_eq!(
            manifest.packages.arch.as_deref(),
            Some("https://example.com/ilovearina.pkg.tar.zst")
        );
        manifest.validate().expect("manifest should validate");
    }

    #[test]
    fn selects_package_for_ubuntu_environment() {
        let manifest = Manifest {
            name: "Cursor".to_string(),
            publisher: "Anysphere".to_string(),
            version: "1.5.0".to_string(),
            description: "AI Code Editor".to_string(),
            homepage: None,
            license: None,
            changelog: None,
            image: None,
            packages: PackageMatrix {
                arch: None,
                ubuntu: Some("https://example.com/ilovearina.deb".to_string()),
                fedora: Some("https://example.com/ilovearina.rpm".to_string()),
                opensuse: None,
                fallback: None,
            },
            sha256: None,
            signature: None,
        };

        let environment = Environment::from_os_release(
            r#"
            ID=ubuntu
            ID_LIKE=debian
            PRETTY_NAME="Ubuntu"
            "#,
        );

        let selection = manifest
            .package_for_environment(&environment)
            .expect("a package should be selected");

        assert_eq!(selection.slot, PackageSlot::Ubuntu);
        assert_eq!(selection.package_manager, PackageManager::Apt);
        assert_eq!(selection.reference, "https://example.com/ilovearina.deb");
    }

    #[test]
    fn falls_back_to_first_available_package() {
        let manifest = Manifest {
            name: "Cursor".to_string(),
            publisher: "Anysphere".to_string(),
            version: "1.5.0".to_string(),
            description: "AI Code Editor".to_string(),
            homepage: None,
            license: None,
            changelog: None,
            image: None,
            packages: PackageMatrix {
                arch: None,
                ubuntu: None,
                fedora: Some("https://example.com/ilovearina.rpm".to_string()),
                opensuse: None,
                fallback: None,
            },
            sha256: None,
            signature: None,
        };

        let selection = manifest
            .package_for_environment(&Environment::unknown())
            .expect("a fallback package should be selected");

        assert_eq!(selection.slot, PackageSlot::Fedora);
        assert_eq!(selection.package_manager, PackageManager::Dnf);
    }

    #[test]
    fn validates_missing_package_set() {
        let manifest = Manifest {
            name: "Cursor".to_string(),
            publisher: "Anysphere".to_string(),
            version: "1.5.0".to_string(),
            description: "AI Code Editor".to_string(),
            homepage: None,
            license: None,
            changelog: None,
            image: None,
            packages: PackageMatrix::default(),
            sha256: None,
            signature: None,
        };

        let error = manifest
            .validate()
            .expect_err("manifest should be rejected");
        assert!(matches!(error, ManifestError::Validation(_)));
    }

    #[test]
    fn fallback_slot_is_last_resort() {
        let manifest = Manifest {
            name: "Test".to_string(),
            publisher: "TestPub".to_string(),
            version: "1.0".to_string(),
            description: "test".to_string(),
            homepage: None,
            license: None,
            changelog: None,
            image: None,
            packages: PackageMatrix {
                arch: None,
                ubuntu: None,
                fedora: None,
                opensuse: None,
                fallback: Some("https://example.com/test.AppImage".to_string()),
            },
            sha256: None,
            signature: None,
        };

        let env = Environment::from_os_release(
            "ID=unknown\nPRETTY_NAME=\"Unknown\"\n",
        );
        let selection = manifest
            .package_for_environment(&env)
            .expect("should return fallback");
        assert_eq!(selection.slot, PackageSlot::Fallback);
    }

    #[test]
    fn fallback_is_ignored_when_preferred_exists() {
        let manifest = Manifest {
            name: "Test".to_string(),
            publisher: "TestPub".to_string(),
            version: "1.0".to_string(),
            description: "test".to_string(),
            homepage: None,
            license: None,
            changelog: None,
            image: None,
            packages: PackageMatrix {
                arch: None,
                ubuntu: Some("https://example.com/test.deb".to_string()),
                fedora: None,
                opensuse: None,
                fallback: Some("https://example.com/test.AppImage".to_string()),
            },
            sha256: None,
            signature: None,
        };

        let env = Environment::from_os_release(
            "ID=ubuntu\nID_LIKE=debian\n",
        );
        let selection = manifest
            .package_for_environment(&env)
            .expect("should return ubuntu");
        assert_eq!(selection.slot, PackageSlot::Ubuntu);
        assert_eq!(
            selection.reference,
            "https://example.com/test.deb"
        );
    }
}
