use crate::PackageManager;
use super::PackageAdapter;

pub struct PacmanAdapter;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pacman_install_command_format() {
        let adapter = PacmanAdapter;
        let (command, args) = adapter.install_command("/cache/test.pkg.tar.zst");
        assert_eq!(command, "pacman");
        assert_eq!(args, vec!["-U", "--noconfirm", "/cache/test.pkg.tar.zst"]);
    }

    #[test]
    fn pacman_query_installed_command_format() {
        let adapter = PacmanAdapter;
        let (command, args) = adapter.query_installed_command("myapp");
        assert_eq!(command, "pacman");
        assert_eq!(args, vec!["-Q", "myapp"]);
    }
}
