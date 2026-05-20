//! GitHub auth discovery.
//!
//! Order: GITHUB_TOKEN env → config token → gh CLI fallback → auto (try all).
//! Never log tokens. Handle rate limits with backoff.
