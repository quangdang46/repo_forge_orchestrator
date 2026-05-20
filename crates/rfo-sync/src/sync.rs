//! Sync engine implementation.
//!
//! Read repos from SQLite → create run record → for each repo:
//! acquire fs4 lock → clone if missing → fetch → pull per strategy
//! → record pre/post OID → release lock.
