//! Model catalog, Hugging Face resolve, resumable download, SHA-256 verification.
//!
//! See `docs/plans/2026-11-21-live-linguist-pc-master-plan.md` and
//! `AGENTS.md` for the behavioral contract.

/// Crate version from Cargo metadata.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
