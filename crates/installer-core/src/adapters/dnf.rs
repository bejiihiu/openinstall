use super::PackageAdapter;
use crate::PackageManager;

pub struct DnfAdapter;

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dnf_remove_command_format() {
        let adapter = DnfAdapter;
        let (command, args) = adapter.remove_command("myapp");
        assert_eq!(command, "dnf");
        assert_eq!(args, vec!["remove", "-y", "myapp"]);
    }
}
