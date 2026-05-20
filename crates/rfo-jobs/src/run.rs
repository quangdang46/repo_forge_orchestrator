//! Run record management.
//!
//! Every command that mutates state creates a run record.
//! Fields: id, command, started_at, ended_at, exit_code, args_json, user, host.
