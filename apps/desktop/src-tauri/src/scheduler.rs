//! Durable reminders, wake triggers, and retryable notification delivery.

use rusqlite::{Connection, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::State;
use uuid::Uuid;

const MIN_INTERVAL_MS: i64 = 60_000;
const MAX_TEXT_BYTES: usize = 4_096;
const MAX_RETRIES: i64 = 8;
const MAX_ACTIVE_TASKS: i64 = 100;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScheduledTask {
    id: String,
    title: String,
    message: String,
    schedule_kind: String,
    next_run_at_unix_ms: i64,
    interval_ms: Option<i64>,
    state: String,
    occurrence_count: i64,
    created_at_unix_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ReminderNotification {
    id: String,
    task_id: String,
    occurrence_at_unix_ms: i64,
    title: String,
    message: String,
    delivery_state: String,
    delivery_attempts: i64,
    created_at_unix_ms: i64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateScheduleRequest {
    title: String,
    message: String,
    run_at_unix_ms: i64,
    #[serde(default)]
    interval_ms: Option<i64>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SchedulerSnapshot {
    tasks: Vec<ScheduledTask>,
    notifications: Vec<ReminderNotification>,
    wake_trigger: &'static str,
    retry_policy: &'static str,
}

pub struct SchedulerState {
    conn: Mutex<Connection>,
}

impl SchedulerState {
    pub fn load(config_dir: &Path) -> Result<Self, String> {
        std::fs::create_dir_all(config_dir)
            .map_err(|_| "create scheduler data directory failed")?;
        Self::from_connection(
            Connection::open(config_dir.join("scheduler.sqlite3"))
                .map_err(|_| "open scheduler database failed")?,
        )
    }
    fn from_connection(conn: Connection) -> Result<Self, String> {
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;
        CREATE TABLE IF NOT EXISTS scheduled_tasks(id TEXT PRIMARY KEY,title TEXT NOT NULL,message TEXT NOT NULL,schedule_kind TEXT NOT NULL,next_run_at INTEGER NOT NULL,interval_ms INTEGER,state TEXT NOT NULL,occurrence_count INTEGER NOT NULL DEFAULT 0,created_at INTEGER NOT NULL);
        CREATE TABLE IF NOT EXISTS reminder_notifications(id TEXT PRIMARY KEY,task_id TEXT NOT NULL REFERENCES scheduled_tasks(id),occurrence_at INTEGER NOT NULL,title TEXT NOT NULL,message TEXT NOT NULL,delivery_state TEXT NOT NULL,delivery_attempts INTEGER NOT NULL DEFAULT 0,next_delivery_at INTEGER NOT NULL,created_at INTEGER NOT NULL,UNIQUE(task_id,occurrence_at));").map_err(|_|"migrate scheduler database failed")?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
    fn create(&self, r: CreateScheduleRequest) -> Result<ScheduledTask, String> {
        validate_text(&r.title, "title")?;
        validate_text(&r.message, "message")?;
        if r.run_at_unix_ms <= 0 {
            return Err("run time must be a positive Unix timestamp".into());
        }
        if let Some(interval) = r.interval_ms {
            if interval < MIN_INTERVAL_MS {
                return Err("repeat interval must be at least 60000 ms".into());
            }
        }
        let guard = self
            .conn
            .lock()
            .map_err(|_| "scheduler state unavailable")?;
        let active: i64 = guard
            .query_row(
                "SELECT COUNT(*) FROM scheduled_tasks WHERE state='active'",
                [],
                |row| row.get(0),
            )
            .map_err(|_| "count active schedules failed")?;
        if active >= MAX_ACTIVE_TASKS {
            return Err("active schedule limit reached".into());
        }
        let task = ScheduledTask {
            id: Uuid::new_v4().to_string(),
            title: r.title,
            message: r.message,
            schedule_kind: if r.interval_ms.is_some() {
                "interval".into()
            } else {
                "once".into()
            },
            next_run_at_unix_ms: r.run_at_unix_ms,
            interval_ms: r.interval_ms,
            state: "active".into(),
            occurrence_count: 0,
            created_at_unix_ms: now_ms(),
        };
        guard
            .execute(
                "INSERT INTO scheduled_tasks VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                params![
                    task.id,
                    task.title,
                    task.message,
                    task.schedule_kind,
                    task.next_run_at_unix_ms,
                    task.interval_ms,
                    task.state,
                    task.occurrence_count,
                    task.created_at_unix_ms
                ],
            )
            .map_err(|_| "persist scheduled task failed")?;
        Ok(task)
    }
    fn cancel(&self, id: &str) -> Result<bool, String> {
        Ok(self
            .conn
            .lock()
            .map_err(|_| "scheduler state unavailable")?
            .execute(
                "UPDATE scheduled_tasks SET state='cancelled' WHERE id=?1 AND state='active'",
                [id],
            )
            .map_err(|_| "cancel scheduled task failed")?
            > 0)
    }
    fn process_due(&self, now: i64) -> Result<Vec<ReminderNotification>, String> {
        let mut guard = self
            .conn
            .lock()
            .map_err(|_| "scheduler state unavailable")?;
        let tx = guard
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| "begin scheduler wake failed")?;
        let due = {
            let mut q=tx.prepare("SELECT id,title,message,next_run_at,interval_ms FROM scheduled_tasks WHERE state='active' AND next_run_at<=?1 ORDER BY next_run_at,id").map_err(|_|"read due schedules failed")?;
            q.query_map([now], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, i64>(3)?,
                    r.get::<_, Option<i64>>(4)?,
                ))
            })
            .map_err(|_| "read due schedules failed")?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "read due schedules failed")?
        };
        for (id, title, message, occurrence, interval) in due {
            let notification_id = Uuid::new_v4().to_string();
            tx.execute("INSERT OR IGNORE INTO reminder_notifications VALUES(?1,?2,?3,?4,?5,'pending',0,?6,?6)",params![notification_id,id,occurrence,title,message,now]).map_err(|_|"create reminder notification failed")?;
            if let Some(ms) = interval {
                let skipped = ((now - occurrence) / ms).max(0) + 1;
                let next = occurrence.saturating_add(ms.saturating_mul(skipped));
                tx.execute("UPDATE scheduled_tasks SET next_run_at=?2,occurrence_count=occurrence_count+1 WHERE id=?1",params![id,next]).map_err(|_|"advance repeating schedule failed")?;
            } else {
                tx.execute("UPDATE scheduled_tasks SET state='completed',occurrence_count=occurrence_count+1 WHERE id=?1",[id]).map_err(|_|"complete schedule failed")?;
            }
        }
        tx.commit().map_err(|_| "commit scheduler wake failed")?;
        drop(guard);
        self.claim_deliveries(now)
    }
    fn claim_deliveries(&self, now: i64) -> Result<Vec<ReminderNotification>, String> {
        let guard = self
            .conn
            .lock()
            .map_err(|_| "scheduler state unavailable")?;
        let rows = {
            let mut q=guard.prepare("SELECT id,task_id,occurrence_at,title,message,delivery_state,delivery_attempts,created_at FROM reminder_notifications WHERE delivery_state='pending' AND next_delivery_at<=?1 AND delivery_attempts<?2 ORDER BY created_at").map_err(|_|"read pending notifications failed")?;
            q.query_map(params![now, MAX_RETRIES], notification_from_row)
                .map_err(|_| "read pending notifications failed")?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| "read pending notifications failed")?
        };
        for item in &rows {
            let attempts = item.delivery_attempts + 1;
            let delay = 1_000_i64.saturating_mul(1_i64 << attempts.min(6));
            guard.execute("UPDATE reminder_notifications SET delivery_attempts=?2,next_delivery_at=?3 WHERE id=?1 AND delivery_state='pending'",params![item.id,attempts,now.saturating_add(delay)]).map_err(|_|"schedule notification retry failed")?;
        }
        Ok(rows
            .into_iter()
            .map(|mut n| {
                n.delivery_attempts += 1;
                n
            })
            .collect())
    }
    fn acknowledge(&self, id: &str) -> Result<bool, String> {
        Ok(self.conn.lock().map_err(|_|"scheduler state unavailable")?.execute("UPDATE reminder_notifications SET delivery_state='acknowledged' WHERE id=?1 AND delivery_state='pending'",[id]).map_err(|_|"acknowledge notification failed")?>0)
    }
    fn snapshot(&self) -> Result<SchedulerSnapshot, String> {
        let guard = self
            .conn
            .lock()
            .map_err(|_| "scheduler state unavailable")?;
        let tasks = {
            let mut q=guard.prepare("SELECT id,title,message,schedule_kind,next_run_at,interval_ms,state,occurrence_count,created_at FROM scheduled_tasks ORDER BY created_at DESC").map_err(|_|"read scheduled tasks failed")?;
            q.query_map([], |r| {
                Ok(ScheduledTask {
                    id: r.get(0)?,
                    title: r.get(1)?,
                    message: r.get(2)?,
                    schedule_kind: r.get(3)?,
                    next_run_at_unix_ms: r.get(4)?,
                    interval_ms: r.get(5)?,
                    state: r.get(6)?,
                    occurrence_count: r.get(7)?,
                    created_at_unix_ms: r.get(8)?,
                })
            })
            .map_err(|_| "read scheduled tasks failed")?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "read scheduled tasks failed")?
        };
        let notifications = {
            let mut q=guard.prepare("SELECT id,task_id,occurrence_at,title,message,delivery_state,delivery_attempts,created_at FROM reminder_notifications ORDER BY created_at DESC LIMIT 100").map_err(|_|"read notifications failed")?;
            q.query_map([], notification_from_row)
                .map_err(|_| "read notifications failed")?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| "read notifications failed")?
        };
        Ok(SchedulerSnapshot {
            tasks,
            notifications,
            wake_trigger: "native_core_1s_tick",
            retry_policy: "durable_exponential_backoff_max_8",
        })
    }
}

fn notification_from_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<ReminderNotification> {
    Ok(ReminderNotification {
        id: r.get(0)?,
        task_id: r.get(1)?,
        occurrence_at_unix_ms: r.get(2)?,
        title: r.get(3)?,
        message: r.get(4)?,
        delivery_state: r.get(5)?,
        delivery_attempts: r.get(6)?,
        created_at_unix_ms: r.get(7)?,
    })
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
pub fn now_ms() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(i64::MAX)
}
pub fn wake(state: &SchedulerState) -> Result<Vec<ReminderNotification>, String> {
    state.process_due(now_ms())
}

#[tauri::command]
pub fn get_scheduler_snapshot(
    state: State<'_, SchedulerState>,
) -> Result<SchedulerSnapshot, String> {
    state.snapshot()
}
#[tauri::command]
pub fn create_scheduled_task(
    state: State<'_, SchedulerState>,
    request: CreateScheduleRequest,
) -> Result<ScheduledTask, String> {
    state.create(request)
}
#[tauri::command]
pub fn cancel_scheduled_task(state: State<'_, SchedulerState>, id: String) -> Result<bool, String> {
    state.cancel(&id)
}
#[tauri::command]
pub fn acknowledge_notification(
    state: State<'_, SchedulerState>,
    id: String,
) -> Result<bool, String> {
    state.acknowledge(&id)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn state() -> SchedulerState {
        SchedulerState::from_connection(Connection::open_in_memory().unwrap()).unwrap()
    }
    #[test]
    fn restart_does_not_duplicate_occurrence() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("scheduler.sqlite3");
        {
            let s = SchedulerState::from_connection(Connection::open(&path).unwrap()).unwrap();
            s.create(CreateScheduleRequest {
                title: "Stand up".into(),
                message: "Move".into(),
                run_at_unix_ms: 100,
                interval_ms: None,
            })
            .unwrap();
            assert_eq!(s.process_due(100).unwrap().len(), 1);
        }
        let reopened = SchedulerState::from_connection(Connection::open(&path).unwrap()).unwrap();
        reopened.process_due(10_000).unwrap();
        let snapshot = reopened.snapshot().unwrap();
        assert_eq!(snapshot.notifications.len(), 1);
        assert_eq!(snapshot.tasks[0].occurrence_count, 1);
        assert_eq!(snapshot.tasks[0].state, "completed");
    }
    #[test]
    fn repeated_wake_is_idempotent() {
        let s = state();
        s.create(CreateScheduleRequest {
            title: "One".into(),
            message: "Only once".into(),
            run_at_unix_ms: 100,
            interval_ms: None,
        })
        .unwrap();
        s.process_due(100).unwrap();
        s.process_due(100).unwrap();
        assert_eq!(s.snapshot().unwrap().notifications.len(), 1);
    }
    #[test]
    fn repeating_schedule_skips_missed_burst_and_advances() {
        let s = state();
        s.create(CreateScheduleRequest {
            title: "Pulse".into(),
            message: "Check".into(),
            run_at_unix_ms: 100,
            interval_ms: Some(60_000),
        })
        .unwrap();
        s.process_due(180_100).unwrap();
        let snap = s.snapshot().unwrap();
        assert_eq!(snap.notifications.len(), 1);
        assert!(snap.tasks[0].next_run_at_unix_ms > 180_100);
    }
    #[test]
    fn retry_is_durable_until_acknowledged() {
        let s = state();
        s.create(CreateScheduleRequest {
            title: "Retry".into(),
            message: "Me".into(),
            run_at_unix_ms: 100,
            interval_ms: None,
        })
        .unwrap();
        let first = s.process_due(100).unwrap();
        assert_eq!(first[0].delivery_attempts, 1);
        let second = s.claim_deliveries(2_100).unwrap();
        assert_eq!(second[0].delivery_attempts, 2);
        assert!(s.acknowledge(&first[0].id).unwrap());
        assert!(s.claim_deliveries(10_000).unwrap().is_empty());
    }
    #[test]
    fn interval_has_lower_bound() {
        assert!(
            state()
                .create(CreateScheduleRequest {
                    title: "x".into(),
                    message: "y".into(),
                    run_at_unix_ms: 1,
                    interval_ms: Some(10)
                })
                .is_err()
        );
    }
}
