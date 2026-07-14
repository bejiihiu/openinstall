pub mod adapters;
pub mod api;
pub mod constants;
pub mod desktop;
pub mod environment;
pub mod github;
pub mod manifest;
pub mod matrix;
pub mod publish;
pub mod runtime;
pub mod signature;
pub mod uri;

pub use adapters::{adapter_for, PackageAdapter};
pub use api::{serve_latest_app, ApiError, LatestAppResponse};
pub use desktop::{desktop_entry, desktop_entry_for_install_uri};
pub use environment::{Environment, PackageManager};
pub use github::{resolve_latest_release_asset, GitHubReleaseAsset};
pub use manifest::{Manifest, ManifestError, ResolvedPackage, Scripts};
pub use matrix::{PackageMatrix, PackageSlot};
pub use publish::{PublishError, PublishSpec};
pub use runtime::{
    CacheInfo, HistoryEntry, InstallOutcome, InstallProgress, InstallStage, InstallationState,
    Installer, InstallerError, VerificationOutcome,
};
pub use signature::{SignatureError, SignatureSpec};
pub use uri::{InstallUri, InstallUriError};
