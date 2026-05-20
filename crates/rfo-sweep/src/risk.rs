//! Risk classification.
//!
//! LOW, MEDIUM, HIGH based on:
//! - touches many files
//! - modifies CI workflow
//! - modifies dependency file
//! - deletes files
//! - no tests changed
//! - quality gates unavailable
//! - similar failure happened recently

/// Risk level for a plan or operation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RiskLevel {
    Low,
    Medium,
    High,
}

impl std::fmt::Display for RiskLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskLevel::Low => write!(f, "LOW"),
            RiskLevel::Medium => write!(f, "MEDIUM"),
            RiskLevel::High => write!(f, "HIGH"),
        }
    }
}
