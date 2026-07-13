use std::fs;
use std::io::{BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Output, Stdio};
use std::sync::mpsc;
use std::time::{SystemTime, UNIX_EPOCH};

use hex::ToHex;
use reqwest::blocking::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::{
    adapter_for, resolve_latest_release_asset, Environment, Manifest, ManifestError,
    PackageManager, ResolvedPackage, SignatureSpec,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheInfo {
    pub file_count: usize,
    pub total_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallOutcome {
    pub package_id: String,
    pub package_manager: PackageManager,
    pub staged_path: PathBuf,
    pub command: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerificationOutcome {
    pub staged_path: PathBuf,
    pub sha256_ok: bool,
    pub signature_present: bool,
    pub signature_ok: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallationState {
    NotInstalled,
    SameVersion {
        version: String,
    },
    DifferentVersion {
        current_version: String,
        available_version: String,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallStage {
    Downloading,
    Verifying,
    Installing,
    Done,
}

#[derive(Debug, Clone)]
pub struct InstallProgress {
    pub downloaded_bytes: u64,
    pub total_bytes: u64,
    pub speed_bytes_per_sec: f64,
    pub log_line: Option<String>,
    pub stage: InstallStage,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub package_id: String,
    pub package_manager: PackageManager,
    pub version: String,
    pub staged_path: String,
    pub reference: String,
    pub sha256: Option<String>,
    pub installed_at_unix_secs: u64,
}

#[derive(Debug, Error)]
pub enum InstallerError {
    #[error(transparent)]
    Manifest(#[from] ManifestError),
    #[error("no package is available for this environment")]
    NoPackage,
    #[error("unsupported or unknown package reference: {0}")]
    UnsupportedReference(String),
    #[error("cache directory error: {0}")]
    Cache(String),
    #[error("failed to create cache directory {path}: {source}")]
    CacheIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to stage package from {url}: {source}")]
    Download {
        url: String,
        #[source]
        source: reqwest::Error,
    },
    #[error("failed to read file {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to execute {command}: {source}")]
    Command {
        command: String,
        #[source]
        source: std::io::Error,
    },
    #[error("package manager command {command} failed with status {status}")]
    CommandStatus { command: String, status: i32 },
    #[error("package manager {0} is not supported for this operation")]
    UnsupportedManager(PackageManager),
    #[error("sha256 mismatch for {path}")]
    Sha256Mismatch {
        path: PathBuf,
        expected: String,
        actual: String,
    },
    #[error("signature verification failed: {0}")]
    Signature(String),
    #[error("history error: {0}")]
    History(String),
}

#[derive(Debug, Clone)]
pub struct Installer {
    cache_dir: PathBuf,
    history_path: PathBuf,
}

impl Default for Installer {
    fn default() -> Self {
        Self::new(default_cache_dir())
    }
}

impl Installer {
    pub fn new(cache_dir: PathBuf) -> Self {
        let history_path = cache_dir.join("history.json");
        Self {
            cache_dir,
            history_path,
        }
    }

    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    pub fn verify(
        &self,
        manifest: &Manifest,
        environment: &Environment,
    ) -> Result<VerificationOutcome, InstallerError> {
        manifest.validate()?;
        let package = manifest
            .package_for_environment(environment)
            .ok_or(InstallerError::NoPackage)?;
        let staged_path = self.stage_package(manifest, &package, None)?;
        let sha256_ok = self.verify_sha256_if_present(manifest, &staged_path)?;
        let signature_ok = self.verify_signature_if_present(manifest, &staged_path)?;

        Ok(VerificationOutcome {
            staged_path,
            sha256_ok,
            signature_present: manifest.signature.is_some(),
            signature_ok,
        })
    }

    pub fn inspect(
        &self,
        manifest: &Manifest,
        environment: &Environment,
    ) -> Result<InstallationState, InstallerError> {
        manifest.validate()?;
        let is_installed = self.is_installed(manifest, environment)?;
        if !is_installed {
            return Ok(InstallationState::NotInstalled);
        }

        let current_version = self
            .get_version(manifest, environment)?
            .unwrap_or_else(|| "unknown".to_string());
        if current_version == manifest.version {
            Ok(InstallationState::SameVersion {
                version: current_version,
            })
        } else {
            Ok(InstallationState::DifferentVersion {
                current_version,
                available_version: manifest.version.clone(),
            })
        }
    }

    pub fn install(
        &self,
        manifest: &Manifest,
        environment: &Environment,
    ) -> Result<InstallOutcome, InstallerError> {
        manifest.validate()?;
        let package = manifest
            .package_for_environment(environment)
            .ok_or(InstallerError::NoPackage)?;
        let package_id = package_id_for_manifest(manifest);
        let staged_path = self.stage_package(manifest, &package, None)?;
        self.verify_sha256_if_present(manifest, &staged_path)?;
        self.verify_signature_if_present(manifest, &staged_path)?;

        let command = if package.slot == crate::PackageSlot::Fallback {
            format!("staged: {}", staged_path.display())
        } else {
            self.install_with_package_manager(package.package_manager, &staged_path, &package_id)?
        };
        self.append_history(&HistoryEntry {
            package_id: package_id.clone(),
            package_manager: package.package_manager,
            version: manifest.version.clone(),
            staged_path: staged_path.display().to_string(),
            reference: package.reference.to_string(),
            sha256: manifest
                .sha256
                .as_ref()
                .map(|value| normalize_sha256(value)),
            installed_at_unix_secs: current_unix_secs(),
        })?;

        Ok(InstallOutcome {
            package_id,
            package_manager: package.package_manager,
            staged_path,
            command,
        })
    }

    pub fn install_with_progress(
        &self,
        manifest: &Manifest,
        environment: &Environment,
        tx: mpsc::Sender<InstallProgress>,
    ) -> Result<InstallOutcome, InstallerError> {
        manifest.validate()?;
        let package = manifest
            .package_for_environment(environment)
            .ok_or(InstallerError::NoPackage)?;
        let package_id = package_id_for_manifest(manifest);

        let _ = tx.send(InstallProgress {
            downloaded_bytes: 0,
            total_bytes: 0,
            speed_bytes_per_sec: 0.0,
            log_line: Some("Downloading package...".to_string()),
            stage: InstallStage::Downloading,
        });

        let staged_path = self.stage_package(manifest, &package, Some(&tx))?;

        let _ = tx.send(InstallProgress {
            downloaded_bytes: 0,
            total_bytes: 0,
            speed_bytes_per_sec: 0.0,
            log_line: Some("Verifying package...".to_string()),
            stage: InstallStage::Verifying,
        });

        self.verify_sha256_if_present(manifest, &staged_path)?;
        self.verify_signature_if_present(manifest, &staged_path)?;

        let command = if package.slot == crate::PackageSlot::Fallback {
            format!("staged: {}", staged_path.display())
        } else {
            let _ = tx.send(InstallProgress {
                downloaded_bytes: 0,
                total_bytes: 0,
                speed_bytes_per_sec: 0.0,
                log_line: Some("Installing package...".to_string()),
                stage: InstallStage::Installing,
            });
            let Some(adapter) = adapter_for(package.package_manager) else {
                return Err(InstallerError::UnsupportedManager(package.package_manager));
            };
            let (cmd, args) = adapter.install_command(&staged_path.display().to_string());
            self.run_command_streaming(&cmd, &args, &tx)?;
            format!("{cmd} {}", args.join(" "))
        };

        let _ = tx.send(InstallProgress {
            downloaded_bytes: 0,
            total_bytes: 0,
            speed_bytes_per_sec: 0.0,
            log_line: Some("Done.".to_string()),
            stage: InstallStage::Done,
        });

        self.append_history(&HistoryEntry {
            package_id: package_id.clone(),
            package_manager: package.package_manager,
            version: manifest.version.clone(),
            staged_path: staged_path.display().to_string(),
            reference: package.reference.to_string(),
            sha256: manifest
                .sha256
                .as_ref()
                .map(|value| normalize_sha256(value)),
            installed_at_unix_secs: current_unix_secs(),
        })?;

        Ok(InstallOutcome {
            package_id,
            package_manager: package.package_manager,
            staged_path,
            command,
        })
    }

    pub fn update(
        &self,
        manifest: &Manifest,
        environment: &Environment,
    ) -> Result<InstallOutcome, InstallerError> {
        self.install(manifest, environment)
    }

    pub fn remove(
        &self,
        manifest: &Manifest,
        environment: &Environment,
    ) -> Result<InstallOutcome, InstallerError> {
        manifest.validate()?;
        let package = manifest
            .package_for_environment(environment)
            .ok_or(InstallerError::NoPackage)?;
        let package_id = package_id_for_manifest(manifest);
        let command = self.remove_with_package_manager(package.package_manager, &package_id)?;

        Ok(InstallOutcome {
            package_id,
            package_manager: package.package_manager,
            staged_path: PathBuf::new(),
            command,
        })
    }

    pub fn rollback(
        &self,
        manifest: &Manifest,
        _environment: &Environment,
    ) -> Result<InstallOutcome, InstallerError> {
        manifest.validate()?;
        let history = self.read_history()?;
        let Some(previous) = history.iter().rev().nth(1) else {
            return Err(InstallerError::History(
                "rollback requires at least one previous successful installation".to_string(),
            ));
        };

        let staged_path = PathBuf::from(&previous.staged_path);
        if !staged_path.exists() {
            return Err(InstallerError::Io {
                path: staged_path,
                source: std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "cached rollback artifact is missing",
                ),
            });
        }

        if let Some(expected) = previous.sha256.as_deref() {
            verify_sha256_value(expected, &staged_path)?;
        }
        let command = self.install_with_package_manager(
            previous.package_manager,
            &staged_path,
            &previous.package_id,
        )?;

        Ok(InstallOutcome {
            package_id: previous.package_id.clone(),
            package_manager: previous.package_manager,
            staged_path,
            command,
        })
    }

    pub fn is_installed(
        &self,
        manifest: &Manifest,
        environment: &Environment,
    ) -> Result<bool, InstallerError> {
        let package = manifest
            .package_for_environment(environment)
            .ok_or(InstallerError::NoPackage)?;
        let package_id = package_id_for_manifest(manifest);
        let output = self.run_query(package.package_manager, &package_id, QueryKind::Installed)?;
        Ok(output.status.success())
    }

    pub fn get_version(
        &self,
        manifest: &Manifest,
        environment: &Environment,
    ) -> Result<Option<String>, InstallerError> {
        let package = manifest
            .package_for_environment(environment)
            .ok_or(InstallerError::NoPackage)?;
        let package_id = package_id_for_manifest(manifest);
        let output = self.run_query(package.package_manager, &package_id, QueryKind::Version)?;
        if !output.status.success() {
            return Ok(None);
        }

        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        Ok((!text.is_empty()).then_some(text))
    }

    pub fn get_dependencies(
        &self,
        manifest: &Manifest,
        environment: &Environment,
    ) -> Result<Vec<String>, InstallerError> {
        let package = manifest
            .package_for_environment(environment)
            .ok_or(InstallerError::NoPackage)?;
        let package_id = package_id_for_manifest(manifest);
        let output = self.run_query(
            package.package_manager,
            &package_id,
            QueryKind::Dependencies,
        )?;
        if !output.status.success() {
            return Ok(Vec::new());
        }

        Ok(String::from_utf8_lossy(&output.stdout)
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty())
            .map(|line| line.to_string())
            .collect())
    }

    pub fn get_history(&self) -> Result<Vec<HistoryEntry>, InstallerError> {
        self.read_history()
    }

    pub fn cache_info(&self) -> Result<CacheInfo, InstallerError> {
        let mut file_count = 0usize;
        let mut total_bytes = 0u64;

        if self.cache_dir.exists() {
            for entry in fs::read_dir(&self.cache_dir).map_err(|source| InstallerError::Io {
                path: self.cache_dir.clone(),
                source,
            })? {
                let entry = entry.map_err(|source| InstallerError::Io {
                    path: self.cache_dir.clone(),
                    source,
                })?;
                let metadata = entry.metadata().map_err(|source| InstallerError::Io {
                    path: entry.path(),
                    source,
                })?;
                if metadata.is_file() {
                    file_count += 1;
                    total_bytes += metadata.len();
                }
            }
        }

        Ok(CacheInfo {
            file_count,
            total_bytes,
        })
    }

    pub fn reinstall(
        &self,
        manifest: &Manifest,
        environment: &Environment,
    ) -> Result<InstallOutcome, InstallerError> {
        self.install(manifest, environment)
    }

    pub fn clear_cache(&self) -> Result<(), InstallerError> {
        if self.cache_dir.exists() {
            self.cache_dir
                .read_dir()
                .map_err(|source| InstallerError::CacheIo {
                    path: self.cache_dir.clone(),
                    source,
                })?
                .filter_map(|e| e.ok())
                .filter(|e| e.file_type().is_ok_and(|t| !t.is_symlink()))
                .for_each(|e| {
                    let path = e.path();
                    if path.is_dir() {
                        let _ = fs::remove_dir_all(&path);
                    } else {
                        let _ = fs::remove_file(&path);
                    }
                });
        }

        fs::create_dir_all(&self.cache_dir).map_err(|source| InstallerError::CacheIo {
            path: self.cache_dir.clone(),
            source,
        })
    }

    fn install_with_package_manager(
        &self,
        package_manager: PackageManager,
        staged_path: &Path,
        _package_id: &str,
    ) -> Result<String, InstallerError> {
        let Some(adapter) = adapter_for(package_manager) else {
            return Err(InstallerError::UnsupportedManager(package_manager));
        };
        let (command, args) = adapter.install_command(&staged_path.display().to_string());
        self.run_command(&command, &args)?;
        Ok(format!("{command} {}", args.join(" ")))
    }

    fn remove_with_package_manager(
        &self,
        manager: PackageManager,
        package_id: &str,
    ) -> Result<String, InstallerError> {
        let Some(adapter) = adapter_for(manager) else {
            return Err(InstallerError::UnsupportedManager(manager));
        };
        let (command, args) = adapter.remove_command(package_id);
        self.run_command(&command, &args)?;
        Ok(format!("{command} {}", args.join(" ")))
    }

    fn run_query(
        &self,
        manager: PackageManager,
        package_id: &str,
        kind: QueryKind,
    ) -> Result<Output, InstallerError> {
        let Some(adapter) = adapter_for(manager) else {
            return Err(InstallerError::UnsupportedManager(manager));
        };
        let (command, args) = match kind {
            QueryKind::Installed => adapter.query_installed_command(package_id),
            QueryKind::Version => adapter.query_version_command(package_id),
            QueryKind::Dependencies => adapter.query_dependencies_command(package_id),
        };

        self.run_command_capture(&command, &args)
    }

    fn run_command(&self, command: &str, args: &[String]) -> Result<(), InstallerError> {
        let status = Command::new(command)
            .args(args)
            .status()
            .map_err(|source| InstallerError::Command {
                command: command.to_string(),
                source,
            })?;

        if !status.success() {
            return Err(InstallerError::CommandStatus {
                command: command.to_string(),
                status: status.code().unwrap_or(-1),
            });
        }

        Ok(())
    }

    fn run_command_capture(
        &self,
        command: &str,
        args: &[String],
    ) -> Result<Output, InstallerError> {
        Command::new(command)
            .args(args)
            .output()
            .map_err(|source| InstallerError::Command {
                command: command.to_string(),
                source,
            })
    }

    fn run_command_streaming(
        &self,
        command: &str,
        args: &[String],
        tx: &mpsc::Sender<InstallProgress>,
    ) -> Result<(), InstallerError> {
        let mut child = Command::new(command)
            .args(args)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|source| InstallerError::Command {
                command: command.to_string(),
                source,
            })?;

        let stdout = child.stdout.take();
        let stderr = child.stderr.take();

        if let Some(stdout) = stdout {
            let reader = BufReader::new(stdout);
            for text in reader.lines().map_while(Result::ok) {
                let _ = tx.send(InstallProgress {
                    downloaded_bytes: 0,
                    total_bytes: 0,
                    speed_bytes_per_sec: 0.0,
                    log_line: Some(text),
                    stage: InstallStage::Installing,
                });
            }
        }

        if let Some(stderr) = stderr {
            let reader = BufReader::new(stderr);
            for text in reader.lines().map_while(Result::ok) {
                let _ = tx.send(InstallProgress {
                    downloaded_bytes: 0,
                    total_bytes: 0,
                    speed_bytes_per_sec: 0.0,
                    log_line: Some(format!("stderr: {text}")),
                    stage: InstallStage::Installing,
                });
            }
        }

        let status = child.wait().map_err(|source| InstallerError::Command {
            command: command.to_string(),
            source,
        })?;

        if !status.success() {
            return Err(InstallerError::CommandStatus {
                command: command.to_string(),
                status: status.code().unwrap_or(-1),
            });
        }

        Ok(())
    }

    fn stage_package(
        &self,
        manifest: &Manifest,
        package: &ResolvedPackage<'_>,
        progress_tx: Option<&mpsc::Sender<InstallProgress>>,
    ) -> Result<PathBuf, InstallerError> {
        fs::create_dir_all(&self.cache_dir).map_err(|source| InstallerError::CacheIo {
            path: self.cache_dir.clone(),
            source,
        })?;

        let reference = package.reference;
        if let Some(url) = reference
            .strip_prefix("https://")
            .or_else(|| reference.strip_prefix("http://"))
        {
            if reference.contains("github.com/") {
                let suffix = asset_suffix_for_package(package);
                let asset = resolve_latest_release_asset(reference, suffix)
                    .map_err(|error| InstallerError::UnsupportedReference(error.to_string()))?;
                let destination = self
                    .cache_dir
                    .join(package_file_name(manifest, &asset.name));
                self.download(&asset.download_url, &destination, progress_tx)?;
                return Ok(destination);
            }

            let destination = self.cache_dir.join(package_file_name(manifest, reference));
            self.download(url, &destination, progress_tx)?;
            return Ok(destination);
        }

        let source_path = Path::new(reference);
        if source_path.exists() {
            let destination = self.cache_dir.join(
                source_path
                    .file_name()
                    .map(|name| name.to_owned())
                    .unwrap_or_else(|| package_file_name(manifest, reference).into()),
            );
            if source_path != destination {
                fs::copy(source_path, &destination).map_err(|source| InstallerError::Io {
                    path: source_path.to_path_buf(),
                    source,
                })?;
            }
            return Ok(destination);
        }

        Err(InstallerError::UnsupportedReference(reference.to_string()))
    }

    fn download(
        &self,
        url: &str,
        destination: &Path,
        progress_tx: Option<&mpsc::Sender<InstallProgress>>,
    ) -> Result<(), InstallerError> {
        let client = Client::builder()
            .build()
            .map_err(|source| InstallerError::Download {
                url: url.to_string(),
                source,
            })?;
        let response = client
            .get(url)
            .send()
            .and_then(|response| response.error_for_status())
            .map_err(|source| InstallerError::Download {
                url: url.to_string(),
                source,
            })?;

        let total = response.content_length().unwrap_or(0);
        let bytes = response
            .bytes()
            .map_err(|source| InstallerError::Download {
                url: url.to_string(),
                source,
            })?;

        let downloaded = bytes.len() as u64;
        if let Some(tx) = progress_tx {
            let _ = tx.send(InstallProgress {
                downloaded_bytes: downloaded,
                total_bytes: total.max(downloaded),
                speed_bytes_per_sec: 0.0,
                log_line: None,
                stage: InstallStage::Downloading,
            });
        }

        fs::write(destination, &bytes).map_err(|source| InstallerError::Io {
            path: destination.to_path_buf(),
            source,
        })
    }

    fn verify_sha256_if_present(
        &self,
        manifest: &Manifest,
        staged_path: &Path,
    ) -> Result<bool, InstallerError> {
        let Some(expected) = manifest.sha256.as_deref() else {
            return Ok(false);
        };

        let expected = normalize_sha256(expected);
        let actual = sha256_of_file(staged_path)?;
        if actual != expected {
            return Err(InstallerError::Sha256Mismatch {
                path: staged_path.to_path_buf(),
                expected,
                actual,
            });
        }

        Ok(true)
    }

    fn verify_signature_if_present(
        &self,
        manifest: &Manifest,
        staged_path: &Path,
    ) -> Result<Option<bool>, InstallerError> {
        let Some(signature) = manifest.signature.as_deref() else {
            return Ok(None);
        };

        let parsed = SignatureSpec::parse(signature)
            .map_err(|error| InstallerError::Signature(error.to_string()))?;
        parsed
            .verify_file(staged_path)
            .map(|()| Some(true))
            .map_err(|error| InstallerError::Signature(error.to_string()))
    }

    fn append_history(&self, entry: &HistoryEntry) -> Result<(), InstallerError> {
        let mut history = self.read_history()?;
        history.push(entry.clone());
        let json = serde_json::to_vec_pretty(&history)
            .map_err(|error| InstallerError::History(error.to_string()))?;
        fs::create_dir_all(&self.cache_dir).map_err(|source| InstallerError::CacheIo {
            path: self.cache_dir.clone(),
            source,
        })?;
        let tmp_path = self.history_path.with_extension("json.tmp");
        {
            let mut tmp = fs::File::create(&tmp_path).map_err(|source| InstallerError::Io {
                path: tmp_path.clone(),
                source,
            })?;
            tmp.write_all(&json).map_err(|source| InstallerError::Io {
                path: tmp_path.clone(),
                source,
            })?;
            tmp.sync_all().map_err(|source| InstallerError::Io {
                path: tmp_path.clone(),
                source,
            })?;
        }
        fs::rename(&tmp_path, &self.history_path).map_err(|source| InstallerError::Io {
            path: self.history_path.clone(),
            source,
        })
    }

    fn read_history(&self) -> Result<Vec<HistoryEntry>, InstallerError> {
        let Ok(contents) = fs::read_to_string(&self.history_path) else {
            return Ok(Vec::new());
        };

        serde_json::from_str(&contents).map_err(|error| InstallerError::History(error.to_string()))
    }
}

#[derive(Debug, Clone, Copy)]
enum QueryKind {
    Installed,
    Version,
    Dependencies,
}

fn package_id_for_manifest(manifest: &Manifest) -> String {
    slugify(&manifest.name)
}

fn package_file_name(manifest: &Manifest, reference: &str) -> String {
    let candidate = reference
        .rsplit('/')
        .next()
        .unwrap_or(reference)
        .split('?')
        .next()
        .unwrap_or(reference);
    if candidate.contains('.') && !candidate.ends_with('.') {
        candidate.to_string()
    } else {
        format!("{}-{}.pkg", slugify(&manifest.name), manifest.version)
    }
}

fn asset_suffix_for_package(package: &ResolvedPackage<'_>) -> &'static str {
    match package.slot {
        crate::PackageSlot::Arch => ".pkg.tar.zst",
        crate::PackageSlot::Ubuntu => ".deb",
        crate::PackageSlot::Fedora => ".rpm",
        crate::PackageSlot::OpenSuse => ".rpm",
        crate::PackageSlot::Fallback => ".AppImage",
    }
}

fn slugify(input: &str) -> String {
    let mut output = String::with_capacity(input.len());
    let mut previous_dash = false;

    for ch in input.chars() {
        let normalized = ch.to_ascii_lowercase();
        if normalized.is_ascii_alphanumeric() {
            output.push(normalized);
            previous_dash = false;
        } else if !previous_dash {
            output.push('-');
            previous_dash = true;
        }
    }

    while output.starts_with('-') {
        output.remove(0);
    }
    while output.ends_with('-') {
        output.pop();
    }

    if output.is_empty() {
        "package".to_string()
    } else {
        output
    }
}

fn normalize_sha256(input: &str) -> String {
    input
        .trim()
        .trim_start_matches("sha256:")
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .to_ascii_lowercase()
}

fn sha256_of_file(path: &Path) -> Result<String, InstallerError> {
    let mut file = fs::File::open(path).map_err(|source| InstallerError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 8192];

    loop {
        let read = file
            .read(&mut buffer)
            .map_err(|source| InstallerError::Io {
                path: path.to_path_buf(),
                source,
            })?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }

    Ok(hasher.finalize().encode_hex::<String>())
}

fn verify_sha256_value(expected: &str, path: &Path) -> Result<(), InstallerError> {
    let expected = normalize_sha256(expected);
    let actual = sha256_of_file(path)?;
    if actual != expected {
        return Err(InstallerError::Sha256Mismatch {
            path: path.to_path_buf(),
            expected,
            actual,
        });
    }

    Ok(())
}

fn current_unix_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or(0)
}

fn default_cache_dir() -> PathBuf {
    if let Some(cache_home) = std::env::var_os("XDG_CACHE_HOME") {
        return PathBuf::from(cache_home).join("openinstall");
    }

    if let Some(local_app_data) = std::env::var_os("LOCALAPPDATA") {
        return PathBuf::from(local_app_data).join("openinstall");
    }

    std::env::temp_dir().join("openinstall")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use std::io::Write;

    fn assert_send<T: Send>() {}

    #[test]
    fn install_progress_types_are_send() {
        assert_send::<InstallProgress>();
        assert_send::<InstallStage>();
    }

    #[test]
    fn cache_info_returns_zero_for_empty_dir() {
        let dir = std::env::temp_dir().join("openinstall-test-empty-cache");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp dir");
        let installer = Installer::new(dir.clone());
        let info = installer.cache_info().expect("cache_info should succeed");
        assert_eq!(info.file_count, 0);
        assert_eq!(info.total_bytes, 0);
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn get_history_returns_empty_vec() {
        let dir = std::env::temp_dir().join("openinstall-test-empty-history");
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).expect("create temp dir");
        let installer = Installer::new(dir.clone());
        let entries = installer.get_history().expect("get_history should succeed");
        assert!(entries.is_empty());
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn slugifies_package_id() {
        assert_eq!(
            package_id_for_manifest(&Manifest {
                name: "Cursor Pro".to_string(),
                publisher: "Anysphere".to_string(),
                version: "1.5.0".to_string(),
                description: "AI Code Editor".to_string(),
                homepage: None,
                license: None,
                changelog: None,
                image: None,
                packages: crate::PackageMatrix::default(),
                sha256: None,
                signature: None,
            }),
            "cursor-pro"
        );
    }

    #[test]
    fn computes_sha256_for_file() {
        let file_path = std::env::temp_dir().join("openinstall-sha256-test.txt");
        let mut file = File::create(&file_path).expect("temp file");
        file.write_all(b"hello").expect("write temp file");
        drop(file);

        let hash = sha256_of_file(&file_path).expect("hash");
        assert_eq!(
            hash,
            "2cf24dba5fb0a30e26e83b2ac5b9e29e1b161e5c1fa7425e73043362938b9824"
        );
        let _ = fs::remove_file(file_path);
    }

    #[test]
    fn normalizes_sha256_with_prefix() {
        let result = normalize_sha256("sha256:abc123");
        assert_eq!(result, "abc123");
    }

    #[test]
    fn normalizes_sha256_with_whitespace() {
        let result = normalize_sha256("  AbC 123 ");
        assert_eq!(result, "abc123");
    }

    #[test]
    fn normalizes_sha256_uppercase() {
        let result = normalize_sha256("ABCDEF");
        assert_eq!(result, "abcdef");
    }

    #[test]
    fn extracts_package_filename_from_url() {
        let manifest = Manifest {
            name: "Test".to_string(),
            publisher: "Pub".to_string(),
            version: "1.0".to_string(),
            description: "desc".to_string(),
            homepage: None,
            license: None,
            changelog: None,
            image: None,
            packages: crate::PackageMatrix::default(),
            sha256: None,
            signature: None,
        };
        let name = package_file_name(&manifest, "https://example.com/app.deb");
        assert_eq!(name, "app.deb");
    }

    #[test]
    fn package_file_name_fallback_without_extension() {
        let manifest = Manifest {
            name: "My App".to_string(),
            publisher: "Pub".to_string(),
            version: "2.0".to_string(),
            description: "desc".to_string(),
            homepage: None,
            license: None,
            changelog: None,
            image: None,
            packages: crate::PackageMatrix::default(),
            sha256: None,
            signature: None,
        };
        let name = package_file_name(&manifest, "my-app");
        assert_eq!(name, "my-app-2.0.pkg");
    }

    #[test]
    fn slugify_mixed_case() {
        let result = slugify("Hello World App");
        assert_eq!(result, "hello-world-app");
    }

    #[test]
    fn slugify_special_chars() {
        let result = slugify("Test@App#1");
        assert_eq!(result, "test-app-1");
    }

    #[test]
    fn slugify_multiple_dashes() {
        let result = slugify("foo___bar");
        assert_eq!(result, "foo-bar");
    }

    #[test]
    fn asset_suffix_for_arch() {
        let pkg = ResolvedPackage {
            slot: crate::PackageSlot::Arch,
            package_manager: PackageManager::Pacman,
            reference: "https://example.com/pkg",
        };
        assert_eq!(asset_suffix_for_package(&pkg), ".pkg.tar.zst");
    }

    #[test]
    fn asset_suffix_for_ubuntu() {
        let pkg = ResolvedPackage {
            slot: crate::PackageSlot::Ubuntu,
            package_manager: PackageManager::Apt,
            reference: "https://example.com/pkg",
        };
        assert_eq!(asset_suffix_for_package(&pkg), ".deb");
    }

    #[test]
    fn asset_suffix_for_fedora_and_opensuse() {
        let fed = ResolvedPackage {
            slot: crate::PackageSlot::Fedora,
            package_manager: PackageManager::Dnf,
            reference: "",
        };
        let suse = ResolvedPackage {
            slot: crate::PackageSlot::OpenSuse,
            package_manager: PackageManager::Zypper,
            reference: "",
        };
        assert_eq!(asset_suffix_for_package(&fed), ".rpm");
        assert_eq!(asset_suffix_for_package(&suse), ".rpm");
    }

    #[test]
    fn asset_suffix_for_fallback() {
        let pkg = ResolvedPackage {
            slot: crate::PackageSlot::Fallback,
            package_manager: PackageManager::PackageKit,
            reference: "",
        };
        assert_eq!(asset_suffix_for_package(&pkg), ".AppImage");
    }

    #[test]
    fn current_unix_secs_is_non_zero() {
        assert!(current_unix_secs() > 1_700_000_000);
    }
}
