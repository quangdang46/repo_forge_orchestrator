//! Default denylist paths.

/// Default denylist patterns.
pub const DEFAULT_DENYLIST: &[&str] = &[
    ".env",
    ".env.*",
    "*.pem",
    "*.key",
    "id_rsa",
    "id_ed25519",
    "**/.git/**",
    "**/target/**",
    "**/node_modules/**",
];
