use serde::Deserialize;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GitHubReleaseAsset {
    pub name: String,
    pub download_url: String,
}

#[derive(Debug, Error)]
pub enum GitHubReleaseError {
    #[error("invalid github repository url")]
    InvalidRepositoryUrl,
    #[error("unsupported github repository host: {0}")]
    UnsupportedHost(String),
    #[error("failed to fetch github release: {0}")]
    Http(#[from] reqwest::Error),
    #[error("failed to parse github release: {0}")]
    Parse(#[from] serde_json::Error),
    #[error("no release asset with suffix {0} was found")]
    MissingAsset(String),
}

#[derive(Debug, Deserialize)]
struct ReleaseResponse {
    assets: Vec<ReleaseAsset>,
}

#[derive(Debug, Deserialize)]
struct ReleaseAsset {
    name: String,
    browser_download_url: String,
}

pub fn resolve_latest_release_asset(
    repository_url: &str,
    suffix: &str,
) -> Result<GitHubReleaseAsset, GitHubReleaseError> {
    let (owner, repo) = parse_repository_url(repository_url)?;
    let client = reqwest::blocking::Client::builder()
        .user_agent("openinstall/0.1.0")
        .build()?;
    let response = client
        .get(format!(
            "https://api.github.com/repos/{owner}/{repo}/releases/latest"
        ))
        .header("Accept", "application/vnd.github+json")
        .send()?
        .error_for_status()?;
    let release: ReleaseResponse = serde_json::from_reader(response)?;
    pick_release_asset(&release, suffix)
}

fn parse_repository_url(input: &str) -> Result<(String, String), GitHubReleaseError> {
    let input = input.trim();
    let without_scheme = input
        .strip_prefix("https://")
        .or_else(|| input.strip_prefix("http://"))
        .ok_or(GitHubReleaseError::InvalidRepositoryUrl)?;
    let without_host = without_scheme.strip_prefix("github.com/").ok_or_else(|| {
        GitHubReleaseError::UnsupportedHost(
            without_scheme.split('/').next().unwrap_or("").to_string(),
        )
    })?;

    let mut parts = without_host.split('/').filter(|part| !part.is_empty());
    let owner = parts
        .next()
        .ok_or(GitHubReleaseError::InvalidRepositoryUrl)?;
    let repo = parts
        .next()
        .ok_or(GitHubReleaseError::InvalidRepositoryUrl)?;

    Ok((owner.to_string(), repo.trim_end_matches(".git").to_string()))
}

fn pick_release_asset(
    release: &ReleaseResponse,
    suffix: &str,
) -> Result<GitHubReleaseAsset, GitHubReleaseError> {
    release
        .assets
        .iter()
        .find(|asset| asset.name.ends_with(suffix))
        .map(|asset| GitHubReleaseAsset {
            name: asset.name.clone(),
            download_url: asset.browser_download_url.clone(),
        })
        .ok_or_else(|| GitHubReleaseError::MissingAsset(suffix.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_repository_url() {
        let (owner, repo) = parse_repository_url("https://github.com/openai/codex").unwrap();
        assert_eq!(owner, "openai");
        assert_eq!(repo, "codex");
    }

    #[test]
    fn picks_matching_asset() {
        let release = ReleaseResponse {
            assets: vec![
                ReleaseAsset {
                    name: "app.zip".to_string(),
                    browser_download_url: "https://example.com/app.zip".to_string(),
                },
                ReleaseAsset {
                    name: "app.deb".to_string(),
                    browser_download_url: "https://example.com/app.deb".to_string(),
                },
            ],
        };

        let asset = pick_release_asset(&release, ".deb").unwrap();
        assert_eq!(asset.name, "app.deb");
        assert_eq!(asset.download_url, "https://example.com/app.deb");
    }
}
