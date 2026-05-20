//! Sweep operations for rfo.
//!
//! sweep commit: scan denylist → secret scan → quality gates → commit/push
//! sweep agent: AI-driven sweep with plan/apply cycle

pub mod agent;
pub mod commit;
pub mod denylist;
pub mod quality_gates;
pub mod risk;
pub mod secret_scan;
pub mod train;

pub use denylist::{DEFAULT_DENYLIST, Denylist};
pub use risk::{RiskLevel, RiskReason, classify, to_json};
pub use secret_scan::{
    SecretFinding, SecretScanMode, scan_file, scan_files, scan_text, should_block,
};
