//! Deterministic, quota-bounded autonomous supervisor.
//!
//! Phase 18 executes only internal world-integrity audits. It has no external
//! tool capability and cannot turn model output into authority.

use crate::world::WorldState;
use rusqlite::{Connection, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::{Arc, Condvar, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::State;
use uuid::Uuid;

const MAX_TASKS_PER_GOAL: i64 = 100;
const TASKS_PER_WAKE: i64 = 3;
const TASKS_PER_HOUR: i64 = 20;
const LEASE_MS: i64 = 30_000;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupervisorGoal {
    id: String,
    objective: String,
    priority: i64,
    max_tasks: i64,
    completed_tasks: i64,
    state: String,
    next_wake_at_unix_ms: i64,
    created_at_unix_ms: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupervisorTask {
    id: String,
    goal_id: String,
    sequence: i64,
    state: String,
    attempts: i64,
    outcome: Option<String>,
    evidence: Option<serde_json::Value>,
    created_at_unix_ms: i64,
    completed_at_unix_ms: Option<i64>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SupervisorSnapshot {
    paused: bool,
    goals: Vec<SupervisorGoal>,
    tasks: Vec<SupervisorTask>,
    tasks_per_wake: i64,
    tasks_per_hour: i64,
    capability: &'static str,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateGoalRequest {
    objective: String,
    priority: i64,
    max_tasks: i64,
    #[serde(default)]
    sleep_until_unix_ms: Option<i64>,
}

pub struct SupervisorState {
    conn: Mutex<Connection>,
    signal: Arc<(Mutex<bool>, Condvar)>,
}
impl SupervisorState {
    pub fn load(config_dir: &Path) -> Result<Self, String> {
        std::fs::create_dir_all(config_dir)
            .map_err(|_| "create supervisor data directory failed")?;
        Self::from_connection(
            Connection::open(config_dir.join("supervisor.sqlite3"))
                .map_err(|_| "open supervisor database failed")?,
        )
    }
    fn from_connection(conn: Connection) -> Result<Self, String> {
        conn.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE IF NOT EXISTS supervisor_config(id INTEGER PRIMARY KEY CHECK(id=1),paused INTEGER NOT NULL); INSERT OR IGNORE INTO supervisor_config VALUES(1,0); CREATE TABLE IF NOT EXISTS supervisor_goals(id TEXT PRIMARY KEY,objective TEXT NOT NULL,priority INTEGER NOT NULL,max_tasks INTEGER NOT NULL,completed_tasks INTEGER NOT NULL,state TEXT NOT NULL,next_wake_at INTEGER NOT NULL,created_at INTEGER NOT NULL); CREATE TABLE IF NOT EXISTS supervisor_tasks(id TEXT PRIMARY KEY,goal_id TEXT NOT NULL REFERENCES supervisor_goals(id),sequence INTEGER NOT NULL,state TEXT NOT NULL,attempts INTEGER NOT NULL,lease_until INTEGER,outcome TEXT,evidence_json TEXT,created_at INTEGER NOT NULL,completed_at INTEGER,UNIQUE(goal_id,sequence));").map_err(|_|"migrate supervisor database failed")?;
        let now = now_ms();
        conn.execute("UPDATE supervisor_tasks SET state='pending',lease_until=NULL WHERE state='running' AND lease_until<=?1",[now]).map_err(|_|"recover supervisor leases failed")?;
        Ok(Self {
            conn: Mutex::new(conn),
            signal: Arc::new((Mutex::new(false), Condvar::new())),
        })
    }
    pub fn signal(&self) {
        let (lock, cvar) = &*self.signal;
        if let Ok(mut pending) = lock.lock() {
            *pending = true;
            cvar.notify_one()
        }
    }
    pub fn wait(&self) {
        let (lock, cvar) = &*self.signal;
        if let Ok(pending) = lock.lock() {
            let _ = cvar
                .wait_timeout_while(pending, Duration::from_secs(30), |v| !*v)
                .map(|(mut v, _)| *v = false);
        }
    }
    fn create_goal(&self, r: CreateGoalRequest) -> Result<SupervisorGoal, String> {
        if r.objective.trim().is_empty() || r.objective.len() > 4096 {
            return Err("objective must be non-empty and at most 4096 bytes".into());
        }
        if !(0..=100).contains(&r.priority) {
            return Err("priority must be between 0 and 100".into());
        }
        if !(1..=MAX_TASKS_PER_GOAL).contains(&r.max_tasks) {
            return Err("maxTasks must be between 1 and 100".into());
        }
        let now = now_ms();
        let goal = SupervisorGoal {
            id: Uuid::new_v4().to_string(),
            objective: r.objective,
            priority: r.priority,
            max_tasks: r.max_tasks,
            completed_tasks: 0,
            state: "active".into(),
            next_wake_at_unix_ms: r.sleep_until_unix_ms.unwrap_or(now),
            created_at_unix_ms: now,
        };
        self.conn
            .lock()
            .map_err(|_| "supervisor state unavailable")?
            .execute(
                "INSERT INTO supervisor_goals VALUES(?1,?2,?3,?4,0,'active',?5,?6)",
                params![
                    goal.id,
                    goal.objective,
                    goal.priority,
                    goal.max_tasks,
                    goal.next_wake_at_unix_ms,
                    goal.created_at_unix_ms
                ],
            )
            .map_err(|_| "persist supervisor goal failed")?;
        self.signal();
        Ok(goal)
    }
    fn set_paused(&self, paused: bool) -> Result<(), String> {
        self.conn
            .lock()
            .map_err(|_| "supervisor state unavailable")?
            .execute(
                "UPDATE supervisor_config SET paused=?1 WHERE id=1",
                [i64::from(paused)],
            )
            .map_err(|_| "update supervisor pause failed")?;
        if !paused {
            self.signal()
        }
        Ok(())
    }
    fn wake(&self, world: &WorldState, now: i64) -> Result<Vec<SupervisorTask>, String> {
        let mut guard = self
            .conn
            .lock()
            .map_err(|_| "supervisor state unavailable")?;
        guard.execute("UPDATE supervisor_tasks SET state='pending',lease_until=NULL WHERE state='running' AND lease_until<=?1",[now]).map_err(|_|"recover expired supervisor leases failed")?;
        let paused: bool = guard
            .query_row("SELECT paused FROM supervisor_config WHERE id=1", [], |r| {
                r.get(0)
            })
            .map_err(|_| "read supervisor pause failed")?;
        if paused {
            return Ok(vec![]);
        }
        let hour_start = now.saturating_sub(3_600_000);
        let used:i64=guard.query_row("SELECT COUNT(*) FROM supervisor_tasks WHERE state='completed' AND completed_at>=?1",[hour_start],|r|r.get(0)).map_err(|_|"read supervisor quota failed")?;
        let budget = TASKS_PER_WAKE.min((TASKS_PER_HOUR - used).max(0));
        let mut completed = vec![];
        for _ in 0..budget {
            let tx = guard
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(|_| "begin supervisor claim failed")?;
            let candidate=tx.query_row("SELECT id,completed_tasks FROM supervisor_goals WHERE state='active' AND next_wake_at<=?1 ORDER BY priority DESC,created_at,id LIMIT 1",[now],|r|Ok((r.get::<_,String>(0)?,r.get::<_,i64>(1)?))).optional().map_err(|_|"select supervisor goal failed")?;
            let Some((goal_id, completed_count)) = candidate else {
                tx.commit()
                    .map_err(|_| "commit empty supervisor wake failed")?;
                break;
            };
            let sequence = completed_count + 1;
            let task_id = Uuid::new_v4().to_string();
            tx.execute("INSERT INTO supervisor_tasks(id,goal_id,sequence,state,attempts,lease_until,created_at) VALUES(?1,?2,?3,'running',1,?4,?5) ON CONFLICT(goal_id,sequence) DO UPDATE SET state='running',attempts=attempts+1,lease_until=excluded.lease_until",params![task_id,goal_id,sequence,now+LEASE_MS,now]).map_err(|_|"claim supervisor task failed")?;
            let actual_id: String = tx
                .query_row(
                    "SELECT id FROM supervisor_tasks WHERE goal_id=?1 AND sequence=?2",
                    params![goal_id, sequence],
                    |r| r.get(0),
                )
                .map_err(|_| "read claimed task failed")?;
            tx.commit().map_err(|_| "commit supervisor claim failed")?;
            let evidence = world.integrity_summary().and_then(|summary| {
                serde_json::to_value(summary)
                    .map_err(|_| "serialize supervisor evidence failed".to_string())
            });
            let tx = guard
                .transaction_with_behavior(TransactionBehavior::Immediate)
                .map_err(|_| "begin supervisor completion failed")?;
            match evidence {
                Ok(e) => {
                    tx.execute("UPDATE supervisor_tasks SET state='completed',lease_until=NULL,outcome='world_integrity_audit',evidence_json=?2,completed_at=?3 WHERE id=?1",params![actual_id,e.to_string(),now]).map_err(|_|"complete supervisor task failed")?;
                    tx.execute("UPDATE supervisor_goals SET completed_tasks=completed_tasks+1,state=CASE WHEN completed_tasks+1>=max_tasks THEN 'completed' ELSE 'active' END,next_wake_at=?2 WHERE id=?1",params![goal_id,now+1_000]).map_err(|_|"advance supervisor goal failed")?;
                }
                Err(error) => {
                    tx.execute("UPDATE supervisor_tasks SET state=CASE WHEN attempts>=3 THEN 'failed' ELSE 'pending' END,lease_until=NULL,outcome=?2 WHERE id=?1",params![actual_id,error]).map_err(|_|"retry supervisor task failed")?;
                }
            }
            tx.commit()
                .map_err(|_| "commit supervisor completion failed")?;
            let task = read_task(&guard, &actual_id)?;
            completed.push(task)
        }
        Ok(completed)
    }
    fn snapshot(&self) -> Result<SupervisorSnapshot, String> {
        let guard = self
            .conn
            .lock()
            .map_err(|_| "supervisor state unavailable")?;
        let paused = guard
            .query_row("SELECT paused FROM supervisor_config WHERE id=1", [], |r| {
                r.get(0)
            })
            .map_err(|_| "read supervisor config failed")?;
        let goals = {
            let mut q=guard.prepare("SELECT id,objective,priority,max_tasks,completed_tasks,state,next_wake_at,created_at FROM supervisor_goals ORDER BY priority DESC,created_at").map_err(|_|"read supervisor goals failed")?;
            q.query_map([], |r| {
                Ok(SupervisorGoal {
                    id: r.get(0)?,
                    objective: r.get(1)?,
                    priority: r.get(2)?,
                    max_tasks: r.get(3)?,
                    completed_tasks: r.get(4)?,
                    state: r.get(5)?,
                    next_wake_at_unix_ms: r.get(6)?,
                    created_at_unix_ms: r.get(7)?,
                })
            })
            .map_err(|_| "read supervisor goals failed")?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "read supervisor goals failed")?
        };
        let tasks = {
            let mut q=guard.prepare("SELECT id,goal_id,sequence,state,attempts,outcome,evidence_json,created_at,completed_at FROM supervisor_tasks ORDER BY created_at DESC LIMIT 200").map_err(|_|"read supervisor tasks failed")?;
            q.query_map([], task_from_row)
                .map_err(|_| "read supervisor tasks failed")?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| "read supervisor tasks failed")?
        };
        Ok(SupervisorSnapshot {
            paused,
            goals,
            tasks,
            tasks_per_wake: TASKS_PER_WAKE,
            tasks_per_hour: TASKS_PER_HOUR,
            capability: "internal_world_integrity_audit_only",
        })
    }
}
use rusqlite::OptionalExtension;
fn task_from_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<SupervisorTask> {
    let evidence: Option<String> = r.get(6)?;
    Ok(SupervisorTask {
        id: r.get(0)?,
        goal_id: r.get(1)?,
        sequence: r.get(2)?,
        state: r.get(3)?,
        attempts: r.get(4)?,
        outcome: r.get(5)?,
        evidence: evidence.and_then(|v| serde_json::from_str(&v).ok()),
        created_at_unix_ms: r.get(7)?,
        completed_at_unix_ms: r.get(8)?,
    })
}
fn read_task(conn: &Connection, id: &str) -> Result<SupervisorTask, String> {
    conn.query_row("SELECT id,goal_id,sequence,state,attempts,outcome,evidence_json,created_at,completed_at FROM supervisor_tasks WHERE id=?1",[id],task_from_row).map_err(|_|"read completed supervisor task failed".into())
}
fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(i64::MAX)
}
pub fn run_wake(
    state: &SupervisorState,
    world: &WorldState,
) -> Result<Vec<SupervisorTask>, String> {
    state.wake(world, now_ms())
}
#[tauri::command]
pub fn get_supervisor_snapshot(
    state: State<'_, SupervisorState>,
) -> Result<SupervisorSnapshot, String> {
    state.snapshot()
}
#[tauri::command]
pub fn create_supervisor_goal(
    state: State<'_, SupervisorState>,
    request: CreateGoalRequest,
) -> Result<SupervisorGoal, String> {
    state.create_goal(request)
}
#[tauri::command]
pub fn set_supervisor_paused(
    state: State<'_, SupervisorState>,
    paused: bool,
) -> Result<SupervisorSnapshot, String> {
    state.set_paused(paused)?;
    state.snapshot()
}

#[cfg(test)]
mod tests {
    use super::*;
    fn states() -> (SupervisorState, WorldState) {
        (
            SupervisorState::from_connection(Connection::open_in_memory().unwrap()).unwrap(),
            WorldState::test_state(),
        )
    }
    #[test]
    fn completes_bounded_task_without_prompt() {
        let (s, w) = states();
        s.create_goal(CreateGoalRequest {
            objective: "Audit world".into(),
            priority: 50,
            max_tasks: 1,
            sleep_until_unix_ms: Some(1),
        })
        .unwrap();
        let done = s.wake(&w, 2).unwrap();
        assert_eq!(done.len(), 1);
        assert_eq!(done[0].state, "completed");
        let snap = s.snapshot().unwrap();
        assert_eq!(snap.goals[0].state, "completed");
    }
    #[test]
    fn pause_prevents_execution() {
        let (s, w) = states();
        s.create_goal(CreateGoalRequest {
            objective: "Audit".into(),
            priority: 1,
            max_tasks: 1,
            sleep_until_unix_ms: Some(1),
        })
        .unwrap();
        s.set_paused(true).unwrap();
        assert!(s.wake(&w, 2).unwrap().is_empty());
        assert_eq!(s.snapshot().unwrap().goals[0].completed_tasks, 0);
    }
    #[test]
    fn priority_is_deterministic() {
        let (s, w) = states();
        s.create_goal(CreateGoalRequest {
            objective: "low".into(),
            priority: 1,
            max_tasks: 1,
            sleep_until_unix_ms: Some(1),
        })
        .unwrap();
        let high = s
            .create_goal(CreateGoalRequest {
                objective: "high".into(),
                priority: 100,
                max_tasks: 1,
                sleep_until_unix_ms: Some(1),
            })
            .unwrap();
        let done = s.wake(&w, 2).unwrap();
        assert_eq!(done[0].goal_id, high.id);
    }
    #[test]
    fn expired_running_task_recovers_on_restart() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("s.sqlite3");
        {
            let s = SupervisorState::from_connection(Connection::open(&path).unwrap()).unwrap();
            let g = s
                .create_goal(CreateGoalRequest {
                    objective: "recover".into(),
                    priority: 1,
                    max_tasks: 1,
                    sleep_until_unix_ms: Some(1),
                })
                .unwrap();
            s.conn.lock().unwrap().execute("INSERT INTO supervisor_tasks(id,goal_id,sequence,state,attempts,lease_until,created_at) VALUES('t',?1,1,'running',1,0,0)",[g.id]).unwrap();
        }
        let reopened = SupervisorState::from_connection(Connection::open(path).unwrap()).unwrap();
        assert_eq!(reopened.snapshot().unwrap().tasks[0].state, "pending");
    }
}
