pub mod apt;
pub mod dnf;
pub mod pacman;
pub mod zypper;
pub mod flatpak;
pub mod packagekit;

use crate::PackageManager;

pub trait PackageAdapter: Send + Sync {
    fn manager(&self) -> PackageManager;
    fn install_command(&self, staged_path: &str) -> (String, Vec<String>);
    fn remove_command(&self, package_id: &str) -> (String, Vec<String>);
    fn query_installed_command(&self, package_id: &str) -> (String, Vec<String>);
    fn query_version_command(&self, package_id: &str) -> (String, Vec<String>);
    fn query_dependencies_command(&self, package_id: &str) -> (String, Vec<String>);
}

use apt::AptAdapter;
use dnf::DnfAdapter;
use pacman::PacmanAdapter;
use zypper::ZypperAdapter;
use flatpak::FlatpakAdapter;
use packagekit::PackageKitAdapter;

static APT: AptAdapter = AptAdapter;
static DNF: DnfAdapter = DnfAdapter;
static PACMAN: PacmanAdapter = PacmanAdapter;
static ZYPPER: ZypperAdapter = ZypperAdapter;
static FLATPAK: FlatpakAdapter = FlatpakAdapter;
static PACKAGEKIT: PackageKitAdapter = PackageKitAdapter;

pub fn adapter_for(manager: PackageManager) -> Option<&'static dyn PackageAdapter> {
    match manager {
        PackageManager::Apt => Some(&APT),
        PackageManager::Dnf => Some(&DNF),
        PackageManager::Pacman => Some(&PACMAN),
        PackageManager::Zypper => Some(&ZYPPER),
        PackageManager::Flatpak => Some(&FLATPAK),
        PackageManager::PackageKit => Some(&PACKAGEKIT),
        PackageManager::Unknown => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn adapter_for_returns_correct_adapter() {
        assert!(adapter_for(PackageManager::Apt).is_some());
        assert!(adapter_for(PackageManager::Dnf).is_some());
        assert!(adapter_for(PackageManager::Pacman).is_some());
        assert!(adapter_for(PackageManager::Zypper).is_some());
        assert!(adapter_for(PackageManager::PackageKit).is_some());
        assert!(adapter_for(PackageManager::Unknown).is_none());
    }
}
