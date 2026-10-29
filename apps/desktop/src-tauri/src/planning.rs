//! Durable deterministic goals, plans, tasks, and evidence reviews.

use crate::world::WorldState;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    path::Path,
    sync::{Arc, Condvar, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannedGoal {
    id: String,
    objective: String,
    success_criterion: String,
    status: String,
    current_plan_version: i64,
    created_at_unix_ms: i64,
    updated_at_unix_ms: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    id: String,
    goal_id: String,
    version: i64,
    rationale: String,
    status: String,
    created_at_unix_ms: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlannedTask {
    id: String,
    plan_id: String,
    ordinal: i64,
    task_type: String,
    status: String,
    selection_reason: String,
    evidence: Option<serde_json::Value>,
    created_at_unix_ms: i64,
    completed_at_unix_ms: Option<i64>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EvidenceReview {
    id: String,
    goal_id: String,
    plan_version: i64,
    task_id: String,
    verdict: String,
    reason: String,
    evidence_digest: String,
    created_at_unix_ms: i64,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlanningSnapshot {
    goals: Vec<PlannedGoal>,
    plans: Vec<Plan>,
    tasks: Vec<PlannedTask>,
    reviews: Vec<EvidenceReview>,
    capability: &'static str,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreatePlannedGoalRequest {
    objective: String,
    success_criterion: String,
}

pub struct PlanningState {
    conn: Mutex<Connection>,
    signal: Arc<(Mutex<bool>, Condvar)>,
}
impl PlanningState {
    pub fn load(config_dir: &Path) -> Result<Self, String> {
        std::fs::create_dir_all(config_dir).map_err(|_| "create planning directory failed")?;
        Self::from_connection(
            Connection::open(config_dir.join("planning.sqlite3"))
                .map_err(|_| "open planning database failed")?,
        )
    }
    fn from_connection(conn: Connection) -> Result<Self, String> {
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; CREATE TABLE IF NOT EXISTS planned_goals(id TEXT PRIMARY KEY,objective TEXT NOT NULL,success_criterion TEXT NOT NULL,status TEXT NOT NULL,current_plan_version INTEGER NOT NULL,created_at INTEGER NOT NULL,updated_at INTEGER NOT NULL); CREATE TABLE IF NOT EXISTS plans(id TEXT PRIMARY KEY,goal_id TEXT NOT NULL REFERENCES planned_goals(id),version INTEGER NOT NULL,rationale TEXT NOT NULL,status TEXT NOT NULL,created_at INTEGER NOT NULL,UNIQUE(goal_id,version)); CREATE TABLE IF NOT EXISTS planned_tasks(id TEXT PRIMARY KEY,plan_id TEXT NOT NULL REFERENCES plans(id),ordinal INTEGER NOT NULL,task_type TEXT NOT NULL,status TEXT NOT NULL,selection_reason TEXT NOT NULL,evidence_json TEXT,created_at INTEGER NOT NULL,completed_at INTEGER,UNIQUE(plan_id,ordinal)); CREATE TABLE IF NOT EXISTS evidence_reviews(id TEXT PRIMARY KEY,goal_id TEXT NOT NULL,plan_version INTEGER NOT NULL,task_id TEXT NOT NULL,verdict TEXT NOT NULL,reason TEXT NOT NULL,evidence_digest TEXT NOT NULL,created_at INTEGER NOT NULL);").map_err(|_|"migrate planning database failed")?;
        Ok(Self {
            conn: Mutex::new(conn),
            signal: Arc::new((Mutex::new(false), Condvar::new())),
        })
    }
    pub fn signal(&self) {
        let (lock, cvar) = &*self.signal;
        if let Ok(mut value) = lock.lock() {
            *value = true;
            cvar.notify_one()
        }
    }
    pub fn wait(&self) {
        let (lock, cvar) = &*self.signal;
        if let Ok(value) = lock.lock() {
            let _ = cvar
                .wait_timeout_while(value, Duration::from_secs(30), |pending| !*pending)
                .map(|(mut v, _)| *v = false);
        }
    }
    fn create_goal(&self, r: CreatePlannedGoalRequest) -> Result<PlannedGoal, String> {
        validate(&r.objective, "objective")?;
        validate(&r.success_criterion, "success criterion")?;
        let now = now_ms();
        let goal = PlannedGoal {
            id: Uuid::new_v4().to_string(),
            objective: r.objective,
            success_criterion: r.success_criterion,
            status: "active".into(),
            current_plan_version: 1,
            created_at_unix_ms: now,
            updated_at_unix_ms: now,
        };
        let mut guard = self.conn.lock().map_err(|_| "planning state unavailable")?;
        let tx = guard
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| "begin goal planning failed")?;
        tx.execute(
            "INSERT INTO planned_goals VALUES(?1,?2,?3,'active',1,?4,?4)",
            params![goal.id, goal.objective, goal.success_criterion, now],
        )
        .map_err(|_| "persist planned goal failed")?;
        create_plan(
            &tx,
            &goal.id,
            1,
            "Verify the durable world reconstruction before taking any broader action.",
            "world_integrity_audit",
            "This read-only audit is the lowest-risk task that directly tests the stated success criterion.",
            now,
        )?;
        tx.commit().map_err(|_| "commit goal plan failed")?;
        self.signal();
        Ok(goal)
    }
    fn wake(&self, world: &WorldState) -> Result<Option<PlannedTask>, String> {
        let mut guard = self.conn.lock().map_err(|_| "planning state unavailable")?;
        let candidate:Option<(String,String,i64)>=guard.query_row("SELECT t.id,g.id,g.current_plan_version FROM planned_tasks t JOIN plans p ON p.id=t.plan_id JOIN planned_goals g ON g.id=p.goal_id WHERE g.status='active' AND p.status='active' AND t.status='pending' ORDER BY g.created_at,p.version,t.ordinal LIMIT 1",[],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?))).optional().map_err(|_|"select planned task failed")?;
        let Some((task_id, goal_id, plan_version)) = candidate else {
            return Ok(None);
        };
        guard
            .execute(
                "UPDATE planned_tasks SET status='running' WHERE id=?1 AND status='pending'",
                [&task_id],
            )
            .map_err(|_| "claim planned task failed")?;
        let summary = world.integrity_summary()?;
        let evidence =
            serde_json::to_value(&summary).map_err(|_| "serialize planning evidence failed")?;
        let (verdict, reason) = review_decision(summary.consistent, plan_version);
        let now = now_ms();
        let digest = format!(
            "sha256:{:x}",
            Sha256::digest(evidence.to_string().as_bytes())
        );
        let tx = guard
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| "begin evidence review failed")?;
        tx.execute("UPDATE planned_tasks SET status='completed',evidence_json=?2,completed_at=?3 WHERE id=?1",params![task_id,evidence.to_string(),now]).map_err(|_|"complete planned task failed")?;
        let review = EvidenceReview {
            id: Uuid::new_v4().to_string(),
            goal_id: goal_id.clone(),
            plan_version,
            task_id: task_id.clone(),
            verdict: verdict.into(),
            reason: reason.into(),
            evidence_digest: digest,
            created_at_unix_ms: now,
        };
        tx.execute(
            "INSERT INTO evidence_reviews VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
            params![
                review.id,
                review.goal_id,
                review.plan_version,
                review.task_id,
                review.verdict,
                review.reason,
                review.evidence_digest,
                review.created_at_unix_ms
            ],
        )
        .map_err(|_| "persist evidence review failed")?;
        match verdict {
            "stop_success" => {
                tx.execute(
                    "UPDATE planned_goals SET status='completed',updated_at=?2 WHERE id=?1",
                    params![goal_id, now],
                )
                .map_err(|_| "complete planned goal failed")?;
                tx.execute(
                    "UPDATE plans SET status='completed' WHERE goal_id=?1 AND version=?2",
                    params![goal_id, plan_version],
                )
                .map_err(|_| "complete plan failed")?;
            }
            "pivot" => {
                tx.execute(
                    "UPDATE plans SET status='superseded' WHERE goal_id=?1 AND version=?2",
                    params![goal_id, plan_version],
                )
                .map_err(|_| "supersede plan failed")?;
                tx.execute(
                    "UPDATE planned_goals SET current_plan_version=2,updated_at=?2 WHERE id=?1",
                    params![goal_id, now],
                )
                .map_err(|_| "pivot goal failed")?;
                create_plan(
                    &tx,
                    &goal_id,
                    2,
                    "World reconstruction is inconsistent; inventory the inconsistency and stop before any mutation.",
                    "world_inconsistency_inventory",
                    "The prior audit evidence failed the success criterion, so the safest pivot is a bounded diagnostic inventory.",
                    now,
                )?;
            }
            _ => {
                tx.execute(
                    "UPDATE planned_goals SET status='stopped',updated_at=?2 WHERE id=?1",
                    params![goal_id, now],
                )
                .map_err(|_| "stop planned goal failed")?;
                tx.execute(
                    "UPDATE plans SET status='stopped' WHERE goal_id=?1 AND version=?2",
                    params![goal_id, plan_version],
                )
                .map_err(|_| "stop plan failed")?;
            }
        }
        tx.commit().map_err(|_| "commit evidence review failed")?;
        let task = read_task(&guard, &task_id)?;
        if verdict == "pivot" {
            self.signal()
        }
        Ok(Some(task))
    }
    fn stop(&self, id: &str) -> Result<bool, String> {
        Ok(self.conn.lock().map_err(|_|"planning state unavailable")?.execute("UPDATE planned_goals SET status='stopped',updated_at=?2 WHERE id=?1 AND status='active'",params![id,now_ms()]).map_err(|_|"stop goal failed")?>0)
    }
    fn snapshot(&self) -> Result<PlanningSnapshot, String> {
        let guard = self.conn.lock().map_err(|_| "planning state unavailable")?;
        let goals = {
            let mut q=guard.prepare("SELECT id,objective,success_criterion,status,current_plan_version,created_at,updated_at FROM planned_goals ORDER BY created_at DESC").map_err(|_|"read planned goals failed")?;
            q.query_map([], |r| {
                Ok(PlannedGoal {
                    id: r.get(0)?,
                    objective: r.get(1)?,
                    success_criterion: r.get(2)?,
                    status: r.get(3)?,
                    current_plan_version: r.get(4)?,
                    created_at_unix_ms: r.get(5)?,
                    updated_at_unix_ms: r.get(6)?,
                })
            })
            .map_err(|_| "read planned goals failed")?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "read planned goals failed")?
        };
        let plans = {
            let mut q=guard.prepare("SELECT id,goal_id,version,rationale,status,created_at FROM plans ORDER BY created_at DESC").map_err(|_|"read plans failed")?;
            q.query_map([], |r| {
                Ok(Plan {
                    id: r.get(0)?,
                    goal_id: r.get(1)?,
                    version: r.get(2)?,
                    rationale: r.get(3)?,
                    status: r.get(4)?,
                    created_at_unix_ms: r.get(5)?,
                })
            })
            .map_err(|_| "read plans failed")?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "read plans failed")?
        };
        let tasks = {
            let mut q=guard.prepare("SELECT id,plan_id,ordinal,task_type,status,selection_reason,evidence_json,created_at,completed_at FROM planned_tasks ORDER BY created_at DESC").map_err(|_|"read planned tasks failed")?;
            q.query_map([], task_row)
                .map_err(|_| "read planned tasks failed")?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| "read planned tasks failed")?
        };
        let reviews = {
            let mut q=guard.prepare("SELECT id,goal_id,plan_version,task_id,verdict,reason,evidence_digest,created_at FROM evidence_reviews ORDER BY created_at DESC").map_err(|_|"read evidence reviews failed")?;
            q.query_map([], |r| {
                Ok(EvidenceReview {
                    id: r.get(0)?,
                    goal_id: r.get(1)?,
                    plan_version: r.get(2)?,
                    task_id: r.get(3)?,
                    verdict: r.get(4)?,
                    reason: r.get(5)?,
                    evidence_digest: r.get(6)?,
                    created_at_unix_ms: r.get(7)?,
                })
            })
            .map_err(|_| "read evidence reviews failed")?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "read evidence reviews failed")?
        };
        Ok(PlanningSnapshot {
            goals,
            plans,
            tasks,
            reviews,
            capability: "internal_world_review_only",
        })
    }
}
fn create_plan(
    tx: &rusqlite::Transaction<'_>,
    goal: &str,
    version: i64,
    rationale: &str,
    task_type: &str,
    selection: &str,
    now: i64,
) -> Result<(), String> {
    let plan_id = Uuid::new_v4().to_string();
    tx.execute(
        "INSERT INTO plans VALUES(?1,?2,?3,?4,'active',?5)",
        params![plan_id, goal, version, rationale, now],
    )
    .map_err(|_| "persist plan failed")?;
    tx.execute(
        "INSERT INTO planned_tasks VALUES(?1,?2,1,?3,'pending',?4,NULL,?5,NULL)",
        params![
            Uuid::new_v4().to_string(),
            plan_id,
            task_type,
            selection,
            now
        ],
    )
    .map_err(|_| "persist planned task failed")?;
    Ok(())
}
fn review_decision(consistent: bool, version: i64) -> (&'static str, &'static str) {
    if consistent {
        (
            "stop_success",
            "Evidence shows the durable world reconstruction is consistent; the success criterion is satisfied.",
        )
    } else if version < 2 {
        (
            "pivot",
            "Evidence contradicts the success criterion; pivot to a bounded inconsistency inventory.",
        )
    } else {
        (
            "stop_blocked",
            "Evidence remains inconsistent after the diagnostic pivot; stop without mutation and require owner review.",
        )
    }
}
fn task_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<PlannedTask> {
    let raw: Option<String> = r.get(6)?;
    Ok(PlannedTask {
        id: r.get(0)?,
        plan_id: r.get(1)?,
        ordinal: r.get(2)?,
        task_type: r.get(3)?,
        status: r.get(4)?,
        selection_reason: r.get(5)?,
        evidence: raw.and_then(|v| serde_json::from_str(&v).ok()),
        created_at_unix_ms: r.get(7)?,
        completed_at_unix_ms: r.get(8)?,
    })
}
fn read_task(conn: &Connection, id: &str) -> Result<PlannedTask, String> {
    conn.query_row("SELECT id,plan_id,ordinal,task_type,status,selection_reason,evidence_json,created_at,completed_at FROM planned_tasks WHERE id=?1",[id],task_row).map_err(|_|"read planned task failed".into())
}
fn validate(v: &str, label: &str) -> Result<(), String> {
    if v.trim().is_empty() || v.len() > 4096 {
        Err(format!("{label} must be non-empty and at most 4096 bytes"))
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
pub fn run_wake(state: &PlanningState, world: &WorldState) -> Result<Option<PlannedTask>, String> {
    state.wake(world)
}
#[tauri::command]
pub fn get_planning_snapshot(state: State<'_, PlanningState>) -> Result<PlanningSnapshot, String> {
    state.snapshot()
}
#[tauri::command]
pub fn create_planned_goal(
    state: State<'_, PlanningState>,
    request: CreatePlannedGoalRequest,
) -> Result<PlannedGoal, String> {
    state.create_goal(request)
}
#[tauri::command]
pub fn stop_planned_goal(state: State<'_, PlanningState>, id: String) -> Result<bool, String> {
    state.stop(&id)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn task_explains_choice_and_shows_evidence() {
        let p = PlanningState::from_connection(Connection::open_in_memory().unwrap()).unwrap();
        let w = WorldState::test_state();
        p.create_goal(CreatePlannedGoalRequest {
            objective: "Keep world coherent".into(),
            success_criterion: "Reconstruction is consistent".into(),
        })
        .unwrap();
        let task = p.wake(&w).unwrap().unwrap();
        assert!(task.selection_reason.contains("lowest-risk"));
        assert_eq!(task.status, "completed");
        assert_eq!(task.evidence.as_ref().unwrap()["consistent"], true);
        let snap = p.snapshot().unwrap();
        assert_eq!(snap.reviews[0].verdict, "stop_success");
        assert_eq!(snap.goals[0].status, "completed");
    }
    #[test]
    fn contradictory_evidence_pivots_then_stops() {
        assert_eq!(review_decision(false, 1).0, "pivot");
        assert_eq!(review_decision(false, 2).0, "stop_blocked");
    }
    #[test]
    fn owner_can_stop_before_execution() {
        let p = PlanningState::from_connection(Connection::open_in_memory().unwrap()).unwrap();
        let w = WorldState::test_state();
        let goal = p
            .create_goal(CreatePlannedGoalRequest {
                objective: "x".into(),
                success_criterion: "y".into(),
            })
            .unwrap();
        assert!(p.stop(&goal.id).unwrap());
        assert!(p.wake(&w).unwrap().is_none());
    }
}
