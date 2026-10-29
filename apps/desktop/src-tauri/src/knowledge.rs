//! Local document knowledge: ingestion, chunking, embedding, and cited
//! retrieval, owned by the Rust core and backed by SQLite.
//!
//! Embeddings here are **deterministic lexical vectors** (FNV-hashed term
//! frequencies, L2-normalized), scored by cosine similarity. They are fully
//! local and require no external model, so retrieval always works offline. A
//! learned neural embedding can later replace `embed()` behind the same
//! contract without changing the schema or the citation shape.
//!
//! Every retrieval result cites the actual local document it came from (title,
//! path, and chunk index). Each document carries an owner-declared privacy
//! label; `sensitive` documents are excluded from retrieval unless the owner
//! explicitly opts in. Secrets must never be ingested.

use crate::memory::Sensitivity;
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::State;

const SCHEMA_VERSION: i64 = 1;
const EMBED_DIM: usize = 256;
const MAX_CHUNK_CHARS: usize = 1200;
const MAX_DOCUMENT_BYTES: u64 = 2 * 1024 * 1024;
const MAX_TITLE_BYTES: usize = 200;
const RETRIEVE_LIMIT: usize = 8;
const SNIPPET_CHARS: usize = 240;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DocumentSummary {
    id: String,
    title: String,
    path: String,
    sensitivity: Sensitivity,
    byte_len: i64,
    chunk_count: i64,
    ingested_at_unix_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Citation {
    document_id: String,
    title: String,
    path: String,
    sensitivity: Sensitivity,
    chunk_index: i64,
    score: f64,
    snippet: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct IngestRequest {
    path: String,
    sensitivity: Sensitivity,
    #[serde(default)]
    title: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RetrieveRequest {
    query: String,
    #[serde(default)]
    include_sensitive: bool,
}

pub struct KnowledgeState {
    conn: Mutex<Connection>,
}

impl KnowledgeState {
    pub fn load(data_dir: &Path) -> Result<Self, String> {
        std::fs::create_dir_all(data_dir).map_err(|_| "create knowledge directory failed")?;
        let connection = Connection::open(data_dir.join("knowledge.sqlite3"))
            .map_err(|_| "open knowledge database failed")?;
        Self::from_connection(connection)
    }

    fn from_connection(connection: Connection) -> Result<Self, String> {
        connection
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(|_| "configure knowledge database failed")?;
        connection
            .pragma_update(None, "foreign_keys", true)
            .map_err(|_| "configure knowledge database failed")?;
        migrate(&connection)?;
        Ok(Self {
            conn: Mutex::new(connection),
        })
    }

    fn ingest(&self, request: &IngestRequest) -> Result<DocumentSummary, String> {
        let path = request.path.trim();
        if path.is_empty() {
            return Err("a document path is required".into());
        }
        let metadata = std::fs::metadata(path).map_err(|_| "document path is not readable")?;
        if !metadata.is_file() {
            return Err("document path is not a file".into());
        }
        if metadata.len() > MAX_DOCUMENT_BYTES {
            return Err("document exceeds the 2 MB ingestion limit".into());
        }
        let content = std::fs::read(path).map_err(|_| "reading the document failed")?;
        let text = String::from_utf8(content).map_err(|_| "document is not UTF-8 text")?;
        let chunks = chunk_text(&text);
        if chunks.is_empty() {
            return Err("document has no ingestable text".into());
        }
        let title = resolve_title(request.title.as_deref(), path)?;
        let byte_len = metadata.len() as i64;
        let now = now_unix_ms();
        let id = uuid::Uuid::new_v4().to_string();

        let mut guard = self
            .conn
            .lock()
            .map_err(|_| "knowledge state unavailable")?;
        let tx = guard
            .transaction()
            .map_err(|_| "begin knowledge write failed")?;
        tx.execute(
            "INSERT INTO documents (id, title, path, sensitivity, byte_len, chunk_count, ingested_at_unix_ms) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![id, title, path, request.sensitivity.as_str(), byte_len, chunks.len() as i64, now],
        )
        .map_err(|_| "insert document failed")?;
        for (index, chunk) in chunks.iter().enumerate() {
            let embedding = embed(chunk);
            tx.execute(
                "INSERT INTO chunks (id, document_id, chunk_index, content, embedding) VALUES (?1, ?2, ?3, ?4, ?5)",
                params![uuid::Uuid::new_v4().to_string(), id, index as i64, chunk, embedding_to_bytes(&embedding)],
            )
            .map_err(|_| "insert chunk failed")?;
        }
        tx.commit().map_err(|_| "commit knowledge write failed")?;
        read_document(&guard, &id)
    }

    fn list(&self) -> Result<Vec<DocumentSummary>, String> {
        let guard = self
            .conn
            .lock()
            .map_err(|_| "knowledge state unavailable")?;
        let mut statement = guard
            .prepare(
                "SELECT id, title, path, sensitivity, byte_len, chunk_count, ingested_at_unix_ms \
                 FROM documents ORDER BY ingested_at_unix_ms DESC",
            )
            .map_err(|_| "prepare document list failed")?;
        let rows = statement
            .query_map([], map_document)
            .map_err(|_| "document list failed")?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|_| "read document list failed".into())
    }

    fn forget(&self, id: &str) -> Result<bool, String> {
        let guard = self
            .conn
            .lock()
            .map_err(|_| "knowledge state unavailable")?;
        // chunks cascade via the foreign key.
        let affected = guard
            .execute("DELETE FROM documents WHERE id = ?1", params![id])
            .map_err(|_| "delete document failed")?;
        Ok(affected > 0)
    }

    fn retrieve(&self, request: &RetrieveRequest, limit: usize) -> Result<Vec<Citation>, String> {
        let query_vec = embed(&request.query);
        if query_vec.iter().all(|value| *value == 0.0) {
            return Ok(Vec::new());
        }
        let guard = self
            .conn
            .lock()
            .map_err(|_| "knowledge state unavailable")?;
        let sql = "SELECT d.id, d.title, d.path, d.sensitivity, c.chunk_index, c.content, c.embedding \
             FROM chunks c JOIN documents d ON d.id = c.document_id \
             WHERE (?1 = 1 OR d.sensitivity != 'sensitive')";
        let mut statement = guard.prepare(sql).map_err(|_| "prepare retrieval failed")?;
        let scored = statement
            .query_map(params![request.include_sensitive as i64], |row| {
                let sensitivity_raw: String = row.get(3)?;
                let content: String = row.get(5)?;
                let embedding_bytes: Vec<u8> = row.get(6)?;
                let sensitivity = Sensitivity::parse(&sensitivity_raw).map_err(|_| {
                    rusqlite::Error::InvalidColumnType(
                        3,
                        "sensitivity".into(),
                        rusqlite::types::Type::Text,
                    )
                })?;
                let score = cosine(&query_vec, &embedding_from_bytes(&embedding_bytes));
                Ok(Citation {
                    document_id: row.get(0)?,
                    title: row.get(1)?,
                    path: row.get(2)?,
                    sensitivity,
                    chunk_index: row.get(4)?,
                    score,
                    snippet: snippet(&content),
                })
            })
            .map_err(|_| "retrieval failed")?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "read retrieval failed")?;

        let mut hits: Vec<Citation> = scored.into_iter().filter(|c| c.score > 0.0).collect();
        hits.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        hits.truncate(limit);
        Ok(hits)
    }
}

fn migrate(conn: &Connection) -> Result<(), String> {
    let version: i64 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|_| "read knowledge schema version failed")?;
    if version < 1 {
        conn.execute_batch(
            "CREATE TABLE documents (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                path TEXT NOT NULL,
                sensitivity TEXT NOT NULL,
                byte_len INTEGER NOT NULL,
                chunk_count INTEGER NOT NULL,
                ingested_at_unix_ms INTEGER NOT NULL
            );
            CREATE TABLE chunks (
                id TEXT PRIMARY KEY,
                document_id TEXT NOT NULL REFERENCES documents(id) ON DELETE CASCADE,
                chunk_index INTEGER NOT NULL,
                content TEXT NOT NULL,
                embedding BLOB NOT NULL
            );
            CREATE INDEX idx_chunks_document ON chunks(document_id);",
        )
        .map_err(|_| "apply knowledge migration v1 failed")?;
        conn.pragma_update(None, "user_version", SCHEMA_VERSION)
            .map_err(|_| "record knowledge schema version failed")?;
    }
    Ok(())
}

fn read_document(conn: &Connection, id: &str) -> Result<DocumentSummary, String> {
    conn.query_row(
        "SELECT id, title, path, sensitivity, byte_len, chunk_count, ingested_at_unix_ms FROM documents WHERE id = ?1",
        params![id],
        map_document,
    )
    .map_err(|_| "read document failed".into())
}

fn map_document(row: &rusqlite::Row<'_>) -> rusqlite::Result<DocumentSummary> {
    let sensitivity_raw: String = row.get(3)?;
    Ok(DocumentSummary {
        id: row.get(0)?,
        title: row.get(1)?,
        path: row.get(2)?,
        sensitivity: Sensitivity::parse(&sensitivity_raw).map_err(|_| {
            rusqlite::Error::InvalidColumnType(3, "sensitivity".into(), rusqlite::types::Type::Text)
        })?,
        byte_len: row.get(4)?,
        chunk_count: row.get(5)?,
        ingested_at_unix_ms: row.get(6)?,
    })
}

fn resolve_title(provided: Option<&str>, path: &str) -> Result<String, String> {
    let title = match provided {
        Some(value) if !value.trim().is_empty() => value.trim().to_owned(),
        _ => Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("document")
            .to_owned(),
    };
    if title.len() > MAX_TITLE_BYTES {
        return Err("title must be at most 200 bytes".into());
    }
    Ok(title)
}

/// Split text into chunks on blank-line boundaries, bounded by character count.
/// Oversized paragraphs are hard-split so no chunk exceeds the limit.
fn chunk_text(text: &str) -> Vec<String> {
    let mut chunks = Vec::new();
    let mut current = String::new();
    for paragraph in text.split("\n\n") {
        let paragraph = paragraph.trim();
        if paragraph.is_empty() {
            continue;
        }
        if paragraph.chars().count() > MAX_CHUNK_CHARS {
            if !current.is_empty() {
                chunks.push(std::mem::take(&mut current));
            }
            let mut window = String::new();
            for ch in paragraph.chars() {
                window.push(ch);
                if window.chars().count() >= MAX_CHUNK_CHARS {
                    chunks.push(std::mem::take(&mut window));
                }
            }
            if !window.is_empty() {
                chunks.push(window);
            }
            continue;
        }
        if !current.is_empty()
            && current.chars().count() + paragraph.chars().count() + 2 > MAX_CHUNK_CHARS
        {
            chunks.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push_str("\n\n");
        }
        current.push_str(paragraph);
    }
    if !current.is_empty() {
        chunks.push(current);
    }
    chunks
}

fn snippet(content: &str) -> String {
    let trimmed = content.trim();
    if trimmed.chars().count() <= SNIPPET_CHARS {
        return trimmed.to_owned();
    }
    let mut out: String = trimmed.chars().take(SNIPPET_CHARS).collect();
    out.push('…');
    out
}

fn tokenize(text: &str) -> Vec<String> {
    text.split(|c: char| !c.is_alphanumeric())
        .filter(|token| token.len() >= 2)
        .map(|token| token.to_lowercase())
        .collect()
}

/// Stable FNV-1a hash so embeddings are identical across runs (unlike the std
/// hasher, which is not stability-guaranteed).
fn fnv1a(token: &str) -> u64 {
    let mut hash: u64 = 0xcbf2_9ce4_8422_2325;
    for byte in token.bytes() {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    hash
}

fn embed(text: &str) -> Vec<f32> {
    let mut vector = vec![0f32; EMBED_DIM];
    for token in tokenize(text) {
        let bucket = (fnv1a(&token) as usize) % EMBED_DIM;
        vector[bucket] += 1.0;
    }
    let norm: f32 = vector.iter().map(|value| value * value).sum::<f32>().sqrt();
    if norm > 0.0 {
        for value in &mut vector {
            *value /= norm;
        }
    }
    vector
}

fn cosine(a: &[f32], b: &[f32]) -> f64 {
    if a.len() != b.len() {
        return 0.0;
    }
    // Both operands are L2-normalized, so the dot product is the cosine.
    a.iter()
        .zip(b)
        .map(|(x, y)| (*x as f64) * (*y as f64))
        .sum()
}

fn embedding_to_bytes(vector: &[f32]) -> Vec<u8> {
    vector
        .iter()
        .flat_map(|value| value.to_le_bytes())
        .collect()
}

fn embedding_from_bytes(bytes: &[u8]) -> Vec<f32> {
    bytes
        .chunks_exact(4)
        .map(|chunk| f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]]))
        .collect()
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
pub fn ingest_document(
    state: State<'_, KnowledgeState>,
    request: IngestRequest,
) -> Result<DocumentSummary, String> {
    state.ingest(&request)
}

#[tauri::command]
pub fn list_documents(state: State<'_, KnowledgeState>) -> Result<Vec<DocumentSummary>, String> {
    state.list()
}

#[tauri::command]
pub fn forget_document(state: State<'_, KnowledgeState>, id: String) -> Result<bool, String> {
    state.forget(&id)
}

#[tauri::command]
pub fn retrieve_knowledge(
    state: State<'_, KnowledgeState>,
    request: RetrieveRequest,
) -> Result<Vec<Citation>, String> {
    state.retrieve(&request, RETRIEVE_LIMIT)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn knowledge() -> KnowledgeState {
        KnowledgeState::from_connection(Connection::open_in_memory().unwrap()).unwrap()
    }

    fn write_file(dir: &Path, name: &str, body: &str) -> String {
        let path = dir.join(name);
        let mut file = std::fs::File::create(&path).unwrap();
        file.write_all(body.as_bytes()).unwrap();
        path.to_str().unwrap().to_owned()
    }

    fn ingest(state: &KnowledgeState, path: &str, sensitivity: Sensitivity) -> DocumentSummary {
        state
            .ingest(&IngestRequest {
                path: path.into(),
                sensitivity,
                title: None,
            })
            .unwrap()
    }

    #[test]
    fn retrieval_cites_the_real_local_document_across_restart() {
        let dir = tempfile::tempdir().unwrap();
        let data = tempfile::tempdir().unwrap();
        let doc_path = write_file(
            dir.path(),
            "notes.txt",
            "The deployment runbook lives in the operations handbook.\n\nRollback uses the blue-green switch.",
        );
        {
            let state = KnowledgeState::load(data.path()).unwrap();
            let summary = ingest(&state, &doc_path, Sensitivity::Private);
            assert!(summary.chunk_count >= 1);
        }
        // Reopen the store, as a restart would, and retrieve.
        let reopened = KnowledgeState::load(data.path()).unwrap();
        let hits = reopened
            .retrieve(
                &RetrieveRequest {
                    query: "rollback blue-green".into(),
                    include_sensitive: false,
                },
                RETRIEVE_LIMIT,
            )
            .unwrap();
        assert!(!hits.is_empty());
        // The top citation points at the actual local file that was ingested.
        assert_eq!(hits[0].path, doc_path);
        assert!(hits[0].snippet.to_lowercase().contains("rollback"));
        assert!(hits[0].score > 0.0);
    }

    #[test]
    fn long_documents_are_chunked() {
        let paragraph = "lorem ipsum dolor sit amet ".repeat(80);
        let text = format!("{paragraph}\n\n{paragraph}\n\n{paragraph}");
        let chunks = chunk_text(&text);
        assert!(chunks.len() >= 3);
        assert!(
            chunks
                .iter()
                .all(|chunk| chunk.chars().count() <= MAX_CHUNK_CHARS)
        );
    }

    #[test]
    fn sensitive_documents_are_excluded_unless_opted_in() {
        let dir = tempfile::tempdir().unwrap();
        let state = knowledge();
        let path = write_file(dir.path(), "secret.txt", "project aurora launches in march");
        ingest(&state, &path, Sensitivity::Sensitive);
        let default_hits = state
            .retrieve(
                &RetrieveRequest {
                    query: "aurora launches".into(),
                    include_sensitive: false,
                },
                RETRIEVE_LIMIT,
            )
            .unwrap();
        assert!(default_hits.is_empty());
        let opted_in = state
            .retrieve(
                &RetrieveRequest {
                    query: "aurora launches".into(),
                    include_sensitive: true,
                },
                RETRIEVE_LIMIT,
            )
            .unwrap();
        assert_eq!(opted_in.len(), 1);
        assert_eq!(opted_in[0].sensitivity, Sensitivity::Sensitive);
    }

    #[test]
    fn forgetting_a_document_removes_its_chunks() {
        let dir = tempfile::tempdir().unwrap();
        let state = knowledge();
        let path = write_file(dir.path(), "doc.txt", "retrievable content about pipelines");
        let summary = ingest(&state, &path, Sensitivity::Public);
        assert!(state.forget(&summary.id).unwrap());
        let hits = state
            .retrieve(
                &RetrieveRequest {
                    query: "pipelines".into(),
                    include_sensitive: true,
                },
                RETRIEVE_LIMIT,
            )
            .unwrap();
        assert!(hits.is_empty());
        assert!(state.list().unwrap().is_empty());
    }

    #[test]
    fn embeddings_are_deterministic_and_normalized() {
        let a = embed("the quick brown fox");
        let b = embed("the quick brown fox");
        assert_eq!(a, b);
        let norm: f32 = a.iter().map(|value| value * value).sum::<f32>().sqrt();
        assert!((norm - 1.0).abs() < 1e-5);
        // Related text scores higher than unrelated text.
        let related = cosine(&embed("quick brown fox"), &a);
        let unrelated = cosine(&embed("completely different subject matter"), &a);
        assert!(related > unrelated);
    }

    #[test]
    fn empty_query_returns_no_citations() {
        let state = knowledge();
        let hits = state
            .retrieve(
                &RetrieveRequest {
                    query: "   ".into(),
                    include_sensitive: true,
                },
                RETRIEVE_LIMIT,
            )
            .unwrap();
        assert!(hits.is_empty());
    }

    #[test]
    fn non_utf8_document_is_rejected() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("binary.bin");
        std::fs::write(&path, [0xff, 0xfe, 0x00, 0x01]).unwrap();
        let state = knowledge();
        let result = state.ingest(&IngestRequest {
            path: path.to_str().unwrap().into(),
            sensitivity: Sensitivity::Public,
            title: None,
        });
        assert!(result.is_err());
    }
}
