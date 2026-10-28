//! Deterministic, owner-controlled policy engine.
//!
//! This is the authority boundary: a model may *request* an action, but only
//! this engine decides whether it may run, and it is deny-by-default. The
//! model's only entry point is [`evaluate`]; every authority-granting
//! operation (create/revoke grant, approve, kill switch) is an owner command
//! with no model-reachable path, so model output can never grant, widen, or
//! approve authority.
//!
//! Decisions are `Allow`, `Deny`, or `ApprovalRequired`. An executor proceeds
//! only on `Allow`. Approvals bind an exact SHA-256 payload digest, so any
//! material change to the arguments invalidates a prior approval. Every
//! evaluation and owner action is written to an append-only audit log; there
//! is no command that edits or deletes audit rows.

use rusqlite::{Connection, OptionalExtension, params};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fmt::Write as _,
    path::Path,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::State;

const SCHEMA_VERSION: i64 = 1;
const AUDIT_LIMIT: usize = 200;
const MAX_TTL_MS: u64 = 365 * 24 * 60 * 60 * 1000;
const KILL_SWITCH_REASON: &str =
    "emergency kill switch is engaged; all external actions are denied";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RiskTier {
    Internal,
    Low,
    External,
    Financial,
    Critical,
}

/// Map a capability name to its risk tier. Unknown capabilities are treated as
/// `Critical` — the most restrictive — so an unrecognized model request is
/// never cheap to run.
fn tier_for(capability: &str) -> RiskTier {
    let cap = capability.trim().to_lowercase();
    const INTERNAL: [&str; 4] = ["reason", "plan", "internal.", "note"];
    const LOW: [&str; 4] = ["fs.read", "metrics.read", "inbox.read", "repo.read"];
    const EXTERNAL: [&str; 6] = [
        "fs.write",
        "net.publish",
        "email.send",
        "account.create",
        "content.upload",
        "message.send",
    ];
    const FINANCIAL: [&str; 4] = ["spend.", "payment.", "domain.buy", "refund."];
    let matches = |set: &[&str]| {
        set.iter()
            .any(|p| cap == *p || (p.ends_with('.') && cap.starts_with(*p)))
    };
    // Known critical capabilities (data.delete, mfa.*, secret.*, security.*) and
    // any unrecognized capability both fall through to the most restrictive tier.
    if matches(&INTERNAL) {
        RiskTier::Internal
    } else if matches(&LOW) {
        RiskTier::Low
    } else if matches(&EXTERNAL) {
        RiskTier::External
    } else if matches(&FINANCIAL) {
        RiskTier::Financial
    } else {
        RiskTier::Critical
    }
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(tag = "decision", rename_all = "snake_case")]
pub enum Decision {
    Allow,
    Deny { reason: String },
    ApprovalRequired { tier: RiskTier, reason: String },
}

impl Decision {
    fn label(&self) -> &'static str {
        match self {
            Decision::Allow => "allow",
            Decision::Deny { .. } => "deny",
            Decision::ApprovalRequired { .. } => "approval_required",
        }
    }
    fn reason(&self) -> String {
        match self {
            Decision::Allow => String::new(),
            Decision::Deny { reason } => reason.clone(),
            Decision::ApprovalRequired { reason, .. } => reason.clone(),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Grant {
    id: String,
    capability: String,
    resource: String,
    purpose: String,
    tier: RiskTier,
    max_actions: i64,
    used_actions: i64,
    max_spend: Option<f64>,
    spent: f64,
    expires_at_unix_ms: i64,
    created_at_unix_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalRequest {
    id: String,
    actor: String,
    capability: String,
    resource: String,
    arguments_digest: String,
    status: String,
    tier: RiskTier,
    requested_at_unix_ms: i64,
    expires_at_unix_ms: i64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AuditEntry {
    at_unix_ms: i64,
    actor: String,
    capability: String,
    resource: String,
    decision: String,
    reason: String,
    detail: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PolicyState {
    kill_switch_engaged: bool,
    grants: Vec<Grant>,
    pending_approvals: Vec<ApprovalRequest>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct EvaluateRequest {
    actor: String,
    capability: String,
    resource: String,
    #[serde(default)]
    arguments: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateGrantRequest {
    capability: String,
    resource: String,
    purpose: String,
    max_actions: i64,
    #[serde(default)]
    max_spend: Option<f64>,
    ttl_ms: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RequestApprovalRequest {
    actor: String,
    capability: String,
    resource: String,
    #[serde(default)]
    arguments: serde_json::Value,
    ttl_ms: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ResolveApprovalRequest {
    id: String,
    approve: bool,
}

pub struct PolicyEngine {
    conn: Mutex<Connection>,
}

impl PolicyEngine {
    pub fn load(data_dir: &Path) -> Result<Self, String> {
        std::fs::create_dir_all(data_dir).map_err(|_| "create policy directory failed")?;
        let connection = Connection::open(data_dir.join("policy.sqlite3"))
            .map_err(|_| "open policy database failed")?;
        Self::from_connection(connection)
    }

    fn from_connection(connection: Connection) -> Result<Self, String> {
        connection
            .pragma_update(None, "journal_mode", "WAL")
            .map_err(|_| "configure policy database failed")?;
        migrate(&connection)?;
        Ok(Self {
            conn: Mutex::new(connection),
        })
    }

    /// In-process authorization entry point for other Rust modules (e.g. the
    /// developer-tools shell). Equivalent to the `evaluate_action` command; the
    /// caller must act only on `Decision::Allow`.
    pub fn authorize(
        &self,
        actor: &str,
        capability: &str,
        resource: &str,
        arguments: &serde_json::Value,
    ) -> Result<Decision, String> {
        self.evaluate(&EvaluateRequest {
            actor: actor.to_owned(),
            capability: capability.to_owned(),
            resource: resource.to_owned(),
            arguments: arguments.clone(),
        })
    }

    fn evaluate(&self, request: &EvaluateRequest) -> Result<Decision, String> {
        validate_name(&request.capability, 100, "capability")?;
        validate_name(&request.resource, 200, "resource")?;
        validate_name(&request.actor, 100, "actor")?;
        let amount = request
            .arguments
            .get("amount")
            .and_then(|value| value.as_f64())
            .unwrap_or(0.0);
        let digest = digest_arguments(&request.arguments);
        let now = now_unix_ms();
        let mut guard = self.conn.lock().map_err(|_| "policy state unavailable")?;

        if kill_switch_engaged(&guard)? {
            let decision = Decision::Deny {
                reason: KILL_SWITCH_REASON.into(),
            };
            audit(
                &guard,
                &request.actor,
                &request.capability,
                &request.resource,
                &decision,
                "kill_switch",
            )?;
            return Ok(decision);
        }

        let tier = tier_for(&request.capability);
        if tier == RiskTier::Internal {
            let decision = Decision::Allow;
            audit(
                &guard,
                &request.actor,
                &request.capability,
                &request.resource,
                &decision,
                "internal_tier",
            )?;
            return Ok(decision);
        }

        let tx = guard
            .transaction()
            .map_err(|_| "begin policy evaluation failed")?;
        let mut statement = tx
            .prepare(
                "SELECT id, resource, max_actions, used_actions, max_spend, spent FROM grants \
                 WHERE capability = ?1 AND status = 'active' AND expires_at_unix_ms > ?2",
            )
            .map_err(|_| "read grants failed")?;
        let candidates = statement
            .query_map(params![request.capability, now], |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, i64>(3)?,
                    row.get::<_, Option<f64>>(4)?,
                    row.get::<_, f64>(5)?,
                ))
            })
            .map_err(|_| "read grants failed")?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "read grants failed")?;
        drop(statement);

        let mut limited = false;
        let mut authorizing: Option<String> = None;
        for (id, resource, max_actions, used, max_spend, spent) in &candidates {
            if !resource_matches(resource, &request.resource) {
                continue;
            }
            if *used >= *max_actions {
                limited = true;
                continue;
            }
            if let Some(limit) = max_spend {
                if spent + amount > *limit + f64::EPSILON {
                    limited = true;
                    continue;
                }
            }
            authorizing = Some(id.clone());
            break;
        }

        if let Some(id) = authorizing {
            tx.execute(
                "UPDATE grants SET used_actions = used_actions + 1, spent = spent + ?2 WHERE id = ?1",
                params![id, amount],
            )
            .map_err(|_| "consume grant failed")?;
            let decision = Decision::Allow;
            audit(
                &tx,
                &request.actor,
                &request.capability,
                &request.resource,
                &decision,
                &format!("grant {id}"),
            )?;
            tx.commit().map_err(|_| "commit policy evaluation failed")?;
            return Ok(decision);
        }

        let approved: Option<String> = tx
            .query_row(
                "SELECT id FROM approvals WHERE capability = ?1 AND resource = ?2 AND arguments_digest = ?3 \
                 AND status = 'approved' AND expires_at_unix_ms > ?4 LIMIT 1",
                params![request.capability, request.resource, digest, now],
                |row| row.get(0),
            )
            .optional()
            .map_err(|_| "read approvals failed")?;

        let (decision, detail) = if let Some(id) = approved {
            (Decision::Allow, format!("approval {id}"))
        } else if limited {
            (
                Decision::Deny {
                    reason: "a matching grant is exhausted or over its budget".into(),
                },
                "grant_limited".to_owned(),
            )
        } else {
            (
                Decision::ApprovalRequired {
                    tier,
                    reason: "owner authorization is required for this action".into(),
                },
                "no_authority".to_owned(),
            )
        };
        audit(
            &tx,
            &request.actor,
            &request.capability,
            &request.resource,
            &decision,
            &detail,
        )?;
        tx.commit().map_err(|_| "commit policy evaluation failed")?;
        Ok(decision)
    }

    fn create_grant(&self, request: &CreateGrantRequest) -> Result<Grant, String> {
        validate_name(&request.capability, 100, "capability")?;
        validate_name(&request.resource, 200, "resource")?;
        if request.purpose.trim().is_empty() || request.purpose.len() > 500 {
            return Err("purpose must be 1-500 characters".into());
        }
        if !(1..=100_000).contains(&request.max_actions) {
            return Err("maxActions must be between 1 and 100000".into());
        }
        if let Some(spend) = request.max_spend {
            if !(0.0..=1_000_000.0).contains(&spend) {
                return Err("maxSpend must be between 0 and 1000000".into());
            }
        }
        if !(1_000..=MAX_TTL_MS).contains(&request.ttl_ms) {
            return Err("ttlMs is out of range".into());
        }
        let now = now_unix_ms();
        let expires = now + request.ttl_ms as i64;
        let id = uuid::Uuid::new_v4().to_string();
        let guard = self.conn.lock().map_err(|_| "policy state unavailable")?;
        guard
            .execute(
                "INSERT INTO grants (id, capability, resource, purpose, max_actions, used_actions, max_spend, spent, expires_at_unix_ms, status, created_at_unix_ms) \
                 VALUES (?1, ?2, ?3, ?4, ?5, 0, ?6, 0, ?7, 'active', ?8)",
                params![id, request.capability.trim(), request.resource.trim(), request.purpose.trim(), request.max_actions, request.max_spend, expires, now],
            )
            .map_err(|_| "create grant failed")?;
        audit(
            &guard,
            "owner",
            request.capability.trim(),
            request.resource.trim(),
            &Decision::Allow,
            &format!("grant_created {id}"),
        )?;
        read_grant(&guard, &id)
    }

    fn revoke_grant(&self, id: &str) -> Result<bool, String> {
        let guard = self.conn.lock().map_err(|_| "policy state unavailable")?;
        let affected = guard
            .execute(
                "UPDATE grants SET status = 'revoked' WHERE id = ?1 AND status = 'active'",
                params![id],
            )
            .map_err(|_| "revoke grant failed")?;
        if affected > 0 {
            audit(
                &guard,
                "owner",
                "-",
                "-",
                &Decision::Deny {
                    reason: "grant revoked".into(),
                },
                &format!("grant_revoked {id}"),
            )?;
        }
        Ok(affected > 0)
    }

    fn request_approval(
        &self,
        request: &RequestApprovalRequest,
    ) -> Result<ApprovalRequest, String> {
        validate_name(&request.capability, 100, "capability")?;
        validate_name(&request.resource, 200, "resource")?;
        validate_name(&request.actor, 100, "actor")?;
        if !(1_000..=MAX_TTL_MS).contains(&request.ttl_ms) {
            return Err("ttlMs is out of range".into());
        }
        let now = now_unix_ms();
        let expires = now + request.ttl_ms as i64;
        let id = uuid::Uuid::new_v4().to_string();
        let digest = digest_arguments(&request.arguments);
        let tier = tier_for(&request.capability);
        let guard = self.conn.lock().map_err(|_| "policy state unavailable")?;
        guard
            .execute(
                "INSERT INTO approvals (id, actor, capability, resource, arguments_digest, status, tier, requested_at_unix_ms, decided_at_unix_ms, expires_at_unix_ms) \
                 VALUES (?1, ?2, ?3, ?4, ?5, 'pending', ?6, ?7, NULL, ?8)",
                params![id, request.actor.trim(), request.capability.trim(), request.resource.trim(), digest, tier_label(tier), now, expires],
            )
            .map_err(|_| "create approval request failed")?;
        read_approval(&guard, &id)
    }

    fn resolve_approval(
        &self,
        request: &ResolveApprovalRequest,
    ) -> Result<ApprovalRequest, String> {
        let now = now_unix_ms();
        let status = if request.approve {
            "approved"
        } else {
            "denied"
        };
        let guard = self.conn.lock().map_err(|_| "policy state unavailable")?;
        let affected = guard
            .execute(
                "UPDATE approvals SET status = ?2, decided_at_unix_ms = ?3 WHERE id = ?1 AND status = 'pending'",
                params![request.id, status, now],
            )
            .map_err(|_| "resolve approval failed")?;
        if affected == 0 {
            return Err("no pending approval with that id".into());
        }
        audit(
            &guard,
            "owner",
            "-",
            "-",
            &Decision::Deny {
                reason: format!("approval {status}"),
            },
            &format!("approval_{status} {}", request.id),
        )?;
        read_approval(&guard, &request.id)
    }

    fn set_kill_switch(&self, engaged: bool) -> Result<PolicyState, String> {
        let guard = self.conn.lock().map_err(|_| "policy state unavailable")?;
        guard
            .execute(
                "INSERT INTO policy_flags (key, value) VALUES ('kill_switch', ?1) \
                 ON CONFLICT(key) DO UPDATE SET value = ?1",
                params![if engaged { "1" } else { "0" }],
            )
            .map_err(|_| "set kill switch failed")?;
        audit(
            &guard,
            "owner",
            "-",
            "-",
            &Decision::Deny {
                reason: "kill switch toggled".into(),
            },
            if engaged {
                "kill_switch_engaged"
            } else {
                "kill_switch_released"
            },
        )?;
        build_state(&guard)
    }

    fn state(&self) -> Result<PolicyState, String> {
        let guard = self.conn.lock().map_err(|_| "policy state unavailable")?;
        build_state(&guard)
    }

    fn audit_log(&self) -> Result<Vec<AuditEntry>, String> {
        let guard = self.conn.lock().map_err(|_| "policy state unavailable")?;
        let mut statement = guard
            .prepare(
                "SELECT at_unix_ms, actor, capability, resource, decision, reason, detail FROM audit \
                 ORDER BY id DESC LIMIT ?1",
            )
            .map_err(|_| "prepare audit read failed")?;
        let rows = statement
            .query_map(params![AUDIT_LIMIT as i64], |row| {
                Ok(AuditEntry {
                    at_unix_ms: row.get(0)?,
                    actor: row.get(1)?,
                    capability: row.get(2)?,
                    resource: row.get(3)?,
                    decision: row.get(4)?,
                    reason: row.get(5)?,
                    detail: row.get(6)?,
                })
            })
            .map_err(|_| "read audit failed")?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|_| "read audit failed".into())
    }
}

fn migrate(conn: &Connection) -> Result<(), String> {
    let version: i64 = conn
        .pragma_query_value(None, "user_version", |row| row.get(0))
        .map_err(|_| "read policy schema version failed")?;
    if version < 1 {
        conn.execute_batch(
            "CREATE TABLE grants (
                id TEXT PRIMARY KEY,
                capability TEXT NOT NULL,
                resource TEXT NOT NULL,
                purpose TEXT NOT NULL,
                max_actions INTEGER NOT NULL,
                used_actions INTEGER NOT NULL,
                max_spend REAL,
                spent REAL NOT NULL,
                expires_at_unix_ms INTEGER NOT NULL,
                status TEXT NOT NULL,
                created_at_unix_ms INTEGER NOT NULL
            );
            CREATE INDEX idx_grants_capability ON grants(capability, status);
            CREATE TABLE approvals (
                id TEXT PRIMARY KEY,
                actor TEXT NOT NULL,
                capability TEXT NOT NULL,
                resource TEXT NOT NULL,
                arguments_digest TEXT NOT NULL,
                status TEXT NOT NULL,
                tier TEXT NOT NULL,
                requested_at_unix_ms INTEGER NOT NULL,
                decided_at_unix_ms INTEGER,
                expires_at_unix_ms INTEGER NOT NULL
            );
            CREATE TABLE audit (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                at_unix_ms INTEGER NOT NULL,
                actor TEXT NOT NULL,
                capability TEXT NOT NULL,
                resource TEXT NOT NULL,
                decision TEXT NOT NULL,
                reason TEXT NOT NULL,
                detail TEXT NOT NULL
            );
            CREATE TABLE policy_flags (key TEXT PRIMARY KEY, value TEXT NOT NULL);",
        )
        .map_err(|_| "apply policy migration v1 failed")?;
        conn.pragma_update(None, "user_version", SCHEMA_VERSION)
            .map_err(|_| "record policy schema version failed")?;
    }
    Ok(())
}

fn build_state(conn: &Connection) -> Result<PolicyState, String> {
    let kill = kill_switch_engaged(conn)?;
    let now = now_unix_ms();
    let mut grant_stmt = conn
        .prepare(
            "SELECT id, capability, resource, purpose, max_actions, used_actions, max_spend, spent, expires_at_unix_ms, created_at_unix_ms \
             FROM grants WHERE status = 'active' AND expires_at_unix_ms > ?1 ORDER BY created_at_unix_ms DESC",
        )
        .map_err(|_| "read grants failed")?;
    let grants = grant_stmt
        .query_map(params![now], map_grant)
        .map_err(|_| "read grants failed")?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "read grants failed")?;
    drop(grant_stmt);

    let mut approval_stmt = conn
        .prepare(
            "SELECT id, actor, capability, resource, arguments_digest, status, tier, requested_at_unix_ms, expires_at_unix_ms \
             FROM approvals WHERE status = 'pending' AND expires_at_unix_ms > ?1 ORDER BY requested_at_unix_ms DESC",
        )
        .map_err(|_| "read approvals failed")?;
    let pending_approvals = approval_stmt
        .query_map(params![now], map_approval)
        .map_err(|_| "read approvals failed")?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "read approvals failed")?;
    Ok(PolicyState {
        kill_switch_engaged: kill,
        grants,
        pending_approvals,
    })
}

fn kill_switch_engaged(conn: &Connection) -> Result<bool, String> {
    let value: Option<String> = conn
        .query_row(
            "SELECT value FROM policy_flags WHERE key = 'kill_switch'",
            [],
            |row| row.get(0),
        )
        .optional()
        .map_err(|_| "read kill switch failed")?;
    Ok(value.as_deref() == Some("1"))
}

fn read_grant(conn: &Connection, id: &str) -> Result<Grant, String> {
    conn.query_row(
        "SELECT id, capability, resource, purpose, max_actions, used_actions, max_spend, spent, expires_at_unix_ms, created_at_unix_ms FROM grants WHERE id = ?1",
        params![id],
        map_grant,
    )
    .map_err(|_| "read grant failed".into())
}

fn read_approval(conn: &Connection, id: &str) -> Result<ApprovalRequest, String> {
    conn.query_row(
        "SELECT id, actor, capability, resource, arguments_digest, status, tier, requested_at_unix_ms, expires_at_unix_ms FROM approvals WHERE id = ?1",
        params![id],
        map_approval,
    )
    .map_err(|_| "read approval failed".into())
}

fn map_grant(row: &rusqlite::Row<'_>) -> rusqlite::Result<Grant> {
    let capability: String = row.get(1)?;
    Ok(Grant {
        tier: tier_for(&capability),
        id: row.get(0)?,
        capability,
        resource: row.get(2)?,
        purpose: row.get(3)?,
        max_actions: row.get(4)?,
        used_actions: row.get(5)?,
        max_spend: row.get(6)?,
        spent: row.get(7)?,
        expires_at_unix_ms: row.get(8)?,
        created_at_unix_ms: row.get(9)?,
    })
}

fn map_approval(row: &rusqlite::Row<'_>) -> rusqlite::Result<ApprovalRequest> {
    let tier_raw: String = row.get(6)?;
    Ok(ApprovalRequest {
        id: row.get(0)?,
        actor: row.get(1)?,
        capability: row.get(2)?,
        resource: row.get(3)?,
        arguments_digest: row.get(4)?,
        status: row.get(5)?,
        tier: tier_from_label(&tier_raw),
        requested_at_unix_ms: row.get(7)?,
        expires_at_unix_ms: row.get(8)?,
    })
}

fn audit(
    conn: &Connection,
    actor: &str,
    capability: &str,
    resource: &str,
    decision: &Decision,
    detail: &str,
) -> Result<(), String> {
    conn.execute(
        "INSERT INTO audit (at_unix_ms, actor, capability, resource, decision, reason, detail) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![now_unix_ms(), actor, capability, resource, decision.label(), decision.reason(), detail],
    )
    .map_err(|_| "write audit entry failed")?;
    Ok(())
}

fn digest_arguments(arguments: &serde_json::Value) -> String {
    // serde_json's default Map is key-sorted, so this serialization is
    // deterministic for a given logical payload.
    let bytes = serde_json::to_vec(arguments).unwrap_or_default();
    let mut hasher = Sha256::new();
    hasher.update(&bytes);
    let mut hex = String::with_capacity(64);
    for byte in hasher.finalize() {
        let _ = write!(hex, "{byte:02x}");
    }
    hex
}

fn resource_matches(pattern: &str, resource: &str) -> bool {
    match pattern.strip_suffix('*') {
        Some(prefix) => resource.starts_with(prefix),
        None => pattern == resource,
    }
}

fn validate_name(value: &str, max: usize, field: &str) -> Result<(), String> {
    if value.trim().is_empty() || value.len() > max {
        return Err(format!("{field} must be 1-{max} characters"));
    }
    Ok(())
}

fn tier_label(tier: RiskTier) -> &'static str {
    match tier {
        RiskTier::Internal => "internal",
        RiskTier::Low => "low",
        RiskTier::External => "external",
        RiskTier::Financial => "financial",
        RiskTier::Critical => "critical",
    }
}

fn tier_from_label(label: &str) -> RiskTier {
    match label {
        "internal" => RiskTier::Internal,
        "low" => RiskTier::Low,
        "external" => RiskTier::External,
        "financial" => RiskTier::Financial,
        _ => RiskTier::Critical,
    }
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
pub fn get_policy_state(state: State<'_, PolicyEngine>) -> Result<PolicyState, String> {
    state.state()
}

#[tauri::command]
pub fn evaluate_action(
    state: State<'_, PolicyEngine>,
    request: EvaluateRequest,
) -> Result<Decision, String> {
    state.evaluate(&request)
}

#[tauri::command]
pub fn create_grant(
    state: State<'_, PolicyEngine>,
    request: CreateGrantRequest,
) -> Result<Grant, String> {
    state.create_grant(&request)
}

#[tauri::command]
pub fn revoke_grant(state: State<'_, PolicyEngine>, id: String) -> Result<bool, String> {
    state.revoke_grant(&id)
}

#[tauri::command]
pub fn request_approval(
    state: State<'_, PolicyEngine>,
    request: RequestApprovalRequest,
) -> Result<ApprovalRequest, String> {
    state.request_approval(&request)
}

#[tauri::command]
pub fn resolve_approval(
    state: State<'_, PolicyEngine>,
    request: ResolveApprovalRequest,
) -> Result<ApprovalRequest, String> {
    state.resolve_approval(&request)
}

#[tauri::command]
pub fn set_kill_switch(
    state: State<'_, PolicyEngine>,
    engaged: bool,
) -> Result<PolicyState, String> {
    state.set_kill_switch(engaged)
}

#[tauri::command]
pub fn list_policy_audit(state: State<'_, PolicyEngine>) -> Result<Vec<AuditEntry>, String> {
    state.audit_log()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn engine() -> PolicyEngine {
        PolicyEngine::from_connection(Connection::open_in_memory().unwrap()).unwrap()
    }

    fn evaluate(
        engine: &PolicyEngine,
        actor: &str,
        capability: &str,
        resource: &str,
        arguments: serde_json::Value,
    ) -> Decision {
        engine
            .evaluate(&EvaluateRequest {
                actor: actor.into(),
                capability: capability.into(),
                resource: resource.into(),
                arguments,
            })
            .unwrap()
    }

    #[test]
    fn model_proposed_external_call_is_not_allowed_by_default() {
        let engine = engine();
        // A model proposal for a C-tier capability with no grant must not run.
        let decision = evaluate(
            &engine,
            "model",
            "email.send",
            "mailbox/outreach",
            serde_json::json!({}),
        );
        assert!(matches!(decision, Decision::ApprovalRequired { .. }));
        assert_ne!(decision, Decision::Allow);
    }

    #[test]
    fn internal_tier_is_auto_allowed() {
        let engine = engine();
        assert_eq!(
            evaluate(
                &engine,
                "model",
                "reason",
                "scratchpad",
                serde_json::json!({})
            ),
            Decision::Allow
        );
    }

    #[test]
    fn grant_authorizes_then_revocation_denies() {
        let engine = engine();
        let grant = engine
            .create_grant(&CreateGrantRequest {
                capability: "email.send".into(),
                resource: "mailbox/*".into(),
                purpose: "approved outreach".into(),
                max_actions: 5,
                max_spend: None,
                ttl_ms: 60_000,
            })
            .unwrap();
        assert_eq!(
            evaluate(
                &engine,
                "model",
                "email.send",
                "mailbox/outreach",
                serde_json::json!({})
            ),
            Decision::Allow
        );
        assert!(engine.revoke_grant(&grant.id).unwrap());
        assert!(matches!(
            evaluate(
                &engine,
                "model",
                "email.send",
                "mailbox/outreach",
                serde_json::json!({})
            ),
            Decision::ApprovalRequired { .. }
        ));
    }

    #[test]
    fn grant_action_limit_is_enforced() {
        let engine = engine();
        engine
            .create_grant(&CreateGrantRequest {
                capability: "fs.write".into(),
                resource: "workspace/notes.txt".into(),
                purpose: "edit notes".into(),
                max_actions: 1,
                max_spend: None,
                ttl_ms: 60_000,
            })
            .unwrap();
        assert_eq!(
            evaluate(
                &engine,
                "model",
                "fs.write",
                "workspace/notes.txt",
                serde_json::json!({})
            ),
            Decision::Allow
        );
        assert!(matches!(
            evaluate(
                &engine,
                "model",
                "fs.write",
                "workspace/notes.txt",
                serde_json::json!({})
            ),
            Decision::Deny { .. }
        ));
    }

    #[test]
    fn kill_switch_denies_even_with_a_grant() {
        let engine = engine();
        engine
            .create_grant(&CreateGrantRequest {
                capability: "email.send".into(),
                resource: "mailbox/*".into(),
                purpose: "outreach".into(),
                max_actions: 5,
                max_spend: None,
                ttl_ms: 60_000,
            })
            .unwrap();
        engine.set_kill_switch(true).unwrap();
        assert!(matches!(
            evaluate(
                &engine,
                "model",
                "email.send",
                "mailbox/outreach",
                serde_json::json!({})
            ),
            Decision::Deny { .. }
        ));
    }

    #[test]
    fn approval_binds_exact_payload_digest() {
        let engine = engine();
        let approval = engine
            .request_approval(&RequestApprovalRequest {
                actor: "model".into(),
                capability: "spend.purchase".into(),
                resource: "vendor/domains".into(),
                arguments: serde_json::json!({ "amount": 12.0, "item": "domain" }),
                ttl_ms: 60_000,
            })
            .unwrap();
        engine
            .resolve_approval(&ResolveApprovalRequest {
                id: approval.id,
                approve: true,
            })
            .unwrap();
        // Exact arguments → allowed.
        assert_eq!(
            evaluate(
                &engine,
                "model",
                "spend.purchase",
                "vendor/domains",
                serde_json::json!({ "amount": 12.0, "item": "domain" })
            ),
            Decision::Allow
        );
        // Materially changed arguments (different amount) → not covered.
        assert!(matches!(
            evaluate(
                &engine,
                "model",
                "spend.purchase",
                "vendor/domains",
                serde_json::json!({ "amount": 99.0, "item": "domain" })
            ),
            Decision::ApprovalRequired { .. }
        ));
    }

    #[test]
    fn unknown_capability_is_critical_and_requires_approval() {
        let engine = engine();
        assert_eq!(tier_for("mystery.action"), RiskTier::Critical);
        assert!(matches!(
            evaluate(
                &engine,
                "model",
                "mystery.action",
                "somewhere",
                serde_json::json!({})
            ),
            Decision::ApprovalRequired {
                tier: RiskTier::Critical,
                ..
            }
        ));
    }

    #[test]
    fn spend_limit_is_enforced() {
        let engine = engine();
        engine
            .create_grant(&CreateGrantRequest {
                capability: "spend.purchase".into(),
                resource: "vendor/*".into(),
                purpose: "small purchases".into(),
                max_actions: 10,
                max_spend: Some(50.0),
                ttl_ms: 60_000,
            })
            .unwrap();
        assert_eq!(
            evaluate(
                &engine,
                "model",
                "spend.purchase",
                "vendor/a",
                serde_json::json!({ "amount": 40.0 })
            ),
            Decision::Allow
        );
        // Second spend would exceed the budget.
        assert!(matches!(
            evaluate(
                &engine,
                "model",
                "spend.purchase",
                "vendor/a",
                serde_json::json!({ "amount": 40.0 })
            ),
            Decision::Deny { .. }
        ));
    }

    #[test]
    fn grants_and_kill_switch_persist_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        {
            let engine = PolicyEngine::load(dir.path()).unwrap();
            engine
                .create_grant(&CreateGrantRequest {
                    capability: "email.send".into(),
                    resource: "mailbox/*".into(),
                    purpose: "outreach".into(),
                    max_actions: 5,
                    max_spend: None,
                    ttl_ms: 600_000,
                })
                .unwrap();
            engine.set_kill_switch(true).unwrap();
        }
        let reopened = PolicyEngine::load(dir.path()).unwrap();
        let state = reopened.state().unwrap();
        assert!(state.kill_switch_engaged);
        assert_eq!(state.grants.len(), 1);
        // Release the kill switch; the persisted grant then authorizes.
        reopened.set_kill_switch(false).unwrap();
        assert_eq!(
            evaluate(
                &reopened,
                "model",
                "email.send",
                "mailbox/x",
                serde_json::json!({})
            ),
            Decision::Allow
        );
    }

    #[test]
    fn evaluations_are_audited() {
        let engine = engine();
        evaluate(
            &engine,
            "model",
            "email.send",
            "mailbox/x",
            serde_json::json!({}),
        );
        let audit = engine.audit_log().unwrap();
        assert!(
            audit
                .iter()
                .any(|entry| entry.capability == "email.send"
                    && entry.decision == "approval_required")
        );
    }
}
