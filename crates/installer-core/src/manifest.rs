use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::environment::{Environment, PackageManager, package_manager_for_slot, preferred_slot};
use crate::matrix::{PackageMatrix, PackageSlot};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Scripts {
    #[serde(default)]
    pub preinstall: Option<String>,
    #[serde(default)]
    pub postinstall: Option<String>,
    #[serde(default)]
    pub preremove: Option<String>,
    #[serde(default)]
    pub postremove: Option<String>,
}

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
    #[serde(default)]
    pub scripts: Option<Scripts>,
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
    #[error("failed to fetch manifest from {url}: {source}")]
    Http {
        url: String,
        #[source]
        source: reqwest::Error,
    },
    #[error("failed to parse manifest: {0}")]
    Parse(#[from] serde_json::Error),
}

impl Manifest {
    pub fn from_json_str(input: &str) -> Result<Self, ManifestError> {
        serde_json::from_str(input).map_err(ManifestError::from)
    }

    pub fn from_url(url: &url::Url) -> Result<Self, ManifestError> {
        let response = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_secs(15))
            .build()
            .map_err(|source| ManifestError::Http {
                url: url.to_string(),
                source,
            })?
            .get(url.as_str())
            .send()
            .and_then(|r| r.error_for_status())
            .map_err(|source| ManifestError::Http {
                url: url.to_string(),
                source,
            })?;
        let text = response.text().map_err(|source| ManifestError::Http {
            url: url.to_string(),
            source,
        })?;
        Self::from_json_str(&text)
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
            || self.packages.flatpak.is_some()
            || self.packages.appimage.is_some()
            || self.packages.windows.is_some()
            || self.packages.macos.is_some()
    }

    pub fn package_for_environment<'a>(
        &'a self,
        environment: &Environment,
    ) -> Option<ResolvedPackage<'a>> {
        let slot = preferred_slot(environment);
        self.package_for_slot(slot)
            .or_else(|| self.first_available_package())
    }

    pub fn package_for_slot<'a>(&'a self, slot: PackageSlot) -> Option<ResolvedPackage<'a>> {
        let reference = match slot {
            PackageSlot::Arch => self.packages.arch.as_deref(),
            PackageSlot::Ubuntu => self.packages.ubuntu.as_deref(),
            PackageSlot::Fedora => self.packages.fedora.as_deref(),
            PackageSlot::OpenSuse => self.packages.opensuse.as_deref(),
            PackageSlot::Flatpak => self.packages.flatpak.as_deref(),
            PackageSlot::AppImage => self.packages.appimage.as_deref(),
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
            PackageSlot::Flatpak,
            PackageSlot::AppImage,
        ]
        .into_iter()
        .find_map(|slot| self.package_for_slot(slot))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn manifest_first_available_respects_order() {
        let manifest = Manifest {
            name: "Test".to_string(),
            publisher: "Pub".to_string(),
            version: "1".to_string(),
            description: "desc".to_string(),
            homepage: None,
            license: None,
            changelog: None,
            image: None,
            packages: PackageMatrix {
                arch: None,
                ubuntu: Some("u.deb".into()),
                fedora: None,
                opensuse: None,
                flatpak: None,
                appimage: Some("f.AppImage".into()),
                windows: None,
                macos: None,
            },
            sha256: None,
            signature: None,
            scripts: None,
        };
        let pkg = manifest.first_available_package().unwrap();
        assert_eq!(pkg.slot, PackageSlot::Ubuntu);
    }

    #[test]
    fn manifest_validate_empty_name_rejected() {
        let manifest = Manifest {
            name: "".to_string(),
            publisher: "Pub".to_string(),
            version: "1".to_string(),
            description: "desc".to_string(),
            homepage: None,
            license: None,
            changelog: None,
            image: None,
            packages: PackageMatrix {
                arch: Some("x".into()),
                ..PackageMatrix::default()
            },
            sha256: None,
            signature: None,
            scripts: None,
        };
        assert!(matches!(
            manifest.validate(),
            Err(ManifestError::Validation(_))
        ));
    }

    #[test]
    fn manifest_from_json_str_parse_error() {
        let err = Manifest::from_json_str("{invalid json}");
        assert!(err.is_err());
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
                flatpak: None,
                appimage: None,
                windows: None,
                macos: None,
            },
            sha256: None,
            signature: None,
            scripts: None,
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
                flatpak: None,
                appimage: None,
                windows: None,
                macos: None,
            },
            sha256: None,
            signature: None,
            scripts: None,
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
            scripts: None,
        };

        let error = manifest
            .validate()
            .expect_err("manifest should be rejected");
        assert!(matches!(error, ManifestError::Validation(_)));
    }

    #[test]
    fn appimage_slot_is_last_resort() {
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
                flatpak: None,
                appimage: Some("https://example.com/test.AppImage".to_string()),
                windows: None,
                macos: None,
            },
            sha256: None,
            signature: None,
            scripts: None,
        };

        let env = Environment::from_os_release("ID=unknown\nPRETTY_NAME=\"Unknown\"\n");
        let selection = manifest
            .package_for_environment(&env)
            .expect("should return appimage");
        assert_eq!(selection.slot, PackageSlot::AppImage);
    }

    #[test]
    fn appimage_is_ignored_when_preferred_exists() {
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
                flatpak: None,
                appimage: Some("https://example.com/test.AppImage".to_string()),
                windows: None,
                macos: None,
            },
            sha256: None,
            signature: None,
            scripts: None,
        };

        let env = Environment::from_os_release("ID=ubuntu\nID_LIKE=debian\n");
        let selection = manifest
            .package_for_environment(&env)
            .expect("should return ubuntu");
        assert_eq!(selection.slot, PackageSlot::Ubuntu);
        assert_eq!(selection.reference, "https://example.com/test.deb");
    }

    #[test]
    fn parses_manifest_with_scripts() {
        let manifest = Manifest::from_json_str(
            r#"
            {
                "name": "Test",
                "publisher": "Pub",
                "version": "1.0",
                "description": "test",
                "packages": { "appimage": "test.AppImage" },
                "scripts": {
                    "preinstall": "echo hello",
                    "postinstall": "echo done"
                }
            }
            "#,
        )
        .expect("manifest with scripts should parse");

        let scripts = manifest.scripts.expect("scripts should be present");
        assert_eq!(scripts.preinstall.as_deref(), Some("echo hello"));
        assert_eq!(scripts.postinstall.as_deref(), Some("echo done"));
        assert!(scripts.preremove.is_none());
        assert!(scripts.postremove.is_none());
    }

    #[test]
    fn first_available_flatpak_before_appimage() {
        let manifest = Manifest {
            name: "Test".to_string(),
            publisher: "Pub".to_string(),
            version: "1".to_string(),
            description: "desc".to_string(),
            homepage: None,
            license: None,
            changelog: None,
            image: None,
            packages: PackageMatrix {
                arch: None,
                ubuntu: None,
                fedora: None,
                opensuse: None,
                flatpak: Some("flatpak://com.example.Test".into()),
                appimage: Some("test.AppImage".into()),
                windows: None,
                macos: None,
            },
            sha256: None,
            signature: None,
            scripts: None,
        };
        let pkg = manifest.first_available_package().unwrap();
        assert_eq!(pkg.slot, PackageSlot::Flatpak);
    }
}
