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
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
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
    #[error("download cancelled")]
    Cancelled,
}

impl ModelError {
    /// Stable, content-free code for the UI (`model_error.code`).
    pub fn code(&self) -> &'static str {
        match self {
            ModelError::DiskSpace { .. } => "DISK_FULL",
            ModelError::HashMismatch { .. } | ModelError::Unpinned(_) => "HASH_MISMATCH",
            ModelError::Download { .. } => "NETWORK",
            ModelError::Cancelled => "CANCELLED",
            ModelError::Io(_) => "IO",
            ModelError::Catalog(_) => "CATALOG",
        }
    }
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
    /// Where the mirrored files came from, for audit. Never used to download.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub upstream: Option<String>,
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
    /// The pinned 1.0 model set: one ASR model for all 29 locales and one
    /// simplifier. Both are downloaded on first run and verified before use
    /// (ADR 0019). Every file carries a published SHA-256, so a mirror that
    /// changes fails loudly instead of installing different weights.
    pub fn plan_default() -> Self {
        let asr_files = vec![
            ModelFile::pinned(
                "encoder.int8.onnx",
                657_601_403,
                "012e9321373af99021415e0b0eb3ec827b4be3153be6f30d9b448fe65e896e68",
            ),
            ModelFile::pinned(
                "decoder.int8.onnx",
                14_978_075,
                "19f9c98fc6d0a2c33a65a43b36fdb2e914c26c0aa9764be3aebc502a1e982fb0",
            ),
            ModelFile::pinned(
                "joiner.int8.onnx",
                9_504_438,
                "4101c7c679a0bc30483794b27a059e34e79232aa2068d78d51231a22c8b0d7ce",
            ),
            ModelFile::pinned(
                "tokens.txt",
                131_440,
                "729cc103155bafa785f9cd45746cd41cabe97eab7182fc04d594129587958f8a",
            ),
        ];
        Self {
            artifacts: vec![
                ModelArtifact {
                    id: "asr-nemotron-3.5-0.6b-560ms".to_string(),
                    role: ModelRole::Asr,
                    repo: "ndgold/nemotron-3.5-asr-streaming-0.6b-onnx".to_string(),
                    revision: "f73e3bc81614fa69e24b091b892e8672983fbb9c".to_string(),
                    files: asr_files,
                    license: "NVIDIA Open Model License (OpenMDW v1.1)".to_string(),
                    upstream: Some(
                        "https://github.com/k2-fsa/sherpa-onnx/releases/tag/asr-models \
                         asset sha256:c6bf5e0df765f9d5b43bc9e0536d4b4b3e7d40bdf5ecf13e45f134c51c05ae3a"
                            .to_string(),
                    ),
                },
                ModelArtifact {
                    id: "llm-qwen3-0.6b-easylanguage".to_string(),
                    role: ModelRole::Llm,
                    repo: "ndgold/Qwen3-0.6B-EasyLanguage-GGUF".to_string(),
                    revision: "658db5ad6a05e3f549990b10e8d8c59a78866695".to_string(),
                    files: vec![ModelFile::pinned(
                        "Qwen3-0.6B-EasyLanguage-Q4_K_M.gguf",
                        396_704_608,
                        "1be8c19cc1153d33535c3ad6a3306be7d84ecc4ae7a171bf6c1b428c6737ffb8",
                    )],
                    license: "Apache-2.0".to_string(),
                    upstream: Some(
                        "https://huggingface.co/ndgold/Qwen3-0.6B-EasyLanguage-GGUF".to_string(),
                    ),
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
    download_file_cancellable(
        client,
        artifact,
        file,
        destination,
        allow_unverified,
        None,
        on_progress,
    )
}

/// [`download_file`] with a cancel flag checked between chunks. A cancelled
/// download keeps its partial file so the next run resumes with `Range`.
pub fn download_file_cancellable(
    client: &reqwest::blocking::Client,
    artifact: &ModelArtifact,
    file: &ModelFile,
    destination: &Path,
    allow_unverified: bool,
    cancel: Option<&AtomicBool>,
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
    let mut attempt = 0u32;
    loop {
        attempt += 1;
        match download_once(
            client,
            destination,
            file.size,
            artifact,
            file,
            cancel,
            on_progress,
        ) {
            Ok(()) => break,
            Err(ModelError::Cancelled) => return Err(ModelError::Cancelled),
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
    destination: &Path,
    expected_size: u64,
    artifact: &ModelArtifact,
    file: &ModelFile,
    cancel: Option<&AtomicBool>,
    on_progress: &mut dyn FnMut(DownloadProgress),
) -> Result<(), ModelError> {
    let url = &resolve_url(&artifact.repo, &artifact.revision, &file.path);
    let is_cancelled = || cancel.is_some_and(|flag| flag.load(Ordering::SeqCst));
    if is_cancelled() {
        return Err(ModelError::Cancelled);
    }
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
        if is_cancelled() {
            sink.flush()?;
            return Err(ModelError::Cancelled);
        }
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

/// Present/verified state of one artifact, for the first-run screen.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelStatus {
    pub id: String,
    pub title: String,
    pub bytes: u64,
    /// Every file exists in the cache.
    pub present: bool,
    /// Every file has its pinned size. Hashes are checked during download,
    /// not on every status call.
    pub verified: bool,
}

impl ModelArtifact {
    pub fn title(&self) -> String {
        match self.role {
            ModelRole::Asr => "Speech recognition".to_string(),
            ModelRole::Llm => "Simplifier".to_string(),
            ModelRole::Vad => "Voice activity".to_string(),
        }
    }

    pub fn status(&self, root: &Path) -> ModelStatus {
        let dir = model_dir(root, &self.repo, &self.revision);
        let sizes: Vec<Option<u64>> = self
            .files
            .iter()
            .map(|file| {
                std::fs::metadata(dir.join(&file.path))
                    .ok()
                    .map(|meta| meta.len())
            })
            .collect();
        let present = !sizes.is_empty() && sizes.iter().all(Option::is_some);
        let verified = present
            && self
                .files
                .iter()
                .zip(&sizes)
                .all(|(file, size)| *size == Some(file.size));
        ModelStatus {
            id: self.id.clone(),
            title: self.title(),
            bytes: self.total_bytes(),
            present,
            verified,
        }
    }
}

impl ModelCatalog {
    pub fn status(&self, root: &Path) -> Vec<ModelStatus> {
        self.artifacts
            .iter()
            .map(|artifact| artifact.status(root))
            .collect()
    }
}

/// What the shared download loop reports. Content-free: ids, file names,
/// byte counts, and error codes only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DownloadEvent {
    Progress {
        id: String,
        file: String,
        done_bytes: u64,
        total_bytes: u64,
    },
    Ready {
        id: String,
    },
    Error {
        id: String,
        code: String,
        reason: String,
    },
}

/// Download every artifact of `catalog` into `root` (disk preflight, resume,
/// SHA-256 verify, cancel between chunks). Used by `fetch_models` and the
/// bridge. Progress is throttled to `max_per_sec` events per file (the last
/// chunk of a file is always reported). Stops at the first error.
pub fn download_catalog(
    client: &reqwest::blocking::Client,
    catalog: &ModelCatalog,
    root: &Path,
    cancel: Option<&AtomicBool>,
    max_per_sec: u32,
    on_event: &mut dyn FnMut(DownloadEvent),
) -> Result<(), ModelError> {
    let missing: u64 = catalog
        .artifacts
        .iter()
        .filter(|artifact| !artifact.status(root).verified)
        .map(ModelArtifact::total_bytes)
        .sum();
    let first = catalog
        .artifacts
        .first()
        .map(|artifact| artifact.id.clone());
    if let Err(error) = disk_preflight(&root.join(".preflight"), missing) {
        on_event(DownloadEvent::Error {
            id: first.unwrap_or_default(),
            code: error.code().to_string(),
            reason: error.to_string(),
        });
        return Err(error);
    }
    let interval = std::time::Duration::from_secs_f64(1.0 / f64::from(max_per_sec.max(1)));
    for artifact in &catalog.artifacts {
        let dir = model_dir(root, &artifact.repo, &artifact.revision);
        for file in &artifact.files {
            let mut last: Option<std::time::Instant> = None;
            let mut forward = |progress: DownloadProgress| {
                let now = std::time::Instant::now();
                let due = last.is_none_or(|at| now.duration_since(at) >= interval);
                if due || progress.received >= progress.total {
                    last = Some(now);
                    on_event(DownloadEvent::Progress {
                        id: artifact.id.clone(),
                        file: file.path.clone(),
                        done_bytes: progress.received,
                        total_bytes: progress.total,
                    });
                }
            };
            let result = download_file_cancellable(
                client,
                artifact,
                file,
                &dir.join(&file.path),
                false,
                cancel,
                &mut forward,
            );
            if let Err(error) = result {
                on_event(DownloadEvent::Error {
                    id: artifact.id.clone(),
                    code: error.code().to_string(),
                    reason: error.to_string(),
                });
                return Err(error);
            }
        }
        on_event(DownloadEvent::Ready {
            id: artifact.id.clone(),
        });
    }
    Ok(())
}

/// HTTP client for model downloads (tokenless; the only network use).
pub fn http_client() -> Result<reqwest::blocking::Client, ModelError> {
    reqwest::blocking::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|error| ModelError::Download {
            url: "https://huggingface.co".to_string(),
            message: error.to_string(),
        })
}

/// One download at a time. `try_begin` hands out a ticket; the slot frees
/// when the ticket drops. `cancel` asks the running download to stop.
#[derive(Debug, Clone, Default)]
pub struct DownloadSlot {
    running: Arc<AtomicBool>,
    cancel: Arc<AtomicBool>,
}

#[derive(Debug)]
pub struct DownloadTicket {
    running: Arc<AtomicBool>,
    cancel: Arc<AtomicBool>,
}

impl DownloadTicket {
    pub fn cancel_flag(&self) -> &AtomicBool {
        &self.cancel
    }
}

impl Drop for DownloadTicket {
    fn drop(&mut self) {
        self.running.store(false, Ordering::SeqCst);
    }
}

impl DownloadSlot {
    pub fn try_begin(&self) -> Option<DownloadTicket> {
        if self
            .running
            .compare_exchange(false, true, Ordering::SeqCst, Ordering::SeqCst)
            .is_err()
        {
            return None;
        }
        self.cancel.store(false, Ordering::SeqCst);
        Some(DownloadTicket {
            running: self.running.clone(),
            cancel: self.cancel.clone(),
        })
    }

    pub fn is_running(&self) -> bool {
        self.running.load(Ordering::SeqCst)
    }

    /// Returns true when a running download was asked to stop.
    pub fn cancel(&self) -> bool {
        if self.is_running() {
            self.cancel.store(true, Ordering::SeqCst);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[test]
    fn status_on_empty_root_is_missing() {
        let root = tempfile::tempdir().unwrap();
        let status = ModelCatalog::plan_default().status(root.path());
        assert_eq!(status.len(), 2);
        assert!(status.iter().all(|model| !model.present && !model.verified));
        assert!(status.iter().all(|model| model.bytes > 0));
    }

    #[test]
    fn status_checks_presence_then_size() {
        let root = tempfile::tempdir().unwrap();
        let catalog = ModelCatalog::plan_default();
        let llm = catalog.get("llm-qwen3-0.6b-easylanguage").unwrap();
        let dir = model_dir(root.path(), &llm.repo, &llm.revision);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join(&llm.files[0].path);
        std::fs::write(&path, b"partial").unwrap();
        let status = llm.status(root.path());
        assert!(status.present && !status.verified);
        File::create(&path)
            .unwrap()
            .set_len(llm.files[0].size)
            .unwrap();
        let status = llm.status(root.path());
        assert!(status.present && status.verified);
    }

    #[test]
    fn download_slot_allows_one_at_a_time_and_cancels() {
        let slot = DownloadSlot::default();
        assert!(!slot.cancel(), "nothing running to cancel");
        let ticket = slot.try_begin().expect("first download starts");
        assert!(slot.try_begin().is_none(), "second download is rejected");
        assert!(slot.cancel());
        assert!(ticket.cancel_flag().load(Ordering::SeqCst));
        drop(ticket);
        let again = slot.try_begin().expect("slot frees when the ticket drops");
        assert!(!again.cancel_flag().load(Ordering::SeqCst), "cancel resets");
    }

    #[test]
    fn error_codes_are_stable() {
        assert_eq!(ModelError::Cancelled.code(), "CANCELLED");
        assert_eq!(
            ModelError::DiskSpace {
                needed: 1,
                available: 0
            }
            .code(),
            "DISK_FULL"
        );
        assert_eq!(
            ModelError::Download {
                url: String::new(),
                message: String::new()
            }
            .code(),
            "NETWORK"
        );
    }

    #[test]
    fn cancelled_flag_stops_before_any_network_call() {
        // The URL is unroutable on purpose; a pre-set cancel flag must return
        // before the request is sent.
        let client = reqwest::blocking::Client::builder()
            .timeout(std::time::Duration::from_millis(1))
            .build()
            .unwrap();
        let root = tempfile::tempdir().unwrap();
        let catalog = ModelCatalog::plan_default();
        let artifact = &catalog.artifacts[1];
        let cancel = AtomicBool::new(true);
        let result = download_file_cancellable(
            &client,
            artifact,
            &artifact.files[0],
            &root.path().join("model.gguf"),
            false,
            Some(&cancel),
            &mut |_| {},
        );
        assert!(matches!(result, Err(ModelError::Cancelled)));
    }

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
    fn plan_default_is_two_pinned_artifacts() {
        let catalog = ModelCatalog::plan_default();
        assert_eq!(catalog.artifacts.len(), 2);
        assert!(catalog.unpinned().is_empty());

        let asr = catalog.get("asr-nemotron-3.5-0.6b-560ms").unwrap();
        assert_eq!(asr.role, ModelRole::Asr);
        assert_eq!(asr.files.len(), 4);
        assert!(asr.is_pinned());
        assert_eq!(asr.total_bytes(), 682_215_356);
        assert!(asr.license.contains("NVIDIA"));
        assert!(asr.upstream.as_deref().unwrap().contains("c6bf5e0d"));
        assert_eq!(asr.revision.len(), 40);

        let llm = catalog.get("llm-qwen3-0.6b-easylanguage").unwrap();
        assert_eq!(llm.role, ModelRole::Llm);
        assert_eq!(llm.files.len(), 1);
        assert_eq!(llm.total_bytes(), 396_704_608);
        assert_eq!(llm.license, "Apache-2.0");
        assert_eq!(llm.revision.len(), 40);
        assert_eq!(
            llm.files[0].sha256.as_deref(),
            Some("1be8c19cc1153d33535c3ad6a3306be7d84ecc4ae7a171bf6c1b428c6737ffb8")
        );
    }

    #[test]
    fn downloader_refuses_an_unpinned_file() {
        let artifact = ModelArtifact {
            id: "test".to_string(),
            role: ModelRole::Llm,
            repo: "org/repo".to_string(),
            revision: "main".to_string(),
            files: vec![ModelFile::unpinned("model.gguf", 10)],
            license: "test".to_string(),
            upstream: None,
        };
        let client = reqwest::blocking::Client::new();
        let dir = tempfile::tempdir().unwrap();
        let result = download_file(
            &client,
            &artifact,
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
            catalog.get("asr-nemotron-3.5-0.6b-560ms").unwrap().role,
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
