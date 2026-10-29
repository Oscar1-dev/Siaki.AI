//! Manual, owner-confirmed provider onboarding and recovery checklists.
//! There is deliberately no provider automation or credential input.

use crate::identity::IdentityState;
use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::State;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum WorkflowKind {
    Onboarding,
    Recovery,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowStep {
    id: String,
    workflow_id: String,
    ordinal: i64,
    key: String,
    title: String,
    instruction: String,
    status: String,
    requires_owner_confirmation: bool,
    automated: bool,
    confirmed_at_unix_ms: Option<i64>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AccountWorkflow {
    id: String,
    identity_id: String,
    provider: String,
    kind: WorkflowKind,
    status: String,
    created_at_unix_ms: i64,
    updated_at_unix_ms: i64,
    steps: Vec<WorkflowStep>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowAuditEvent {
    id: String,
    workflow_id: String,
    event_type: String,
    step_key: Option<String>,
    actor: &'static str,
    occurred_at_unix_ms: i64,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkflowSnapshot {
    workflows: Vec<AccountWorkflow>,
    audit: Vec<WorkflowAuditEvent>,
    automation: &'static str,
    secret_input_supported: bool,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateWorkflowRequest {
    identity_id: String,
    kind: WorkflowKind,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConfirmStepRequest {
    workflow_id: String,
    step_id: String,
    owner_confirmed: bool,
}

pub struct OnboardingState {
    conn: Mutex<Connection>,
}
impl OnboardingState {
    pub fn load(config_dir: &Path) -> Result<Self, String> {
        std::fs::create_dir_all(config_dir).map_err(|_| "create workflow directory failed")?;
        Self::from_connection(
            Connection::open(config_dir.join("account-workflows.sqlite3"))
                .map_err(|_| "open workflow database failed")?,
        )
    }
    fn from_connection(conn: Connection) -> Result<Self, String> {
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; CREATE TABLE IF NOT EXISTS account_workflows(id TEXT PRIMARY KEY,identity_id TEXT NOT NULL,provider TEXT NOT NULL,kind TEXT NOT NULL,status TEXT NOT NULL,created_at INTEGER NOT NULL,updated_at INTEGER NOT NULL); CREATE TABLE IF NOT EXISTS workflow_steps(id TEXT PRIMARY KEY,workflow_id TEXT NOT NULL REFERENCES account_workflows(id),ordinal INTEGER NOT NULL,key TEXT NOT NULL,title TEXT NOT NULL,instruction TEXT NOT NULL,status TEXT NOT NULL,confirmed_at INTEGER,UNIQUE(workflow_id,ordinal),UNIQUE(workflow_id,key)); CREATE TABLE IF NOT EXISTS workflow_audit(id TEXT PRIMARY KEY,workflow_id TEXT NOT NULL,event_type TEXT NOT NULL,step_key TEXT,actor TEXT NOT NULL,occurred_at INTEGER NOT NULL);").map_err(|_|"migrate account workflows failed")?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
    fn create(
        &self,
        identity: &IdentityState,
        r: CreateWorkflowRequest,
    ) -> Result<AccountWorkflow, String> {
        let provider = identity.active_provider(&r.identity_id)?;
        let definitions = template(&provider, &r.kind)?;
        let now = now_ms();
        let id = Uuid::new_v4().to_string();
        let mut guard = self.conn.lock().map_err(|_| "workflow state unavailable")?;
        let tx = guard
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| "begin workflow failed")?;
        tx.execute(
            "INSERT INTO account_workflows VALUES(?1,?2,?3,?4,'active',?5,?5)",
            params![id, r.identity_id, provider, kind_name(&r.kind), now],
        )
        .map_err(|_| "persist workflow failed")?;
        for (index, (key, title, instruction)) in definitions.iter().enumerate() {
            tx.execute(
                "INSERT INTO workflow_steps VALUES(?1,?2,?3,?4,?5,?6,'pending',NULL)",
                params![
                    Uuid::new_v4().to_string(),
                    id,
                    (index + 1) as i64,
                    key,
                    title,
                    instruction
                ],
            )
            .map_err(|_| "persist workflow step failed")?;
        }
        append_audit(&tx, &id, "workflow_created", None, now)?;
        tx.commit().map_err(|_| "commit workflow failed")?;
        drop(guard);
        self.get(&id)
    }
    fn confirm(&self, r: ConfirmStepRequest) -> Result<AccountWorkflow, String> {
        if !r.owner_confirmed {
            return Err("owner confirmation is required".into());
        }
        let mut guard = self.conn.lock().map_err(|_| "workflow state unavailable")?;
        let tx = guard
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| "begin step confirmation failed")?;
        let step: Option<(i64, String, String)> = tx
            .query_row(
                "SELECT ordinal,key,status FROM workflow_steps WHERE id=?1 AND workflow_id=?2",
                params![r.step_id, r.workflow_id],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .optional()
            .map_err(|_| "read workflow step failed")?;
        let Some((ordinal, key, status)) = step else {
            return Err("workflow step not found".into());
        };
        if status != "pending" {
            return Err("workflow step is not pending".into());
        }
        let prior:i64=tx.query_row("SELECT COUNT(*) FROM workflow_steps WHERE workflow_id=?1 AND ordinal<?2 AND status!='confirmed'",params![r.workflow_id,ordinal],|row|row.get(0)).map_err(|_|"check workflow order failed")?;
        if prior > 0 {
            return Err("prior workflow steps require owner confirmation".into());
        }
        let now = now_ms();
        tx.execute(
            "UPDATE workflow_steps SET status='confirmed',confirmed_at=?2 WHERE id=?1",
            params![r.step_id, now],
        )
        .map_err(|_| "confirm workflow step failed")?;
        append_audit(&tx, &r.workflow_id, "owner_confirmed_step", Some(&key), now)?;
        let remaining: i64 = tx
            .query_row(
                "SELECT COUNT(*) FROM workflow_steps WHERE workflow_id=?1 AND status!='confirmed'",
                [&r.workflow_id],
                |row| row.get(0),
            )
            .map_err(|_| "count workflow steps failed")?;
        if remaining == 0 {
            tx.execute(
                "UPDATE account_workflows SET status='completed',updated_at=?2 WHERE id=?1",
                params![r.workflow_id, now],
            )
            .map_err(|_| "complete workflow failed")?;
            append_audit(&tx, &r.workflow_id, "workflow_completed", None, now)?
        } else {
            tx.execute(
                "UPDATE account_workflows SET updated_at=?2 WHERE id=?1",
                params![r.workflow_id, now],
            )
            .map_err(|_| "update workflow failed")?;
        }
        tx.commit().map_err(|_| "commit step confirmation failed")?;
        drop(guard);
        self.get(&r.workflow_id)
    }
    fn cancel(&self, id: &str) -> Result<bool, String> {
        let mut guard = self.conn.lock().map_err(|_| "workflow state unavailable")?;
        let tx = guard
            .transaction_with_behavior(TransactionBehavior::Immediate)
            .map_err(|_| "begin workflow cancellation failed")?;
        let changed=tx.execute("UPDATE account_workflows SET status='cancelled',updated_at=?2 WHERE id=?1 AND status='active'",params![id,now_ms()]).map_err(|_|"cancel workflow failed")?;
        if changed > 0 {
            append_audit(&tx, id, "workflow_cancelled", None, now_ms())?
        }
        tx.commit()
            .map_err(|_| "commit workflow cancellation failed")?;
        Ok(changed > 0)
    }
    fn get(&self, id: &str) -> Result<AccountWorkflow, String> {
        let guard = self.conn.lock().map_err(|_| "workflow state unavailable")?;
        read_workflow(&guard, id)
    }
    fn snapshot(&self) -> Result<WorkflowSnapshot, String> {
        let guard = self.conn.lock().map_err(|_| "workflow state unavailable")?;
        let ids = {
            let mut q = guard
                .prepare("SELECT id FROM account_workflows ORDER BY created_at DESC")
                .map_err(|_| "read workflows failed")?;
            q.query_map([], |r| r.get::<_, String>(0))
                .map_err(|_| "read workflows failed")?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| "read workflows failed")?
        };
        let workflows = ids
            .iter()
            .map(|id| read_workflow(&guard, id))
            .collect::<Result<Vec<_>, _>>()?;
        let audit = {
            let mut q=guard.prepare("SELECT id,workflow_id,event_type,step_key,occurred_at FROM workflow_audit ORDER BY occurred_at,id").map_err(|_|"read workflow audit failed")?;
            q.query_map([], |r| {
                Ok(WorkflowAuditEvent {
                    id: r.get(0)?,
                    workflow_id: r.get(1)?,
                    event_type: r.get(2)?,
                    step_key: r.get(3)?,
                    actor: "owner",
                    occurred_at_unix_ms: r.get(4)?,
                })
            })
            .map_err(|_| "read workflow audit failed")?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "read workflow audit failed")?
        };
        Ok(WorkflowSnapshot {
            workflows,
            audit,
            automation: "manual_owner_confirmed_only",
            secret_input_supported: false,
        })
    }
}
fn read_workflow(conn: &Connection, id: &str) -> Result<AccountWorkflow, String> {
    let base:Option<(String,String,String,String,i64,i64)>=conn.query_row("SELECT identity_id,provider,kind,status,created_at,updated_at FROM account_workflows WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).optional().map_err(|_|"read workflow failed")?;
    let Some((identity_id, provider, kind, status, created, updated)) = base else {
        return Err("workflow not found".into());
    };
    let mut q=conn.prepare("SELECT id,ordinal,key,title,instruction,status,confirmed_at FROM workflow_steps WHERE workflow_id=?1 ORDER BY ordinal").map_err(|_|"read workflow steps failed")?;
    let steps = q
        .query_map([id], |r| {
            Ok(WorkflowStep {
                id: r.get(0)?,
                workflow_id: id.into(),
                ordinal: r.get(1)?,
                key: r.get(2)?,
                title: r.get(3)?,
                instruction: r.get(4)?,
                status: r.get(5)?,
                requires_owner_confirmation: true,
                automated: false,
                confirmed_at_unix_ms: r.get(6)?,
            })
        })
        .map_err(|_| "read workflow steps failed")?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "read workflow steps failed")?;
    Ok(AccountWorkflow {
        id: id.into(),
        identity_id,
        provider,
        kind: if kind == "recovery" {
            WorkflowKind::Recovery
        } else {
            WorkflowKind::Onboarding
        },
        status,
        created_at_unix_ms: created,
        updated_at_unix_ms: updated,
        steps,
    })
}
fn template(
    provider: &str,
    kind: &WorkflowKind,
) -> Result<Vec<(&'static str, &'static str, &'static str)>, String> {
    let normalized = provider.trim().to_ascii_lowercase();
    if !["github", "openai", "google", "microsoft"].contains(&normalized.as_str()) {
        return Err("provider-specific workflow is unavailable".into());
    }
    let steps = match kind {
        WorkflowKind::Onboarding => vec![
            (
                "review_terms",
                "Review provider terms",
                "Owner must review and accept the current provider terms manually.",
            ),
            (
                "confirm_account_control",
                "Confirm account control",
                "Owner confirms the registered account is under their control.",
            ),
            (
                "verify_contact",
                "Verify contact channel",
                "Owner completes the provider's official verification flow manually.",
            ),
            (
                "configure_mfa",
                "Configure MFA",
                "Owner configures MFA in the official provider interface.",
            ),
            (
                "confirm_recovery",
                "Confirm recovery control",
                "Owner confirms recovery channels without storing codes in SAIKI.",
            ),
        ],
        WorkflowKind::Recovery => vec![
            (
                "open_official_recovery",
                "Open official recovery",
                "Owner starts recovery using the provider's official interface.",
            ),
            (
                "confirm_recovery_channel",
                "Confirm recovery channel",
                "Owner confirms control of the provider-approved recovery channel.",
            ),
            (
                "rotate_credentials",
                "Rotate credentials manually",
                "Owner rotates credentials in the provider interface; no value is entered into SAIKI.",
            ),
            (
                "revalidate_mfa",
                "Revalidate MFA",
                "Owner verifies MFA and recovery posture after recovery.",
            ),
        ],
    };
    Ok(steps)
}
fn append_audit(
    tx: &rusqlite::Transaction<'_>,
    workflow: &str,
    event: &str,
    step: Option<&str>,
    now: i64,
) -> Result<(), String> {
    tx.execute(
        "INSERT INTO workflow_audit VALUES(?1,?2,?3,?4,'owner',?5)",
        params![Uuid::new_v4().to_string(), workflow, event, step, now],
    )
    .map_err(|_| "persist workflow audit failed")?;
    Ok(())
}
fn kind_name(kind: &WorkflowKind) -> &'static str {
    match kind {
        WorkflowKind::Onboarding => "onboarding",
        WorkflowKind::Recovery => "recovery",
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
pub fn get_account_workflows(
    state: State<'_, OnboardingState>,
) -> Result<WorkflowSnapshot, String> {
    state.snapshot()
}
#[tauri::command]
pub fn create_account_workflow(
    state: State<'_, OnboardingState>,
    identities: State<'_, IdentityState>,
    request: CreateWorkflowRequest,
) -> Result<AccountWorkflow, String> {
    state.create(&identities, request)
}
#[tauri::command]
pub fn confirm_account_workflow_step(
    state: State<'_, OnboardingState>,
    request: ConfirmStepRequest,
) -> Result<AccountWorkflow, String> {
    state.confirm(request)
}
#[tauri::command]
pub fn cancel_account_workflow(
    state: State<'_, OnboardingState>,
    id: String,
) -> Result<bool, String> {
    state.cancel(&id)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn states(provider: &str) -> (OnboardingState, IdentityState, String) {
        let (identity, id) = IdentityState::test_with_provider(provider);
        (
            OnboardingState::from_connection(Connection::open_in_memory().unwrap()).unwrap(),
            identity,
            id,
        )
    }
    #[test]
    fn onboarding_is_manual_and_terms_cannot_be_skipped() {
        let (s, i, id) = states("GitHub");
        let w = s
            .create(
                &i,
                CreateWorkflowRequest {
                    identity_id: id,
                    kind: WorkflowKind::Onboarding,
                },
            )
            .unwrap();
        assert_eq!(w.steps[0].key, "review_terms");
        assert!(
            w.steps
                .iter()
                .all(|step| !step.automated && step.requires_owner_confirmation)
        );
        assert!(
            s.confirm(ConfirmStepRequest {
                workflow_id: w.id.clone(),
                step_id: w.steps[1].id.clone(),
                owner_confirmed: true
            })
            .is_err()
        );
    }
    #[test]
    fn ownership_is_append_only_auditable() {
        let (s, i, id) = states("OpenAI");
        let w = s
            .create(
                &i,
                CreateWorkflowRequest {
                    identity_id: id,
                    kind: WorkflowKind::Onboarding,
                },
            )
            .unwrap();
        s.confirm(ConfirmStepRequest {
            workflow_id: w.id.clone(),
            step_id: w.steps[0].id.clone(),
            owner_confirmed: true,
        })
        .unwrap();
        let snapshot = s.snapshot().unwrap();
        assert_eq!(snapshot.audit[0].actor, "owner");
        assert_eq!(snapshot.audit[1].event_type, "owner_confirmed_step");
        assert_eq!(snapshot.audit[1].step_key.as_deref(), Some("review_terms"));
    }
    #[test]
    fn false_confirmation_is_rejected() {
        let (s, i, id) = states("Google");
        let w = s
            .create(
                &i,
                CreateWorkflowRequest {
                    identity_id: id,
                    kind: WorkflowKind::Recovery,
                },
            )
            .unwrap();
        assert!(
            s.confirm(ConfirmStepRequest {
                workflow_id: w.id,
                step_id: w.steps[0].id.clone(),
                owner_confirmed: false
            })
            .is_err()
        );
    }
    #[test]
    fn unsupported_provider_is_honestly_unavailable() {
        let (s, i, id) = states("UnknownProvider");
        assert!(
            s.create(
                &i,
                CreateWorkflowRequest {
                    identity_id: id,
                    kind: WorkflowKind::Onboarding
                }
            )
            .unwrap_err()
            .contains("unavailable")
        );
    }
}
