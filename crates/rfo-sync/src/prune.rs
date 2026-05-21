//! Prune removed/missing/archived repos.
//!
//! Interactive confirmation by default, --force to skip.
//! Records audit event.

use anyhow::{Context, Result};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

/// Result of a prune operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PruneResult {
    pub repo_id: String,
    pub owner: String,
    pub name: String,
    pub removed: bool,
    pub reason: String,
}

/// Remove a single repo from tracking by ID.
pub fn prune_repo(conn: &Connection, repo_id: &str) -> Result<PruneResult> {
    let (owner, name) = conn
        .query_row(
            "SELECT owner, name FROM repos WHERE id = ?1",
            params![repo_id],
            |row| Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?)),
        )
        .with_context(|| format!("repo {repo_id} not found"))?;

    let n = conn.execute("DELETE FROM repos WHERE id = ?1", params![repo_id])?;
    Ok(PruneResult {
        repo_id: repo_id.to_string(),
        owner,
        name,
        removed: n > 0,
        reason: "manual prune".into(),
    })
}

/// Remove all archived repos from tracking.
pub fn prune_archived(conn: &Connection) -> Result<Vec<PruneResult>> {
    let mut results = Vec::new();
    let mut stmt = conn.prepare("SELECT id, owner, name FROM repos WHERE archived = 1")?;
    let rows: Vec<(String, String, String)> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();

    for (id, owner, name) in rows {
        conn.execute("DELETE FROM repos WHERE id = ?1", params![id])?;
        results.push(PruneResult {
            repo_id: id,
            owner,
            name,
            removed: true,
            reason: "archived".into(),
        });
    }
    Ok(results)
}

/// Remove repos whose local_path no longer exists on disk.
pub fn prune_missing(conn: &Connection) -> Result<Vec<PruneResult>> {
    let mut results = Vec::new();
    let mut stmt = conn.prepare("SELECT id, owner, name, local_path FROM repos")?;
    let rows: Vec<(String, String, String, String)> = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, String>(0)?,
                row.get::<_, String>(1)?,
                row.get::<_, String>(2)?,
                row.get::<_, String>(3)?,
            ))
        })?
        .filter_map(|r| r.ok())
        .collect();

    for (id, owner, name, local_path) in rows {
        if !std::path::Path::new(&local_path).exists() {
            conn.execute("DELETE FROM repos WHERE id = ?1", params![id])?;
            results.push(PruneResult {
                repo_id: id,
                owner,
                name,
                removed: true,
                reason: format!("local path missing: {local_path}"),
            });
        }
    }
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tempfile::TempDir;

    fn setup() -> (TempDir, Connection) {
        let tmp = TempDir::new().unwrap();
        let conn = rfo_state::open_memory().unwrap();
        (tmp, conn)
    }

    fn projects_dir(tmp: &TempDir) -> PathBuf {
        tmp.path().join("projects")
    }

    #[test]
    fn prune_repo_removes_tracked_repo() {
        let (tmp, conn) = setup();
        let repo = crate::manage::add(&conn, "alice/proj1", &projects_dir(&tmp)).unwrap();
        let result = prune_repo(&conn, &repo.id).unwrap();
        assert!(result.removed);
        assert_eq!(result.owner, "alice");
        assert_eq!(result.name, "proj1");
        assert!(crate::manage::list(&conn, None).unwrap().is_empty());
    }

    #[test]
    fn prune_repo_not_found() {
        let (_, conn) = setup();
        let err = prune_repo(&conn, "nonexistent").unwrap_err();
        assert!(err.to_string().contains("not found"));
    }

    #[test]
    fn prune_archived_removes_archived() {
        let (tmp, conn) = setup();
        let repo = crate::manage::add(&conn, "alice/archived-repo", &projects_dir(&tmp)).unwrap();
        // Mark as archived
        conn.execute(
            "UPDATE repos SET archived = 1 WHERE id = ?1",
            params![repo.id],
        )
        .unwrap();

        let results = prune_archived(&conn).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "archived-repo");
        assert!(results[0].removed);
        assert!(crate::manage::list(&conn, None).unwrap().is_empty());
    }

    #[test]
    fn prune_archived_skips_non_archived() {
        let (tmp, conn) = setup();
        crate::manage::add(&conn, "alice/active", &projects_dir(&tmp)).unwrap();
        let results = prune_archived(&conn).unwrap();
        assert!(results.is_empty());
        assert_eq!(crate::manage::list(&conn, None).unwrap().len(), 1);
    }

    #[test]
    fn prune_missing_removes_nonexistent_paths() {
        let (tmp, conn) = setup();
        let repo = crate::manage::add(&conn, "alice/gone", &projects_dir(&tmp)).unwrap();
        // Don't create the directory
        let results = prune_missing(&conn).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].name, "gone");
        assert!(results[0].removed);
    }

    #[test]
    fn prune_missing_keeps_existing_paths() {
        let (tmp, conn) = setup();
        let repo = crate::manage::add(&conn, "alice/present", &projects_dir(&tmp)).unwrap();
        // Create the local path
        let local = projects_dir(&tmp).join("alice").join("present");
        std::fs::create_dir_all(&local).unwrap();

        let results = prune_missing(&conn).unwrap();
        assert!(results.is_empty());
    }
}
