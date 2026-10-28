//! Event-sourced durable world state and in-process event bus.

use rusqlite::{Connection, OptionalExtension, Transaction, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    path::Path,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter, State};
use uuid::Uuid;

pub const WORLD_EVENT: &str = "saiki://world-event";
const MAX_TEXT_BYTES: usize = 16 * 1024;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectLifecycle {
    Planned,
    Active,
    Paused,
    Completed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct WorldProject {
    id: String,
    title: String,
    summary: String,
    lifecycle: ProjectLifecycle,
    version: i64,
    created_at_unix_ms: i64,
    updated_at_unix_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldEvent {
    id: String,
    entity_id: String,
    sequence: i64,
    entity_version: i64,
    event_type: String,
    payload: serde_json::Value,
    actor: &'static str,
    occurred_at_unix_ms: i64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Reconstruction {
    projects: Vec<WorldProject>,
    event_count: i64,
    consistent: bool,
    inconsistencies: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateProjectRequest {
    title: String,
    summary: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct TransitionProjectRequest {
    id: String,
    expected_version: i64,
    lifecycle: ProjectLifecycle,
    #[serde(default)]
    summary: Option<String>,
}

pub struct WorldState {
    conn: Mutex<Connection>,
}

impl WorldState {
    pub fn load(config_dir: &Path) -> Result<Self, String> {
        std::fs::create_dir_all(config_dir).map_err(|_| "create world data directory failed")?;
        Self::from_connection(
            Connection::open(config_dir.join("world.sqlite3"))
                .map_err(|_| "open world database failed")?,
        )
    }
    fn from_connection(conn: Connection) -> Result<Self, String> {
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;
        CREATE TABLE IF NOT EXISTS world_entities(id TEXT PRIMARY KEY,entity_type TEXT NOT NULL,current_version INTEGER NOT NULL,state_json TEXT NOT NULL,created_at INTEGER NOT NULL,updated_at INTEGER NOT NULL);
        CREATE TABLE IF NOT EXISTS world_events(id TEXT PRIMARY KEY,global_sequence INTEGER NOT NULL UNIQUE,entity_id TEXT NOT NULL,entity_version INTEGER NOT NULL,event_type TEXT NOT NULL,payload_json TEXT NOT NULL,actor TEXT NOT NULL,occurred_at INTEGER NOT NULL,UNIQUE(entity_id,entity_version));
        CREATE TABLE IF NOT EXISTS world_snapshots(entity_id TEXT NOT NULL,entity_version INTEGER NOT NULL,state_json TEXT NOT NULL,created_at INTEGER NOT NULL,PRIMARY KEY(entity_id,entity_version));").map_err(|_|"migrate world database failed")?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
    fn create_project(
        &self,
        r: CreateProjectRequest,
    ) -> Result<(WorldProject, WorldEvent), String> {
        validate_text(&r.title, "title")?;
        validate_text(&r.summary, "summary")?;
        let now = now_ms();
        let project = WorldProject {
            id: Uuid::new_v4().to_string(),
            title: r.title,
            summary: r.summary,
            lifecycle: ProjectLifecycle::Planned,
            version: 1,
            created_at_unix_ms: now,
            updated_at_unix_ms: now,
        };
        let payload =
            serde_json::to_value(&project).map_err(|_| "serialize project event failed")?;
        let mut guard = self.conn.lock().map_err(|_| "world state unavailable")?;
        let tx = guard
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| "begin world transaction failed")?;
        let event = append_event(&tx, &project.id, 1, "project_created", &payload, now)?;
        persist_projection(&tx, &project)?;
        tx.commit().map_err(|_| "commit project creation failed")?;
        Ok((project, event))
    }
    fn transition(
        &self,
        r: TransitionProjectRequest,
    ) -> Result<(WorldProject, WorldEvent), String> {
        if let Some(summary) = &r.summary {
            validate_text(summary, "summary")?
        }
        let mut guard = self.conn.lock().map_err(|_| "world state unavailable")?;
        let tx = guard
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| "begin world transaction failed")?;
        let raw: Option<String> = tx
            .query_row(
                "SELECT state_json FROM world_entities WHERE id=?1 AND entity_type='project'",
                [&r.id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| "read project failed")?;
        let mut project: WorldProject = serde_json::from_str(&raw.ok_or("project not found")?)
            .map_err(|_| "stored project is invalid")?;
        if project.version != r.expected_version {
            return Err(format!(
                "version conflict: expected {}, current {}",
                r.expected_version, project.version
            ));
        }
        if !transition_allowed(&project.lifecycle, &r.lifecycle) {
            return Err("invalid project lifecycle transition".into());
        }
        project.lifecycle = r.lifecycle;
        if let Some(summary) = r.summary {
            project.summary = summary
        }
        project.version += 1;
        project.updated_at_unix_ms = now_ms();
        let payload = serde_json::json!({"lifecycle":project.lifecycle,"summary":project.summary});
        let event = append_event(
            &tx,
            &project.id,
            project.version,
            "project_transitioned",
            &payload,
            project.updated_at_unix_ms,
        )?;
        persist_projection(&tx, &project)?;
        tx.commit()
            .map_err(|_| "commit project transition failed")?;
        Ok((project, event))
    }
    fn reconstruct(&self) -> Result<Reconstruction, String> {
        let guard = self.conn.lock().map_err(|_| "world state unavailable")?;
        let mut q=guard.prepare("SELECT id,entity_id,global_sequence,entity_version,event_type,payload_json,occurred_at FROM world_events ORDER BY global_sequence").map_err(|_|"read world events failed")?;
        let rows = q
            .query_map([], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, i64>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, String>(4)?,
                    r.get::<_, String>(5)?,
                    r.get::<_, i64>(6)?,
                ))
            })
            .map_err(|_| "read world events failed")?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "read world events failed")?;
        drop(q);
        let mut projects: BTreeMap<String, WorldProject> = BTreeMap::new();
        let mut inconsistencies = vec![];
        for (_, entity, _, version, event_type, payload, occurred) in &rows {
            match event_type.as_str() {
                "project_created" => match serde_json::from_str::<WorldProject>(payload) {
                    Ok(p) if p.id == *entity && p.version == *version => {
                        projects.insert(entity.clone(), p);
                    }
                    _ => inconsistencies.push(format!("invalid create event for {entity}")),
                },
                "project_transitioned" => {
                    if let Some(p) = projects.get_mut(entity) {
                        if *version != p.version + 1 {
                            inconsistencies.push(format!("non-contiguous version for {entity}"));
                            continue;
                        }
                        let value: serde_json::Value =
                            serde_json::from_str(payload).unwrap_or_default();
                        match serde_json::from_value(
                            value.get("lifecycle").cloned().unwrap_or_default(),
                        ) {
                            Ok(lifecycle) => p.lifecycle = lifecycle,
                            Err(_) => {
                                inconsistencies.push(format!("invalid lifecycle for {entity}"));
                                continue;
                            }
                        }
                        if let Some(summary) = value.get("summary").and_then(|v| v.as_str()) {
                            p.summary = summary.into()
                        }
                        p.version = *version;
                        p.updated_at_unix_ms = *occurred
                    } else {
                        inconsistencies.push(format!("transition before creation for {entity}"))
                    }
                }
                other => inconsistencies.push(format!("unknown event {other}")),
            }
        }
        for (id, reconstructed) in &projects {
            let stored: Option<String> = guard
                .query_row(
                    "SELECT state_json FROM world_entities WHERE id=?1",
                    [id],
                    |r| r.get(0),
                )
                .optional()
                .map_err(|_| "compare world projection failed")?;
            if stored.as_deref()
                != Some(
                    &serde_json::to_string(reconstructed)
                        .map_err(|_| "serialize reconstruction failed")?,
                )
            {
                inconsistencies.push(format!("projection mismatch for {id}"))
            }
        }
        Ok(Reconstruction {
            projects: projects.into_values().collect(),
            event_count: rows.len() as i64,
            consistent: inconsistencies.is_empty(),
            inconsistencies,
        })
    }
    fn events(&self, after: i64) -> Result<Vec<WorldEvent>, String> {
        let guard = self.conn.lock().map_err(|_| "world state unavailable")?;
        let mut q=guard.prepare("SELECT id,entity_id,global_sequence,entity_version,event_type,payload_json,occurred_at FROM world_events WHERE global_sequence>?1 ORDER BY global_sequence LIMIT 500").map_err(|_|"read world event bus failed")?;
        q.query_map([after], |r| {
            Ok(WorldEvent {
                id: r.get(0)?,
                entity_id: r.get(1)?,
                sequence: r.get(2)?,
                entity_version: r.get(3)?,
                event_type: r.get(4)?,
                payload: serde_json::from_str::<serde_json::Value>(&r.get::<_, String>(5)?)
                    .unwrap_or_default(),
                actor: "owner",
                occurred_at_unix_ms: r.get(6)?,
            })
        })
        .map_err(|_| "read world event bus failed")?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "read world event bus failed".to_string())
    }
}

fn persist_projection(tx: &Transaction<'_>, p: &WorldProject) -> Result<(), String> {
    let state = serde_json::to_string(p).map_err(|_| "serialize world projection failed")?;
    tx.execute("INSERT INTO world_entities VALUES(?1,'project',?2,?3,?4,?5) ON CONFLICT(id) DO UPDATE SET current_version=excluded.current_version,state_json=excluded.state_json,updated_at=excluded.updated_at",params![p.id,p.version,state,p.created_at_unix_ms,p.updated_at_unix_ms]).map_err(|_|"persist world projection failed")?;
    tx.execute(
        "INSERT INTO world_snapshots VALUES(?1,?2,?3,?4)",
        params![p.id, p.version, state, p.updated_at_unix_ms],
    )
    .map_err(|_| "persist world snapshot failed")?;
    Ok(())
}
fn append_event(
    tx: &Transaction<'_>,
    entity: &str,
    version: i64,
    event_type: &str,
    payload: &serde_json::Value,
    now: i64,
) -> Result<WorldEvent, String> {
    let sequence: i64 = tx
        .query_row(
            "SELECT COALESCE(MAX(global_sequence),0)+1 FROM world_events",
            [],
            |r| r.get(0),
        )
        .map_err(|_| "allocate world event sequence failed")?;
    let event = WorldEvent {
        id: Uuid::new_v4().to_string(),
        entity_id: entity.into(),
        sequence,
        entity_version: version,
        event_type: event_type.into(),
        payload: payload.clone(),
        actor: "owner",
        occurred_at_unix_ms: now,
    };
    tx.execute(
        "INSERT INTO world_events VALUES(?1,?2,?3,?4,?5,?6,'owner',?7)",
        params![
            event.id,
            event.sequence,
            event.entity_id,
            event.entity_version,
            event.event_type,
            event.payload.to_string(),
            event.occurred_at_unix_ms
        ],
    )
    .map_err(|_| "persist world event failed")?;
    Ok(event)
}
fn transition_allowed(from: &ProjectLifecycle, to: &ProjectLifecycle) -> bool {
    matches!(
        (from, to),
        (ProjectLifecycle::Planned, ProjectLifecycle::Active)
            | (ProjectLifecycle::Planned, ProjectLifecycle::Cancelled)
            | (ProjectLifecycle::Active, ProjectLifecycle::Paused)
            | (ProjectLifecycle::Active, ProjectLifecycle::Completed)
            | (ProjectLifecycle::Active, ProjectLifecycle::Cancelled)
            | (ProjectLifecycle::Paused, ProjectLifecycle::Active)
            | (ProjectLifecycle::Paused, ProjectLifecycle::Cancelled)
    )
}
fn validate_text(v: &str, label: &str) -> Result<(), String> {
    if v.trim().is_empty() || v.len() > MAX_TEXT_BYTES {
        Err(format!(
            "{label} must be non-empty and at most {MAX_TEXT_BYTES} bytes"
        ))
    } else {
        Ok(())
    }
}
fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(i64::MAX)
}

#[tauri::command]
pub fn get_world_snapshot(state: State<'_, WorldState>) -> Result<Reconstruction, String> {
    state.reconstruct()
}
#[tauri::command]
pub fn list_world_events(
    state: State<'_, WorldState>,
    after_sequence: i64,
) -> Result<Vec<WorldEvent>, String> {
    state.events(after_sequence)
}
#[tauri::command]
pub fn create_world_project(
    app: AppHandle,
    state: State<'_, WorldState>,
    request: CreateProjectRequest,
) -> Result<WorldProject, String> {
    let (project, event) = state.create_project(request)?;
    let _ = app.emit(WORLD_EVENT, event);
    Ok(project)
}
#[tauri::command]
pub fn transition_world_project(
    app: AppHandle,
    state: State<'_, WorldState>,
    request: TransitionProjectRequest,
) -> Result<WorldProject, String> {
    let (project, event) = state.transition(request)?;
    let _ = app.emit(WORLD_EVENT, event);
    Ok(project)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn durable_world_reconstructs_after_restart() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("world.sqlite3");
        let id;
        {
            let s = WorldState::from_connection(Connection::open(&path).unwrap()).unwrap();
            let (p, _) = s
                .create_project(CreateProjectRequest {
                    title: "Saiki".into(),
                    summary: "Build".into(),
                })
                .unwrap();
            id = p.id.clone();
            s.transition(TransitionProjectRequest {
                id: id.clone(),
                expected_version: 1,
                lifecycle: ProjectLifecycle::Active,
                summary: None,
            })
            .unwrap();
        }
        let reopened = WorldState::from_connection(Connection::open(&path).unwrap()).unwrap();
        let world = reopened.reconstruct().unwrap();
        assert!(world.consistent);
        assert_eq!(world.event_count, 2);
        assert_eq!(world.projects[0].id, id);
        assert_eq!(world.projects[0].version, 2);
        assert_eq!(world.projects[0].lifecycle, ProjectLifecycle::Active);
    }
    #[test]
    fn stale_version_cannot_transition() {
        let s = WorldState::from_connection(Connection::open_in_memory().unwrap()).unwrap();
        let (p, _) = s
            .create_project(CreateProjectRequest {
                title: "P".into(),
                summary: "S".into(),
            })
            .unwrap();
        let result = s.transition(TransitionProjectRequest {
            id: p.id,
            expected_version: 0,
            lifecycle: ProjectLifecycle::Active,
            summary: None,
        });
        assert!(result.unwrap_err().contains("version conflict"));
    }
    #[test]
    fn invalid_lifecycle_transition_is_rejected() {
        let s = WorldState::from_connection(Connection::open_in_memory().unwrap()).unwrap();
        let (p, _) = s
            .create_project(CreateProjectRequest {
                title: "P".into(),
                summary: "S".into(),
            })
            .unwrap();
        assert!(
            s.transition(TransitionProjectRequest {
                id: p.id,
                expected_version: 1,
                lifecycle: ProjectLifecycle::Completed,
                summary: None
            })
            .is_err()
        );
    }
    #[test]
    fn event_bus_sequences_are_stable() {
        let s = WorldState::from_connection(Connection::open_in_memory().unwrap()).unwrap();
        s.create_project(CreateProjectRequest {
            title: "A".into(),
            summary: "B".into(),
        })
        .unwrap();
        s.create_project(CreateProjectRequest {
            title: "C".into(),
            summary: "D".into(),
        })
        .unwrap();
        let events = s.events(1).unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].sequence, 2);
    }
}
