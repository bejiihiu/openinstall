use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::{Manifest, ManifestError, PackageMatrix};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LatestAppResponse {
    pub version: String,
    pub packages: PackageMatrix,
}

#[derive(Debug, Error)]
pub enum ApiError {
    #[error(transparent)]
    Manifest(#[from] ManifestError),
    #[error("failed to bind api server on {address}: {source}")]
    Bind {
        address: String,
        #[source]
        source: std::io::Error,
    },
    #[error("api io error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<&Manifest> for LatestAppResponse {
    fn from(manifest: &Manifest) -> Self {
        Self {
            version: manifest.version.clone(),
            packages: manifest.packages.clone(),
        }
    }
}

pub fn serve_latest_app(
    address: &str,
    manifest: &Manifest,
) -> Result<thread::JoinHandle<()>, ApiError> {
    let listener = TcpListener::bind(address).map_err(|source| ApiError::Bind {
        address: address.to_string(),
        source,
    })?;
    let response = serde_json::to_vec(&LatestAppResponse::from(manifest))
        .expect("latest app response should serialize");
    let handle = thread::spawn(move || {
        for incoming in listener.incoming() {
            match incoming {
                Ok(stream) => {
                    let _ = handle_connection(stream, &response);
                }
                Err(_) => break,
            }
        }
    });

    Ok(handle)
}

fn handle_connection(mut stream: TcpStream, body: &[u8]) -> Result<(), ApiError> {
    let mut request = [0u8; 1024];
    let bytes_read = stream.read(&mut request)?;
    if bytes_read == 0 {
        return Ok(());
    }
    let request_line = String::from_utf8_lossy(&request[..bytes_read]);
    let (method, path, _version) = match parse_request_line(&request_line) {
        Some(parsed) => parsed,
        None => {
            let bad_request =
                b"HTTP/1.1 400 Bad Request\r\nContent-Length: 0\r\nConnection: close\r\n\r\n";
            stream.write_all(bad_request)?;
            stream.flush()?;
            return Ok(());
        }
    };

    let is_latest = method == "GET" && (path == "/app/latest" || path.starts_with("/app/latest?"));

    let not_found = b"{\"error\":\"not found\"}";
    let body_bytes: &[u8] = if is_latest { body } else { not_found };
    let status_line = if is_latest {
        "HTTP/1.1 200 OK\r\n"
    } else {
        "HTTP/1.1 404 Not Found\r\n"
    };

    let headers = format!(
        "{status_line}Content-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        body_bytes.len()
    );
    stream.write_all(headers.as_bytes())?;
    stream.write_all(body_bytes)?;
    stream.flush()?;
    Ok(())
}

fn parse_request_line(line: &str) -> Option<(&str, &str, &str)> {
    let line = line.trim();
    let (method, rest) = line.split_once(' ')?;
    let (path, version) = rest.split_once(' ')?;
    Some((method, path, version))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn latest_app_response_from_manifest() {
        let manifest = Manifest {
            name: "Test".to_string(),
            publisher: "Pub".to_string(),
            version: "2.0".to_string(),
            description: "desc".to_string(),
            homepage: None,
            license: None,
            changelog: None,
            image: None,
            packages: PackageMatrix::default(),
            sha256: None,
            signature: None,
        };
        let resp = LatestAppResponse::from(&manifest);
        assert_eq!(resp.version, "2.0");
        assert_eq!(resp.packages, PackageMatrix::default());
    }
}
