//! Model catalog, Hugging Face resolve URLs, resumable downloads, SHA-256
//! verification, and cache paths (plan §7.7, §12).
//!
//! Downloads are public and tokenless (INV-009). A file without a published
//! SHA-256 is refused unless a developer explicitly opts into unverified files.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const DISK_PREFLIGHT_BYTES: u64 = 6_000_000_000;

#[derive(Debug, Error)]
pub enum ModelError {
    #[error("model file is not pinned with a SHA-256: {0}")]
    Unpinned(String),
    #[error("hash mismatch for {file}: expected {expected}, got {actual}")]
    HashMismatch {
        file: String,
        expected: String,
        actual: String,
    },
    #[error("insufficient disk space: need {needed} bytes, have {available}")]
    DiskSpace { needed: u64, available: u64 },
    #[error("download failed for {url}: {message}")]
    Download { url: String, message: String },
    #[error("io error: {0}")]
    Io(String),
    #[error("invalid catalog: {0}")]
    Catalog(String),
}

impl From<std::io::Error> for ModelError {
    fn from(error: std::io::Error) -> Self {
        ModelError::Io(error.to_string())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelRole {
    Asr,
    Llm,
    Vad,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelFile {
    pub path: String,
    pub size: u64,
    pub sha256: Option<String>,
}

impl ModelFile {
    pub fn pinned(path: impl Into<String>, size: u64, sha256: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            size,
            sha256: Some(sha256.into()),
        }
    }

    pub fn unpinned(path: impl Into<String>, size: u64) -> Self {
        Self {
            path: path.into(),
            size,
            sha256: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelArtifact {
    pub id: String,
    pub role: ModelRole,
    pub repo: String,
    pub revision: String,
    pub files: Vec<ModelFile>,
    pub license: String,
}

impl ModelArtifact {
    pub fn total_bytes(&self) -> u64 {
        self.files.iter().map(|file| file.size).sum()
    }

    pub fn is_pinned(&self) -> bool {
        !self.files.is_empty() && self.files.iter().all(|file| file.sha256.is_some())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelCatalog {
    pub artifacts: Vec<ModelArtifact>,
}

impl ModelCatalog {
    /// The plan's default artifacts. Hashes stay empty until the publish step
    /// generates them (INV-008); the downloader refuses unpinned files by
    /// default rather than trusting first use.
    pub fn plan_default() -> Self {
        Self {
            artifacts: vec![
                ModelArtifact {
                    id: "asr-nemotron-0.6b".to_string(),
                    role: ModelRole::Asr,
                    repo: "ndgold/nemotron-3.5-asr-streaming-0.6b-onnx".to_string(),
                    revision: "main".to_string(),
                    files: vec![
                        ModelFile::unpinned("encoder.int8.onnx", 500_000_000),
                        ModelFile::unpinned("decoder.onnx", 60_000_000),
                        ModelFile::unpinned("joiner.onnx", 40_000_000),
                        ModelFile::unpinned("tokens.txt", 100_000),
                    ],
                    license: "resolve upstream before publishing".to_string(),
                },
                ModelArtifact {
                    id: "llm-qwen3-0.6b-easylanguage".to_string(),
                    role: ModelRole::Llm,
                    repo: "ndgold/Qwen3-0.6B-EasyLanguage-GGUF".to_string(),
                    revision: "main".to_string(),
                    files: vec![ModelFile::unpinned(
                        "Qwen3-0.6B-EasyLanguage-Q4_K_M.gguf",
                        400_000_000,
                    )],
                    license: "Apache-2.0".to_string(),
                },
                ModelArtifact {
                    id: "llm-qwen3-1.7b-easylanguage".to_string(),
                    role: ModelRole::Llm,
                    repo: "ndgold/Qwen3-1.7B-EasyLanguage-GGUF".to_string(),
                    revision: "main".to_string(),
                    files: vec![ModelFile::unpinned(
                        "Qwen3-1.7B-EasyLanguage-Q4_K_M.gguf",
                        1_000_000_000,
                    )],
                    license: "Apache-2.0".to_string(),
                },
                ModelArtifact {
                    id: "vad-silero".to_string(),
                    role: ModelRole::Vad,
                    repo: "k2-fsa/sherpa-onnx".to_string(),
                    revision: "v1.12.0".to_string(),
                    files: vec![ModelFile::unpinned("silero_vad.onnx", 2_000_000)],
                    license: "MIT (verify upstream)".to_string(),
                },
            ],
        }
    }

    pub fn get(&self, id: &str) -> Option<&ModelArtifact> {
        self.artifacts.iter().find(|artifact| artifact.id == id)
    }

    pub fn from_json(json: &str) -> Result<Self, ModelError> {
        serde_json::from_str(json).map_err(|error| ModelError::Catalog(error.to_string()))
    }

    pub fn to_json(&self) -> Result<String, ModelError> {
        serde_json::to_string_pretty(self).map_err(|error| ModelError::Catalog(error.to_string()))
    }

    pub fn unpinned(&self) -> Vec<&ModelArtifact> {
        self.artifacts
            .iter()
            .filter(|artifact| !artifact.is_pinned())
            .collect()
    }
}

/// Hugging Face resolve URL. Tokenless and pinned by revision.
pub fn resolve_url(repo: &str, revision: &str, file: &str) -> String {
    format!(
        "https://huggingface.co/{repo}/resolve/{revision}/{}",
        file.trim_start_matches('/')
    )
}

/// Model cache root for the current platform.
pub fn cache_root() -> PathBuf {
    if let Ok(path) = std::env::var("LL_MODELS_DIR") {
        return PathBuf::from(path);
    }
    cache_root_from(
        std::env::var("XDG_DATA_HOME").ok(),
        std::env::var("LOCALAPPDATA").ok(),
        std::env::var("HOME").ok(),
        std::env::consts::OS,
    )
}

fn cache_root_from(
    xdg_data_home: Option<String>,
    local_app_data: Option<String>,
    home: Option<String>,
    os: &str,
) -> PathBuf {
    match os {
        "windows" => PathBuf::from(local_app_data.unwrap_or_default()).join("LiveLinguist/models"),
        "macos" => PathBuf::from(home.unwrap_or_default())
            .join("Library/Application Support/LiveLinguist/models"),
        _ => PathBuf::from(
            xdg_data_home.unwrap_or_else(|| format!("{}/.local/share", home.unwrap_or_default())),
        )
        .join("livelinguist/models"),
    }
}

pub fn model_dir(root: &Path, repo: &str, revision: &str) -> PathBuf {
    root.join(repo).join(revision)
}

/// SHA-256 of a file as lowercase hex.
pub fn sha256_file(path: &Path) -> Result<String, ModelError> {
    let mut file = File::open(path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = file.read(&mut buffer)?;
        if read == 0 {
            break;
        }
        hasher.update(&buffer[..read]);
    }
    Ok(format!("{:x}", hasher.finalize()))
}

/// Check free space at `path` against the required budget.
pub fn disk_preflight(path: &Path, required: u64) -> Result<u64, ModelError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let available =
        fs2::available_space(path).unwrap_or_else(|_| fs2::available_space("/").unwrap_or(0));
    if available < required {
        return Err(ModelError::DiskSpace {
            needed: required,
            available,
        });
    }
    Ok(available)
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DownloadProgress {
    pub repo: String,
    pub file: String,
    pub received: u64,
    pub total: u64,
    pub bytes_per_sec: u64,
    pub eta_ms: u64,
}

/// Download one pinned file with Range resume, retries, and SHA-256 verify.
pub fn download_file(
    client: &reqwest::blocking::Client,
    artifact: &ModelArtifact,
    file: &ModelFile,
    destination: &Path,
    allow_unverified: bool,
    on_progress: &mut dyn FnMut(DownloadProgress),
) -> Result<PathBuf, ModelError> {
    let expected = match (&file.sha256, allow_unverified) {
        (Some(hash), _) => Some(hash.clone()),
        (None, true) => None,
        (None, false) => return Err(ModelError::Unpinned(file.path.clone())),
    };
    std::fs::create_dir_all(destination.parent().unwrap_or(destination))?;
    if destination.exists() {
        if let Some(expected) = &expected {
            if sha256_file(destination)?.eq_ignore_ascii_case(expected) {
                return Ok(destination.to_path_buf());
            }
        } else {
            return Ok(destination.to_path_buf());
        }
    }
    let url = resolve_url(&artifact.repo, &artifact.revision, &file.path);
    let mut attempt = 0u32;
    loop {
        attempt += 1;
        match download_once(
            client,
            &url,
            destination,
            file.size,
            artifact,
            file,
            on_progress,
        ) {
            Ok(()) => break,
            Err(error) if attempt < 3 => {
                let _ = error;
                std::thread::sleep(std::time::Duration::from_millis(250 * attempt as u64));
            }
            Err(error) => return Err(error),
        }
    }
    if let Some(expected) = expected {
        let actual = sha256_file(destination)?;
        if !actual.eq_ignore_ascii_case(&expected) {
            let _ = std::fs::remove_file(destination);
            return Err(ModelError::HashMismatch {
                file: file.path.clone(),
                expected,
                actual,
            });
        }
    }
    Ok(destination.to_path_buf())
}

fn download_once(
    client: &reqwest::blocking::Client,
    url: &str,
    destination: &Path,
    expected_size: u64,
    artifact: &ModelArtifact,
    file: &ModelFile,
    on_progress: &mut dyn FnMut(DownloadProgress),
) -> Result<(), ModelError> {
    let existing = if destination.exists() {
        std::fs::metadata(destination)?.len()
    } else {
        0
    };
    if existing >= expected_size && expected_size > 0 {
        return Ok(());
    }
    let mut request = client.get(url);
    if existing > 0 {
        request = request.header(reqwest::header::RANGE, format!("bytes={existing}-"));
    }
    let mut response = request.send().map_err(|error| ModelError::Download {
        url: url.to_string(),
        message: error.to_string(),
    })?;
    if !response.status().is_success() {
        return Err(ModelError::Download {
            url: url.to_string(),
            message: format!("HTTP {}", response.status()),
        });
    }
    let append = existing > 0 && response.status() == reqwest::StatusCode::PARTIAL_CONTENT;
    if existing > 0 && !append {
        // Server ignored Range; start over so bytes do not interleave.
        std::fs::remove_file(destination)?;
    }
    let mut sink = if append {
        std::fs::OpenOptions::new().append(true).open(destination)?
    } else {
        File::create(destination)?
    };
    let mut received = if append { existing } else { 0 };
    let started = std::time::Instant::now();
    let mut buffer = [0u8; 64 * 1024];
    loop {
        let read = response
            .read(&mut buffer)
            .map_err(|error| ModelError::Download {
                url: url.to_string(),
                message: error.to_string(),
            })?;
        if read == 0 {
            break;
        }
        sink.write_all(&buffer[..read])?;
        received += read as u64;
        let elapsed = started.elapsed().as_secs_f64().max(0.001);
        let bytes_per_sec = (received as f64 / elapsed) as u64;
        let remaining = expected_size.saturating_sub(received);
        let eta_ms = if bytes_per_sec > 0 {
            (remaining as f64 / bytes_per_sec as f64 * 1000.0) as u64
        } else {
            0
        };
        on_progress(DownloadProgress {
            repo: artifact.repo.clone(),
            file: file.path.clone(),
            received,
            total: expected_size,
            bytes_per_sec,
            eta_ms,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn resolve_url_is_tokenless_and_pinned() {
        assert_eq!(
            resolve_url("org/repo", "abc123", "sub/model.gguf"),
            "https://huggingface.co/org/repo/resolve/abc123/sub/model.gguf"
        );
    }

    #[test]
    fn cache_root_matches_the_plan() {
        assert_eq!(
            cache_root_from(Some("/data".into()), None, Some("/home/u".into()), "linux"),
            PathBuf::from("/data/livelinguist/models")
        );
        assert_eq!(
            cache_root_from(
                None,
                Some("C:\\Users\\u\\AppData\\Local".into()),
                None,
                "windows"
            ),
            PathBuf::from("C:\\Users\\u\\AppData\\Local").join("LiveLinguist/models")
        );
    }

    #[test]
    fn sha256_matches_a_known_vector() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("abc.txt");
        let mut file = File::create(&path).unwrap();
        file.write_all(b"abc").unwrap();
        drop(file);
        assert_eq!(
            sha256_file(&path).unwrap(),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
    }

    #[test]
    fn disk_preflight_rejects_impossible_requirement() {
        let dir = tempfile::tempdir().unwrap();
        assert!(disk_preflight(dir.path(), 1).is_ok());
        assert!(matches!(
            disk_preflight(dir.path(), u64::MAX),
            Err(ModelError::DiskSpace { .. })
        ));
    }

    #[test]
    fn unpinned_catalog_is_flagged_and_downloader_refuses() {
        let catalog = ModelCatalog::plan_default();
        assert_eq!(catalog.unpinned().len(), 4);
        let artifact = catalog.get("llm-qwen3-0.6b-easylanguage").unwrap();
        let client = reqwest::blocking::Client::new();
        let dir = tempfile::tempdir().unwrap();
        let result = download_file(
            &client,
            artifact,
            &artifact.files[0],
            &dir.path().join("model.gguf"),
            false,
            &mut |_| {},
        );
        assert!(matches!(result, Err(ModelError::Unpinned(_))));
    }

    #[test]
    fn catalog_round_trips() {
        let catalog = ModelCatalog::plan_default();
        let json = catalog.to_json().unwrap();
        let restored = ModelCatalog::from_json(&json).unwrap();
        assert_eq!(restored, catalog);
        assert_eq!(
            catalog.get("asr-nemotron-0.6b").unwrap().role,
            ModelRole::Asr
        );
    }

    #[test]
    fn model_dir_uses_repo_and_revision() {
        assert_eq!(
            model_dir(Path::new("/m"), "org/repo", "rev"),
            PathBuf::from("/m/org/repo/rev")
        );
    }
}
