use url::Url;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallUri {
    pub scheme: String,
    pub app_id: String,
    /// Optional manifest URL extracted from query parameter `?m=` or `?manifest=`.
    /// Present when the URI carries a direct link to the app's manifest.json.
    pub manifest_url: Option<Url>,
}

#[derive(Debug, Error)]
pub enum InstallUriError {
    #[error("invalid install uri")]
    Invalid,
    #[error("unsupported uri scheme: {0}")]
    UnsupportedScheme(String),
    #[error("missing application identifier")]
    MissingAppId,
}

impl InstallUri {
    pub fn parse(input: &str) -> Result<Self, InstallUriError> {
        let input = input.trim();
        let (scheme, remainder) = input.split_once("://").ok_or(InstallUriError::Invalid)?;
        let scheme = scheme.to_ascii_lowercase();
        match scheme.as_str() {
            "linuxinstall" | "openinstall" => {
                let app_id = remainder
                    .split(|c: char| c == '/' || c == '?')
                    .next()
                    .unwrap_or("")
                    .trim();
                if app_id.is_empty() {
                    return Err(InstallUriError::MissingAppId);
                }

                let manifest_url = remainder
                    .split('?')
                    .nth(1)
                    .and_then(Self::parse_manifest_query);

                Ok(Self {
                    scheme,
                    app_id: app_id.to_string(),
                    manifest_url,
                })
            }
            other => Err(InstallUriError::UnsupportedScheme(other.to_string())),
        }
    }

    fn parse_manifest_query(query: &str) -> Option<Url> {
        for pair in query.split('&') {
            let pair = pair.trim();
            if let Some((key, value)) = pair.split_once('=') {
                if matches!(key.trim(), "m" | "manifest") {
                    let decoded = Self::percent_decode(value.trim());
                    return Url::parse(&decoded).ok();
                }
            }
        }
        None
    }

    fn percent_decode(input: &str) -> String {
        let mut result = String::with_capacity(input.len());
        let mut chars = input.chars();
        while let Some(c) = chars.next() {
            if c == '%' {
                let hex: String = chars.by_ref().take(2).collect();
                if hex.len() == 2 {
                    if let Ok(byte) = u8::from_str_radix(&hex, 16) {
                        result.push(byte as char);
                        continue;
                    }
                }
                result.push('%');
                result.push_str(&hex);
            } else if c == '+' {
                result.push(' ');
            } else {
                result.push(c);
            }
        }
        result
    }

    /// Returns true if this URI includes a manifest URL for direct installation.
    pub fn has_manifest(&self) -> bool {
        self.manifest_url.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_known_install_uri() {
        let uri = InstallUri::parse("openinstall://cursor").expect("uri should parse");
        assert_eq!(uri.scheme, "openinstall");
        assert_eq!(uri.app_id, "cursor");
        assert!(uri.manifest_url.is_none());
    }

    #[test]
    fn parses_linuxinstall_scheme() {
        let uri = InstallUri::parse("linuxinstall://cursor").expect("uri should parse");
        assert_eq!(uri.scheme, "linuxinstall");
        assert_eq!(uri.app_id, "cursor");
    }

    #[test]
    fn rejects_unknown_scheme() {
        let error = InstallUri::parse("example://cursor").expect_err("uri should fail");
        assert!(matches!(error, InstallUriError::UnsupportedScheme(_)));
    }

    #[test]
    fn parses_uri_with_manifest_url_via_m() {
        let uri = InstallUri::parse("openinstall://cursor?m=https://example.com/manifest.json")
            .expect("uri should parse");
        assert_eq!(uri.scheme, "openinstall");
        assert_eq!(uri.app_id, "cursor");
        assert_eq!(
            uri.manifest_url.as_ref().map(|u| u.as_str()),
            Some("https://example.com/manifest.json")
        );
    }

    #[test]
    fn parses_uri_with_manifest_url_via_manifest_key() {
        let uri =
            InstallUri::parse("openinstall://cursor?manifest=https://example.com/manifest.json")
                .expect("uri should parse");
        assert_eq!(uri.app_id, "cursor");
        assert!(uri.manifest_url.is_some());
    }

    #[test]
    fn parses_uri_without_manifest_returns_none() {
        let uri = InstallUri::parse("openinstall://cursor").expect("uri should parse");
        assert!(uri.manifest_url.is_none());
        assert!(!uri.has_manifest());
    }

    #[test]
    fn parses_uri_with_multiple_query_params() {
        let uri = InstallUri::parse(
            "openinstall://cursor?m=https://example.com/manifest.json&foo=bar",
        )
        .expect("uri should parse");
        assert_eq!(uri.app_id, "cursor");
        assert_eq!(
            uri.manifest_url.as_ref().map(|u| u.as_str()),
            Some("https://example.com/manifest.json")
        );
    }

    #[test]
    fn ignores_path_after_app_id() {
        let uri =
            InstallUri::parse("openinstall://cursor/extra/path?m=https://example.com/manifest.json")
                .expect("uri should parse");
        assert_eq!(uri.app_id, "cursor");
        assert!(uri.manifest_url.is_some());
    }

    #[test]
    fn rejects_empty_app_id() {
        let error = InstallUri::parse("openinstall://").expect_err("should fail");
        assert!(matches!(error, InstallUriError::MissingAppId));
    }

    #[test]
    fn rejects_invalid_scheme() {
        let error = InstallUri::parse("notavaliduri").expect_err("should fail");
        assert!(matches!(error, InstallUriError::Invalid));
    }

    #[test]
    fn handles_percent_encoded_manifest_url() {
        let uri = InstallUri::parse(
            "openinstall://cursor?m=https%3A%2F%2Fexample.com%2Fmanifest.json",
        )
        .expect("uri should parse");
        assert_eq!(uri.app_id, "cursor");
        assert_eq!(
            uri.manifest_url.as_ref().map(|u| u.as_str()),
            Some("https://example.com/manifest.json")
        );
    }

    #[test]
    fn ignores_invalid_manifest_url_value() {
        let uri = InstallUri::parse("openinstall://cursor?m=not-a-valid-url")
            .expect("uri should parse");
        assert_eq!(uri.app_id, "cursor");
        assert!(uri.manifest_url.is_none());
    }

    #[test]
    fn handles_empty_manifest_param() {
        let uri = InstallUri::parse("openinstall://cursor?m=").expect("uri should parse");
        assert_eq!(uri.app_id, "cursor");
        assert!(uri.manifest_url.is_none());
    }

    #[test]
    fn linuxinstall_scheme_with_manifest() {
        let uri =
            InstallUri::parse("linuxinstall://cursor?m=https://example.com/manifest.json")
                .expect("uri should parse");
        assert_eq!(uri.scheme, "linuxinstall");
        assert_eq!(uri.app_id, "cursor");
        assert!(uri.manifest_url.is_some());
    }

    #[test]
    fn has_manifest_returns_true_when_present() {
        let uri =
            InstallUri::parse("openinstall://cursor?m=https://example.com/manifest.json")
                .expect("uri should parse");
        assert!(uri.has_manifest());
    }
}
