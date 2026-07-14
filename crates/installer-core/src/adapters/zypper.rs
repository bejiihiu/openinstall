use super::PackageAdapter;
use crate::PackageManager;

pub struct ZypperAdapter;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zypper_install_command_format() {
        let adapter = ZypperAdapter;
        let (command, args) = adapter.install_command("/cache/test.rpm");
        assert_eq!(command, "zypper");
        assert_eq!(
            args,
            vec!["--non-interactive", "install", "/cache/test.rpm"]
        );
    }
}
