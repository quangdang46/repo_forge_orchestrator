//! Repo-level file locks using fs4.
//!
//! Before any git mutation, acquire a lock on the repo path.
//! Lock is released after mutation completes or fails.
//! Lock timeout with clear error message.
