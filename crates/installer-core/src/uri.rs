use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InstallUri {
    pub scheme: String,
    pub app_id: String,
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
                    .split('/')
                    .next()
                    .unwrap_or("")
                    .split('?')
                    .next()
                    .unwrap_or("")
                    .trim();
                if app_id.is_empty() {
                    return Err(InstallUriError::MissingAppId);
                }

                Ok(Self {
                    scheme,
                    app_id: app_id.to_string(),
                })
            }
            other => Err(InstallUriError::UnsupportedScheme(other.to_string())),
        }
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
    }

    #[test]
    fn rejects_unknown_scheme() {
        let error = InstallUri::parse("example://cursor").expect_err("uri should fail");
        assert!(matches!(error, InstallUriError::UnsupportedScheme(_)));
    }
}
