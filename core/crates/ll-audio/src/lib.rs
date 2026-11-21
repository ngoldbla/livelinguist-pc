//! Audio sources, resampling, VAD, chunking, and the 10-second ring buffer.
//!
//! See `docs/plans/2026-11-21-live-linguist-pc-master-plan.md` and
//! `AGENTS.md` for the behavioral contract.

/// Crate version from Cargo metadata.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
