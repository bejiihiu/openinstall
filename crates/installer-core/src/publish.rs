use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{Manifest, ManifestError, PackageMatrix};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublishSpec {
    pub name: String,
    pub publisher: String,
    pub version: String,
    pub description: String,
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub changelog: Option<String>,
    pub image: Option<String>,
    pub packages: PackageMatrix,
    pub sha256: Option<String>,
    pub signature: Option<String>,
}

#[derive(Debug, Error)]
pub enum PublishError {
    #[error(transparent)]
    Manifest(#[from] ManifestError),
    #[error("at least one package must be provided")]
    NoPackages,
    #[error("missing package file: {0}")]
    MissingPackage(PathBuf),
    #[error("failed to read package file {path}: {source}")]
    PackageIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to write manifest file {path}: {source}")]
    ManifestIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to serialize manifest: {0}")]
    Serialize(#[from] serde_json::Error),
}

impl PublishSpec {
    pub fn validate(&self) -> Result<(), PublishError> {
        let manifest = self.to_manifest();
        manifest.validate()?;
        if !manifest.has_any_package() {
            return Err(PublishError::NoPackages);
        }
        Ok(())
    }

    pub fn to_manifest(&self) -> Manifest {
        Manifest {
            name: self.name.clone(),
            publisher: self.publisher.clone(),
            version: self.version.clone(),
            description: self.description.clone(),
            homepage: self.homepage.clone(),
            license: self.license.clone(),
            changelog: self.changelog.clone(),
            image: self.image.clone(),
            packages: self.packages.clone(),
            sha256: self.sha256.clone(),
            signature: self.signature.clone(),
        }
    }

    pub fn write_manifest(&self, path: impl AsRef<Path>) -> Result<(), PublishError> {
        self.validate()?;
        let manifest = self.to_manifest();
        let json = serde_json::to_string_pretty(&manifest)?;
        fs::write(path.as_ref(), json).map_err(|source| PublishError::ManifestIo {
            path: path.as_ref().to_path_buf(),
            source,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_spec() -> PublishSpec {
        PublishSpec {
            name: "TestApp".to_string(),
            publisher: "TestCorp".to_string(),
            version: "1.0.0".to_string(),
            description: "A test app".to_string(),
            homepage: None,
            license: None,
            changelog: None,
            image: None,
            packages: PackageMatrix {
                arch: Some("https://example.com/test.pkg.tar.zst".to_string()),
                ubuntu: None,
                fedora: None,
                opensuse: None,
                fallback: None,
            },
            sha256: None,
            signature: None,
        }
    }

    #[test]
    fn validates_publish_spec() {
        let spec = sample_spec();
        assert!(spec.validate().is_ok());
    }

    #[test]
    fn rejects_publish_spec_with_no_packages() {
        let spec = PublishSpec {
            packages: PackageMatrix::default(),
            ..sample_spec()
        };
        assert!(spec.validate().is_err());
    }

    #[test]
    fn converts_to_manifest() {
        let spec = sample_spec();
        let manifest = spec.to_manifest();
        assert_eq!(manifest.name, "TestApp");
        assert_eq!(
            manifest.packages.arch.as_deref(),
            Some("https://example.com/test.pkg.tar.zst")
        );
    }
}
