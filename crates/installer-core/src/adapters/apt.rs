use crate::PackageManager;
use super::PackageAdapter;

pub struct AptAdapter;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn apt_install_command_format() {
        let adapter = AptAdapter;
        let (command, args) = adapter.install_command("/cache/test.deb");
        assert_eq!(command, "apt-get");
        assert_eq!(args, vec!["install", "-y", "/cache/test.deb"]);
    }

    #[test]
    fn apt_query_dependencies_command_format() {
        let adapter = AptAdapter;
        let (command, args) = adapter.query_dependencies_command("myapp");
        assert_eq!(command, "apt-cache");
        assert_eq!(args, vec!["depends", "myapp"]);
    }
}
