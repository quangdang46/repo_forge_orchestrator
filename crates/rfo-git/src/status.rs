//! Repo status detection.
//!
//! States: Current, Behind, Ahead, Diverged, Conflict, Dirty, Missing, Unknown.

/// Repository status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepoStatus {
    Current,
    Behind,
    Ahead,
    Diverged,
    Conflict,
    Dirty,
    Missing,
    Unknown,
}

impl std::fmt::Display for RepoStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RepoStatus::Current => write!(f, "Current"),
            RepoStatus::Behind => write!(f, "Behind"),
            RepoStatus::Ahead => write!(f, "Ahead"),
            RepoStatus::Diverged => write!(f, "Diverged"),
            RepoStatus::Conflict => write!(f, "Conflict"),
            RepoStatus::Dirty => write!(f, "Dirty"),
            RepoStatus::Missing => write!(f, "Missing"),
            RepoStatus::Unknown => write!(f, "Unknown"),
        }
    }
}
