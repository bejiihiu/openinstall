use super::PackageAdapter;
use crate::PackageManager;

pub struct FlatpakAdapter;

impl PackageAdapter for FlatpakAdapter {
    fn manager(&self) -> PackageManager {
        PackageManager::Flatpak
    }

    fn install_command(&self, staged_path: &str) -> (String, Vec<String>) {
        (
            "flatpak".to_string(),
            vec![
                "install".into(),
                "--user".into(),
                "-y".into(),
                staged_path.to_string(),
            ],
        )
    }

    fn remove_command(&self, package_id: &str) -> (String, Vec<String>) {
        (
            "flatpak".to_string(),
            vec![
                "uninstall".into(),
                "--user".into(),
                "-y".into(),
                package_id.to_string(),
            ],
        )
    }

    fn query_installed_command(&self, package_id: &str) -> (String, Vec<String>) {
        (
            "flatpak".to_string(),
            vec!["info".into(), package_id.to_string()],
        )
    }

    fn query_version_command(&self, package_id: &str) -> (String, Vec<String>) {
        (
            "flatpak".to_string(),
            vec!["info".into(), "--show-ref".into(), package_id.to_string()],
        )
    }

    fn query_dependencies_command(&self, package_id: &str) -> (String, Vec<String>) {
        (
            "flatpak".to_string(),
            vec![
                "info".into(),
                "--show-dependencies".into(),
                package_id.to_string(),
            ],
        )
    }
}
