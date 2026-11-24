//! Download the pinned 1.0 model set (T11).
//!
//! Usage: `cargo run --release -p ll-models --example fetch_models [root]`
//! Root defaults to `cache_root()`. Prints `ok <artifact> <file> <bytes>` or
//! `fail <artifact> <file> <error>` per file; exits 1 if any file failed.
//! Uses the same loop as the app (`download_catalog`): disk preflight, resume,
//! SHA-256 verification. Files already present and verified are not fetched.

use ll_models::{
    cache_root, download_catalog, http_client, model_dir, DownloadEvent, ModelCatalog,
};
use std::path::PathBuf;

fn main() {
    let root = std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(cache_root);
    let catalog = ModelCatalog::plan_default();
    let client = match http_client() {
        Ok(client) => client,
        Err(error) => {
            eprintln!("fail http-client - {error}");
            std::process::exit(1);
        }
    };
    let mut failed = false;
    let mut on_event = |event: DownloadEvent| {
        if let DownloadEvent::Error { id, code, reason } = event {
            failed = true;
            println!("fail {id} - {code}: {reason}");
        }
    };
    let result = download_catalog(&client, &catalog, &root, None, 1, &mut on_event);
    for artifact in &catalog.artifacts {
        let dir = model_dir(&root, &artifact.repo, &artifact.revision);
        for file in &artifact.files {
            let path = dir.join(&file.path);
            match std::fs::metadata(&path) {
                Ok(meta) if meta.len() == file.size => {
                    println!("ok {} {} {}", artifact.id, file.path, meta.len())
                }
                Ok(meta) => {
                    failed = true;
                    println!(
                        "fail {} {} size {} != {}",
                        artifact.id,
                        file.path,
                        meta.len(),
                        file.size
                    )
                }
                Err(error) => {
                    failed = true;
                    println!("fail {} {} {error}", artifact.id, file.path)
                }
            }
        }
        println!("dir {} {}", artifact.id, dir.display());
    }
    if result.is_err() || failed {
        std::process::exit(1);
    }
}
