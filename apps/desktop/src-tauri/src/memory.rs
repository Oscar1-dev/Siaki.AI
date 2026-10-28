//! SQLite-backed semantic memory foundation.
//!
//! Memory is a set of owner-teachable claims, each carrying provenance
//! (source, confidence, sensitivity) and timestamps. Retrieval is FTS5 keyword
//! search; embeddings are a later phase. Three owner operations are supported
//! and persist across restart:
//!
//! - **teach**: record a new active claim.
//! - **correct**: supersede an active claim with a new one. The superseded row
//!   is retained for audit (`status = superseded`) and the new claim links back
//!   through `supersedes`; a correction never silently overwrites history.
//! - **forget**: hard-delete a claim's content, honoring the owner's deletion
//!   right. An append-only event row records that a claim was forgotten without
//!   retaining what it said.
//!
//! Schema changes are applied through versioned migrations keyed on
//! `PRAGMA user_version`.

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::State;

const SCHEMA_VERSION: i64 = 1;
const MAX_CLAIM_BYTES: usize = 4096;
const MAX_SOURCE_BYTES: usize = 200;
const SEARCH_LIMIT: usize = 50;
const LIST_LIMIT: usize = 200;

const CLAIM_COLUMNS: &str = "id, claim, source, confidence, sensitivity, status, supersedes, created_at_unix_ms, updated_at_unix_ms";
// Fully qualified for the FTS join, where `memory_fts` also exposes a `claim` column.
const CLAIM_COLUMNS_QUALIFIED: &str = "memory_claims.id, memory_claims.claim, memory_claims.source, memory_claims.confidence, memory_claims.sensitivity, memory_claims.status, memory_claims.supersedes, memory_claims.created_at_unix_ms, memory_claims.updated_at_unix_ms";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Sensitivity {
    Public,
    Private,
    Sensitive,
}

impl Sensitivity {
    pub fn as_str(self) -> &'static str {
        match self {
            Sensitivity::Public => "public",
            Sensitivity::Private => "private",
            Sensitivity::Sensitive => "sensitive",
        }
    }

    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "public" => Ok(Sensitivity::Public),
            "private" => Ok(Sensitivity::Private),
            "sensitive" => Ok(Sensitivity::Sensitive),
            _ => Err("stored sensitivity is invalid".into()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ClaimStatus {
    Active,
    Superseded,
    Forgotten,
}

impl ClaimStatus {
    fn parse(value: &str) -> Result<Self, String> {
        match value {
            "active" => Ok(ClaimStatus::Active),
            "superseded" => Ok(ClaimStatus::Superseded),
            "forgotten" => Ok(ClaimStatus::Forgotten),
            _ => Err("stored claim status is invalid".into()),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryClaim {
    id: String,
    claim: String,
    source: String,
    confidence: f64,
    sensitivity: Sensitivity,
    status: ClaimStatus,
    supersedes: Option<String>,
    created_at_unix_ms: i64,
    updated_at_unix_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryEvent {
    kind: String,
    claim_id: String,
    detail: String,
    at_unix_ms: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MemoryExport {
    schema_version: i64,
    claims: Vec<MemoryClaim>,
    events: Vec<MemoryEvent>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TeachRequest {
    claim: String,
    source: String,
    confidence: f64,
    sensitivity: Sensitivity,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CorrectRequest {
    id: String,
    claim: String,
    source: String,
    confidence: f64,
    sensitivity: Sensitivity,
}

pub struct MemoryState {
    conn: Mutex<Connection>,
}

impl MemoryState {
    pub fn load(data_dir: &Path) -> Result<Self, String> {
        std::fs::create_dir_all(data_dir).map_err(|_| "create memory directory failed")?;
        let connection = Connection::open(data_dir.join("memory.sqlite3"))
            .map_err(|_| "open memory database failed")?;
        Self::from_connection(connection)
    }

    /// Open on an existing connection (used for both the on-disk store and
    /// in-memory tests) and bring the schema up to date.
    fn from_connection(connection: Connection) -> Result<Self, String> {
        connection
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(|_| "configure memory database failed")?;
        connection
            .pragma_update(None, "foreign_keys", true)
            .map_err(|_| "configure memory database failed")?;
        migrate(&connection)?;
        Ok(Self {
            conn: Mutex::new(connection),
        })
    }

    fn teach(&self, request: &TeachRequest) -> Result<MemoryClaim, String> {
        validate_claim(&request.claim, &request.source, request.confidence)?;
        let mut guard = self.conn.lock().map_err(|_| "memory state unavailable")?;
        let now = now_unix_ms();
        let id = uuid::Uuid::new_v4().to_string();
        let tx = guard
            .transaction()
            .map_err(|_| "begin memory write failed")?;
        insert_claim(
            &tx,
            &id,
            request.claim.trim(),
            request.source.trim(),
            request.confidence,
            request.sensitivity,
            None,
            now,
        )?;
        log_event(&tx, "teach", &id, "claim recorded", now)?;
        tx.commit().map_err(|_| "commit memory write failed")?;
        read_claim(&guard, &id)
    }

    fn correct(&self, request: &CorrectRequest) -> Result<MemoryClaim, String> {
        validate_claim(&request.claim, &request.source, request.confidence)?;
        let mut guard = self.conn.lock().map_err(|_| "memory state unavailable")?;
        let now = now_unix_ms();
        let tx = guard
            .transaction()
            .map_err(|_| "begin memory write failed")?;
        let old_rowid: Option<i64> = tx
            .query_row(
                "SELECT rowid FROM memory_claims WHERE id = ?1 AND status = 'active'",
                params![request.id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| "read prior claim failed")?;
        let Some(old_rowid) = old_rowid else {
            return Err("no active claim with that id to correct".into());
        };
        tx.execute(
            "UPDATE memory_claims SET status = 'superseded', updated_at_unix_ms = ?2 WHERE rowid = ?1",
            params![old_rowid, now],
        )
        .map_err(|_| "supersede prior claim failed")?;
        tx.execute(
            "DELETE FROM memory_fts WHERE rowid = ?1",
            params![old_rowid],
        )
        .map_err(|_| "remove prior claim from search failed")?;
        let id = uuid::Uuid::new_v4().to_string();
        insert_claim(
            &tx,
            &id,
            request.claim.trim(),
            request.source.trim(),
            request.confidence,
            request.sensitivity,
            Some(&request.id),
            now,
        )?;
        log_event(
            &tx,
            "correct",
            &id,
            &format!("supersedes {}", request.id),
            now,
        )?;
        tx.commit().map_err(|_| "commit memory write failed")?;
        read_claim(&guard, &id)
    }

    fn forget(&self, id: &str) -> Result<bool, String> {
        let mut guard = self.conn.lock().map_err(|_| "memory state unavailable")?;
        let now = now_unix_ms();
        let tx = guard
            .transaction()
            .map_err(|_| "begin memory write failed")?;
        let rowid: Option<i64> = tx
            .query_row(
                "SELECT rowid FROM memory_claims WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| "read claim failed")?;
        let Some(rowid) = rowid else {
            tx.commit().map_err(|_| "commit memory write failed")?;
            return Ok(false);
        };
        tx.execute("DELETE FROM memory_fts WHERE rowid = ?1", params![rowid])
            .map_err(|_| "remove claim from search failed")?;
        tx.execute("DELETE FROM memory_claims WHERE rowid = ?1", params![rowid])
            .map_err(|_| "delete claim failed")?;
        // Record that a claim was forgotten without retaining its content.
        log_event(&tx, "forget", id, "claim content deleted", now)?;
        tx.commit().map_err(|_| "commit memory write failed")?;
        Ok(true)
    }

    fn search(&self, query: &str, limit: usize) -> Result<Vec<MemoryClaim>, String> {
        let match_query = fts_query(query);
        if match_query.is_empty() {
            return Ok(Vec::new());
        }
        let guard = self.conn.lock().map_err(|_| "memory state unavailable")?;
        let sql = format!(
            "SELECT {CLAIM_COLUMNS_QUALIFIED} FROM memory_fts \
             JOIN memory_claims ON memory_claims.rowid = memory_fts.rowid \
             WHERE memory_fts MATCH ?1 AND memory_claims.status = 'active' \
             ORDER BY rank LIMIT ?2"
        );
        let mut statement = guard.prepare(&sql).map_err(|_| "prepare search failed")?;
        let rows = statement
            .query_map(params![match_query, limit as i64], map_claim)
            .map_err(|_| "memory search failed")?;
        collect_claims(rows)
    }

    fn list(&self, limit: usize) -> Result<Vec<MemoryClaim>, String> {
        let guard = self.conn.lock().map_err(|_| "memory state unavailable")?;
        let sql = format!(
            "SELECT {CLAIM_COLUMNS} FROM memory_claims WHERE status = 'active' \
             ORDER BY updated_at_unix_ms DESC LIMIT ?1"
        );
        let mut statement = guard.prepare(&sql).map_err(|_| "prepare list failed")?;
        let rows = statement
            .query_map(params![limit as i64], map_claim)
            .map_err(|_| "memory list failed")?;
        collect_claims(rows)
    }

    fn export(&self) -> Result<MemoryExport, String> {
        let guard = self.conn.lock().map_err(|_| "memory state unavailable")?;
        let claim_sql =
            format!("SELECT {CLAIM_COLUMNS} FROM memory_claims ORDER BY created_at_unix_ms ASC");
        let mut claim_stmt = guard
            .prepare(&claim_sql)
            .map_err(|_| "prepare export failed")?;
        let claims = collect_claims(
            claim_stmt
                .query_map([], map_claim)
                .map_err(|_| "export claims failed")?,
        )?;
        let mut event_stmt = guard
            .prepare("SELECT kind, claim_id, detail, at_unix_ms FROM memory_events ORDER BY at_unix_ms ASC, id ASC")
            .map_err(|_| "prepare export events failed")?;
        let events = event_stmt
            .query_map([], |row| {
                Ok(MemoryEvent {
                    kind: row.get(0)?,
                    claim_id: row.get(1)?,
                    detail: row.get(2)?,
                    at_unix_ms: row.get(3)?,
                })
            })
            .map_err(|_| "export events failed")?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "read export events failed")?;
        Ok(MemoryExport {
            schema_version: SCHEMA_VERSION,
            claims,
            events,
        })
    }
}

fn migrate(conn: &Connection) -> Result<(), String> {
    let version: i64 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|_| "read schema version failed")?;
    if version < 1 {
        conn.execute_batch(
            "CREATE TABLE memory_claims (
                rowid INTEGER PRIMARY KEY,
                id TEXT NOT NULL UNIQUE,
                claim TEXT NOT NULL,
                source TEXT NOT NULL,
                confidence REAL NOT NULL,
                sensitivity TEXT NOT NULL,
                status TEXT NOT NULL,
                supersedes TEXT,
                created_at_unix_ms INTEGER NOT NULL,
                updated_at_unix_ms INTEGER NOT NULL
            );
            CREATE INDEX idx_memory_claims_status ON memory_claims(status);
            CREATE VIRTUAL TABLE memory_fts USING fts5(claim);
            CREATE TABLE memory_events (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                kind TEXT NOT NULL,
                claim_id TEXT NOT NULL,
                detail TEXT NOT NULL,
                at_unix_ms INTEGER NOT NULL
            );",
        )
        .map_err(|_| "apply schema migration v1 failed (FTS5 may be unavailable)")?;
        conn.pragma_update(None, "user_version", SCHEMA_VERSION)
            .map_err(|_| "record schema version failed")?;
    }
    Ok(())
}

#[allow(clippy::too_many_arguments)]
fn insert_claim(
    conn: &Connection,
    id: &str,
    claim: &str,
    source: &str,
    confidence: f64,
    sensitivity: Sensitivity,
    supersedes: Option<&str>,
    now: i64,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO memory_claims \
         (id, claim, source, confidence, sensitivity, status, supersedes, created_at_unix_ms, updated_at_unix_ms) \
         VALUES (?1, ?2, ?3, ?4, ?5, 'active', ?6, ?7, ?7)",
        params![id, claim, source, confidence, sensitivity.as_str(), supersedes, now],
    )
    .map_err(|_| "insert claim failed")?;
    let rowid = conn.last_insert_rowid();
    conn.execute(
        "INSERT INTO memory_fts (rowid, claim) VALUES (?1, ?2)",
        params![rowid, claim],
    )
    .map_err(|_| "index claim for search failed")?;
    Ok(())
}

fn log_event(
    conn: &Connection,
    kind: &str,
    claim_id: &str,
    detail: &str,
    now: i64,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO memory_events (kind, claim_id, detail, at_unix_ms) VALUES (?1, ?2, ?3, ?4)",
        params![kind, claim_id, detail, now],
    )
    .map_err(|_| "record memory event failed")?;
    Ok(())
}

fn read_claim(conn: &Connection, id: &str) -> Result<MemoryClaim, String> {
    let sql = format!("SELECT {CLAIM_COLUMNS} FROM memory_claims WHERE id = ?1");
    conn.query_row(&sql, params![id], map_claim)
        .map_err(|_| "read claim failed".into())
}

fn map_claim(row: &rusqlite::Row<'_>) -> rusqlite::Result<MemoryClaim> {
    let sensitivity: String = row.get(4)?;
    let status: String = row.get(5)?;
    Ok(MemoryClaim {
        id: row.get(0)?,
        claim: row.get(1)?,
        source: row.get(2)?,
        confidence: row.get(3)?,
        sensitivity: Sensitivity::parse(&sensitivity).map_err(|_| {
            rusqlite::Error::InvalidColumnType(4, "sensitivity".into(), rusqlite::types::Type::Text)
        })?,
        status: ClaimStatus::parse(&status).map_err(|_| {
            rusqlite::Error::InvalidColumnType(5, "status".into(), rusqlite::types::Type::Text)
        })?,
        supersedes: row.get(6)?,
        created_at_unix_ms: row.get(7)?,
        updated_at_unix_ms: row.get(8)?,
    })
}

fn collect_claims(
    rows: rusqlite::MappedRows<'_, impl FnMut(&rusqlite::Row<'_>) -> rusqlite::Result<MemoryClaim>>,
) -> Result<Vec<MemoryClaim>, String> {
    rows.collect::<Result<Vec<_>, _>>()
        .map_err(|_| "read claims failed".into())
}

/// Build a safe FTS5 MATCH expression from arbitrary owner input by quoting
/// each whitespace-separated term. This prevents raw FTS5 operators in user
/// text from producing a query syntax error.
fn fts_query(raw: &str) -> String {
    raw.split_whitespace()
        .map(|term| term.replace('"', ""))
        .filter(|term| !term.is_empty())
        .map(|term| format!("\"{term}\""))
        .collect::<Vec<_>>()
        .join(" ")
}

fn validate_claim(claim: &str, source: &str, confidence: f64) -> Result<(), String> {
    if claim.trim().is_empty() || claim.len() > MAX_CLAIM_BYTES {
        return Err("claim must be 1-4096 bytes".into());
    }
    if source.trim().is_empty() || source.len() > MAX_SOURCE_BYTES {
        return Err("source must be 1-200 bytes".into());
    }
    if !(0.0..=1.0).contains(&confidence) {
        return Err("confidence must be between 0.0 and 1.0".into());
    }
    Ok(())
}

fn now_unix_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(i64::MAX)
}

#[tauri::command]
pub fn teach_memory(
    state: State<'_, MemoryState>,
    request: TeachRequest,
) -> Result<MemoryClaim, String> {
    state.teach(&request)
}

#[tauri::command]
pub fn correct_memory(
    state: State<'_, MemoryState>,
    request: CorrectRequest,
) -> Result<MemoryClaim, String> {
    state.correct(&request)
}

#[tauri::command]
pub fn forget_memory(state: State<'_, MemoryState>, id: String) -> Result<bool, String> {
    state.forget(&id)
}

#[tauri::command]
pub fn search_memory(
    state: State<'_, MemoryState>,
    query: String,
) -> Result<Vec<MemoryClaim>, String> {
    state.search(&query, SEARCH_LIMIT)
}

#[tauri::command]
pub fn list_memory(state: State<'_, MemoryState>) -> Result<Vec<MemoryClaim>, String> {
    state.list(LIST_LIMIT)
}

#[tauri::command]
pub fn export_memory(state: State<'_, MemoryState>) -> Result<MemoryExport, String> {
    state.export()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn memory() -> MemoryState {
        MemoryState::from_connection(Connection::open_in_memory().unwrap()).unwrap()
    }

    fn teach(state: &MemoryState, claim: &str, source: &str) -> MemoryClaim {
        state
            .teach(&TeachRequest {
                claim: claim.into(),
                source: source.into(),
                confidence: 0.8,
                sensitivity: Sensitivity::Private,
            })
            .unwrap()
    }

    #[test]
    fn migration_is_idempotent() {
        let state = memory();
        // Re-running migration on the same connection must not error.
        let guard = state.conn.lock().unwrap();
        migrate(&guard).unwrap();
    }

    #[test]
    fn teach_is_searchable_and_survives_reopen() {
        let dir = tempfile::tempdir().unwrap();
        {
            let state = MemoryState::load(dir.path()).unwrap();
            teach(&state, "the owner prefers metric units", "owner");
        }
        // A fresh state over the same file reconstructs the memory, as a restart would.
        let reopened = MemoryState::load(dir.path()).unwrap();
        let hits = reopened.search("metric", SEARCH_LIMIT).unwrap();
        assert_eq!(hits.len(), 1);
        assert!(hits[0].claim.contains("metric units"));
        assert_eq!(hits[0].status, ClaimStatus::Active);
    }

    #[test]
    fn correct_supersedes_and_links_without_losing_history() {
        let state = memory();
        let original = teach(&state, "the project ships on friday", "owner");
        let corrected = state
            .correct(&CorrectRequest {
                id: original.id.clone(),
                claim: "the project ships on monday".into(),
                source: "owner".into(),
                confidence: 0.9,
                sensitivity: Sensitivity::Private,
            })
            .unwrap();
        assert_eq!(corrected.supersedes.as_deref(), Some(original.id.as_str()));
        // Only the corrected claim is retrievable.
        let friday = state.search("friday", SEARCH_LIMIT).unwrap();
        assert!(friday.is_empty());
        let monday = state.search("monday", SEARCH_LIMIT).unwrap();
        assert_eq!(monday.len(), 1);
        // The superseded original remains in the audit history via export.
        let export = state.export().unwrap();
        assert!(
            export
                .claims
                .iter()
                .any(|claim| claim.id == original.id && claim.status == ClaimStatus::Superseded)
        );
        assert!(export.events.iter().any(|event| event.kind == "correct"));
    }

    #[test]
    fn forget_removes_content_but_logs_the_deletion() {
        let dir = tempfile::tempdir().unwrap();
        let id = {
            let state = MemoryState::load(dir.path()).unwrap();
            let claim = teach(&state, "a secret passphrase hint", "owner");
            assert!(state.forget(&claim.id).unwrap());
            claim.id
        };
        let reopened = MemoryState::load(dir.path()).unwrap();
        // Gone from search and from the claim table, and stays gone after reopen.
        assert!(
            reopened
                .search("passphrase", SEARCH_LIMIT)
                .unwrap()
                .is_empty()
        );
        let export = reopened.export().unwrap();
        assert!(!export.claims.iter().any(|claim| claim.id == id));
        // The deletion itself is auditable, but the content is not retained.
        let forget_events: Vec<_> = export
            .events
            .iter()
            .filter(|event| event.kind == "forget" && event.claim_id == id)
            .collect();
        assert_eq!(forget_events.len(), 1);
        assert!(!forget_events[0].detail.contains("passphrase"));
    }

    #[test]
    fn forget_unknown_id_is_false() {
        let state = memory();
        assert!(!state.forget("does-not-exist").unwrap());
    }

    #[test]
    fn search_tolerates_fts_operator_characters() {
        let state = memory();
        teach(&state, "deployment notes for the api", "owner");
        // Raw FTS5 operators in user input must not raise a syntax error; the
        // query is sanitized into quoted terms rather than rejected.
        state.search("api AND (notes OR \"", SEARCH_LIMIT).unwrap();
        // A normal multi-word query still retrieves the claim.
        let hits = state.search("notes api", SEARCH_LIMIT).unwrap();
        assert_eq!(hits.len(), 1);
    }

    #[test]
    fn claim_validation_rejects_bad_input() {
        assert!(validate_claim("", "owner", 0.5).is_err());
        assert!(validate_claim("ok", "", 0.5).is_err());
        assert!(validate_claim("ok", "owner", 1.5).is_err());
        assert!(validate_claim("ok", "owner", 0.5).is_ok());
    }
}
