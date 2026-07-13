use crate::PackageManager;

pub trait PackageAdapter {
    fn manager(&self) -> PackageManager;
    fn install_command(&self, staged_path: &str) -> (String, Vec<String>);
    fn remove_command(&self, package_id: &str) -> (String, Vec<String>);
    fn query_installed_command(&self, package_id: &str) -> (String, Vec<String>);
    fn query_version_command(&self, package_id: &str) -> (String, Vec<String>);
    fn query_dependencies_command(&self, package_id: &str) -> (String, Vec<String>);
}

pub struct AptAdapter;
pub struct DnfAdapter;
pub struct PacmanAdapter;
pub struct ZypperAdapter;

impl PackageAdapter for AptAdapter {
    fn manager(&self) -> PackageManager {
        PackageManager::Apt
    }

    fn install_command(&self, staged_path: &str) -> (String, Vec<String>) {
        (
            "apt-get".to_string(),
            vec!["install".into(), "-y".into(), staged_path.to_string()],
        )
    }

    fn remove_command(&self, package_id: &str) -> (String, Vec<String>) {
        (
            "apt-get".to_string(),
            vec!["remove".into(), "-y".into(), package_id.to_string()],
        )
    }

    fn query_installed_command(&self, package_id: &str) -> (String, Vec<String>) {
        (
            "dpkg-query".to_string(),
            vec!["-W".into(), "-f=${Status}".into(), package_id.to_string()],
        )
    }

    fn query_version_command(&self, package_id: &str) -> (String, Vec<String>) {
        (
            "dpkg-query".to_string(),
            vec!["-W".into(), "-f=${Version}".into(), package_id.to_string()],
        )
    }

    fn query_dependencies_command(&self, package_id: &str) -> (String, Vec<String>) {
        (
            "apt-cache".to_string(),
            vec!["depends".into(), package_id.to_string()],
        )
    }
}

impl PackageAdapter for DnfAdapter {
    fn manager(&self) -> PackageManager {
        PackageManager::Dnf
    }

    fn install_command(&self, staged_path: &str) -> (String, Vec<String>) {
        (
            "dnf".to_string(),
            vec!["install".into(), "-y".into(), staged_path.to_string()],
        )
    }

    fn remove_command(&self, package_id: &str) -> (String, Vec<String>) {
        (
            "dnf".to_string(),
            vec!["remove".into(), "-y".into(), package_id.to_string()],
        )
    }

    fn query_installed_command(&self, package_id: &str) -> (String, Vec<String>) {
        ("rpm".to_string(), vec!["-q".into(), package_id.to_string()])
    }

    fn query_version_command(&self, package_id: &str) -> (String, Vec<String>) {
        (
            "rpm".to_string(),
            vec![
                "-q".into(),
                "--qf".into(),
                "%{VERSION}-%{RELEASE}".into(),
                package_id.to_string(),
            ],
        )
    }

    fn query_dependencies_command(&self, package_id: &str) -> (String, Vec<String>) {
        (
            "dnf".to_string(),
            vec![
                "repoquery".into(),
                "--requires".into(),
                package_id.to_string(),
            ],
        )
    }
}

impl PackageAdapter for PacmanAdapter {
    fn manager(&self) -> PackageManager {
        PackageManager::Pacman
    }

    fn install_command(&self, staged_path: &str) -> (String, Vec<String>) {
        (
            "pacman".to_string(),
            vec!["-U".into(), "--noconfirm".into(), staged_path.to_string()],
        )
    }

    fn remove_command(&self, package_id: &str) -> (String, Vec<String>) {
        (
            "pacman".to_string(),
            vec!["-R".into(), "--noconfirm".into(), package_id.to_string()],
        )
    }

    fn query_installed_command(&self, package_id: &str) -> (String, Vec<String>) {
        (
            "pacman".to_string(),
            vec!["-Q".into(), package_id.to_string()],
        )
    }

    fn query_version_command(&self, package_id: &str) -> (String, Vec<String>) {
        (
            "pacman".to_string(),
            vec!["-Q".into(), package_id.to_string()],
        )
    }

    fn query_dependencies_command(&self, package_id: &str) -> (String, Vec<String>) {
        (
            "pacman".to_string(),
            vec!["-Qi".into(), package_id.to_string()],
        )
    }
}

impl PackageAdapter for ZypperAdapter {
    fn manager(&self) -> PackageManager {
        PackageManager::Zypper
    }

    fn install_command(&self, staged_path: &str) -> (String, Vec<String>) {
        (
            "zypper".to_string(),
            vec![
                "--non-interactive".into(),
                "install".into(),
                staged_path.to_string(),
            ],
        )
    }

    fn remove_command(&self, package_id: &str) -> (String, Vec<String>) {
        (
            "zypper".to_string(),
            vec![
                "--non-interactive".into(),
                "remove".into(),
                package_id.to_string(),
            ],
        )
    }

    fn query_installed_command(&self, package_id: &str) -> (String, Vec<String>) {
        ("rpm".to_string(), vec!["-q".into(), package_id.to_string()])
    }

    fn query_version_command(&self, package_id: &str) -> (String, Vec<String>) {
        (
            "rpm".to_string(),
            vec![
                "-q".into(),
                "--qf".into(),
                "%{VERSION}-%{RELEASE}".into(),
                package_id.to_string(),
            ],
        )
    }

    fn query_dependencies_command(&self, package_id: &str) -> (String, Vec<String>) {
        (
            "zypper".to_string(),
            vec!["info".into(), package_id.to_string()],
        )
    }
}

pub struct PackageKitAdapter;

impl PackageAdapter for PackageKitAdapter {
    fn manager(&self) -> PackageManager {
        PackageManager::PackageKit
    }

    fn install_command(&self, staged_path: &str) -> (String, Vec<String>) {
        (
            "pkcon".to_string(),
            vec!["install-local".into(), "--noninteractive".into(), staged_path.to_string()],
        )
    }

    fn remove_command(&self, package_id: &str) -> (String, Vec<String>) {
        (
            "pkcon".to_string(),
            vec!["remove".into(), "--noninteractive".into(), package_id.to_string()],
        )
    }

    fn query_installed_command(&self, package_id: &str) -> (String, Vec<String>) {
        (
            "pkcon".to_string(),
            vec!["get-details".into(), package_id.to_string()],
        )
    }

    fn query_version_command(&self, package_id: &str) -> (String, Vec<String>) {
        (
            "pkcon".to_string(),
            vec!["get-details".into(), package_id.to_string()],
        )
    }

    fn query_dependencies_command(&self, package_id: &str) -> (String, Vec<String>) {
        (
            "pkcon".to_string(),
            vec!["get-details".into(), package_id.to_string()],
        )
    }
}

static APT: AptAdapter = AptAdapter;
static DNF: DnfAdapter = DnfAdapter;
static PACMAN: PacmanAdapter = PacmanAdapter;
static ZYPPER: ZypperAdapter = ZypperAdapter;
static PACKAGEKIT: PackageKitAdapter = PackageKitAdapter;

pub fn adapter_for(manager: PackageManager) -> Option<&'static dyn PackageAdapter> {
    match manager {
        PackageManager::Apt => Some(&APT),
        PackageManager::Dnf => Some(&DNF),
        PackageManager::Pacman => Some(&PACMAN),
        PackageManager::Zypper => Some(&ZYPPER),
        PackageManager::PackageKit => Some(&PACKAGEKIT),
        PackageManager::Unknown => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packagekit_accepts_install_command() {
        let adapter = PackageKitAdapter;
        let (command, args) = adapter.install_command("/tmp/test-pkg.deb");
        assert_eq!(command, "pkcon");
        assert_eq!(args, vec!["install-local", "--noninteractive", "/tmp/test-pkg.deb"]);
    }

    #[test]
    fn apt_install_command_format() {
        let adapter = AptAdapter;
        let (command, args) = adapter.install_command("/cache/test.deb");
        assert_eq!(command, "apt-get");
        assert_eq!(args, vec!["install", "-y", "/cache/test.deb"]);
    }

    #[test]
    fn dnf_remove_command_format() {
        let adapter = DnfAdapter;
        let (command, args) = adapter.remove_command("myapp");
        assert_eq!(command, "dnf");
        assert_eq!(args, vec!["remove", "-y", "myapp"]);
    }

    #[test]
    fn pacman_install_command_format() {
        let adapter = PacmanAdapter;
        let (command, args) = adapter.install_command("/cache/test.pkg.tar.zst");
        assert_eq!(command, "pacman");
        assert_eq!(args, vec!["-U", "--noconfirm", "/cache/test.pkg.tar.zst"]);
    }

    #[test]
    fn zypper_install_command_format() {
        let adapter = ZypperAdapter;
        let (command, args) = adapter.install_command("/cache/test.rpm");
        assert_eq!(command, "zypper");
        assert_eq!(args, vec!["--non-interactive", "install", "/cache/test.rpm"]);
    }

    #[test]
    fn adapter_for_returns_correct_adapter() {
        assert!(adapter_for(PackageManager::Apt).is_some());
        assert!(adapter_for(PackageManager::Dnf).is_some());
        assert!(adapter_for(PackageManager::Pacman).is_some());
        assert!(adapter_for(PackageManager::Zypper).is_some());
        assert!(adapter_for(PackageManager::PackageKit).is_some());
        assert!(adapter_for(PackageManager::Unknown).is_none());
    }

    #[test]
    fn pacman_query_installed_command_format() {
        let adapter = PacmanAdapter;
        let (command, args) = adapter.query_installed_command("myapp");
        assert_eq!(command, "pacman");
        assert_eq!(args, vec!["-Q", "myapp"]);
    }

    #[test]
    fn apt_query_dependencies_command_format() {
        let adapter = AptAdapter;
        let (command, args) = adapter.query_dependencies_command("myapp");
        assert_eq!(command, "apt-cache");
        assert_eq!(args, vec!["depends", "myapp"]);
    }

    #[test]
    fn packagekit_remove_command_format() {
        let adapter = PackageKitAdapter;
        let (command, args) = adapter.remove_command("myapp");
        assert_eq!(command, "pkcon");
        assert_eq!(args, vec!["remove", "--noninteractive", "myapp"]);
    }
}
