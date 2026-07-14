use super::PackageAdapter;
use crate::PackageManager;

pub struct PackageKitAdapter;

impl PackageAdapter for PackageKitAdapter {
    fn manager(&self) -> PackageManager {
        PackageManager::PackageKit
    }

    fn install_command(&self, staged_path: &str) -> (String, Vec<String>) {
        (
            "pkcon".to_string(),
            vec![
                "install-local".into(),
                "--non-interactive".into(),
                staged_path.to_string(),
            ],
        )
    }

    fn remove_command(&self, package_id: &str) -> (String, Vec<String>) {
        (
            "pkcon".to_string(),
            vec![
                "remove".into(),
                "--non-interactive".into(),
                package_id.to_string(),
            ],
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn packagekit_accepts_install_command() {
        let adapter = PackageKitAdapter;
        let (command, args) = adapter.install_command("/tmp/test-pkg.deb");
        assert_eq!(command, "pkcon");
        assert_eq!(
            args,
            vec!["install-local", "--non-interactive", "/tmp/test-pkg.deb"]
        );
    }

    #[test]
    fn packagekit_remove_command_format() {
        let adapter = PackageKitAdapter;
        let (command, args) = adapter.remove_command("myapp");
        assert_eq!(command, "pkcon");
        assert_eq!(args, vec!["remove", "--non-interactive", "myapp"]);
    }
}
