//! Claude Code provider.
//!
//! Invokes `claude -p --output-format stream-json` with configurable args.
//! Parses stream-json output. Handles errors, timeouts.
