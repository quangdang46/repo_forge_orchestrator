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
