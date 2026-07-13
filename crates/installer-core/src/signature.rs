use std::fs;
use std::path::Path;

use hex::FromHex;
use ring::signature::{self, UnparsedPublicKey};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SignatureSpec {
    pub public_key_hex: String,
    pub signature_hex: String,
}

#[derive(Debug, Error)]
pub enum SignatureError {
    #[error("invalid signature format")]
    InvalidFormat,
    #[error("invalid hex in {field}")]
    InvalidHex { field: &'static str },
    #[error("failed to read signed file {path}: {source}")]
    Io {
        path: std::path::PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("signature verification failed")]
    VerificationFailed,
}

impl SignatureSpec {
    pub fn parse(input: &str) -> Result<Self, SignatureError> {
        let input = input.trim();
        let Some(rest) = input.strip_prefix("ed25519:") else {
            return Err(SignatureError::InvalidFormat);
        };
        let mut parts = rest.split(':');
        let public_key_hex = parts.next().ok_or(SignatureError::InvalidFormat)?.trim();
        let signature_hex = parts.next().ok_or(SignatureError::InvalidFormat)?.trim();
        if parts.next().is_some() || public_key_hex.is_empty() || signature_hex.is_empty() {
            return Err(SignatureError::InvalidFormat);
        }

        Ok(Self {
            public_key_hex: public_key_hex.to_string(),
            signature_hex: signature_hex.to_string(),
        })
    }

    pub fn verify_file(&self, path: &Path) -> Result<(), SignatureError> {
        let public_key =
            Vec::from_hex(&self.public_key_hex).map_err(|_| SignatureError::InvalidHex {
                field: "public_key",
            })?;
        let signature = Vec::from_hex(&self.signature_hex)
            .map_err(|_| SignatureError::InvalidHex { field: "signature" })?;
        let bytes = fs::read(path).map_err(|source| SignatureError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        let verifier = UnparsedPublicKey::new(&signature::ED25519, &public_key);
        verifier
            .verify(&bytes, &signature)
            .map_err(|_| SignatureError::VerificationFailed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_format() {
        assert!(matches!(
            SignatureSpec::parse("abc"),
            Err(SignatureError::InvalidFormat)
        ));
    }
}
