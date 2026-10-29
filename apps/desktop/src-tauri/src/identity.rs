//! Owner-controlled identity inventory. Metadata only: no credential input,
//! storage, logging, memory export, or model-context integration exists here.

use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::{
    path::Path,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::State;
use uuid::Uuid;

const MAX_TEXT: usize = 512;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OwnerProofMethod {
    OwnerAttestation,
    ProviderSession,
    RecoveryControl,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MfaMethod {
    Authenticator,
    SecurityKey,
    ProviderManaged,
    Sms,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentityRecord {
    id: String,
    provider: String,
    account_label: String,
    login_hint: String,
    owner_proof_method: OwnerProofMethod,
    owner_verified_at_unix_ms: i64,
    vault_reference: String,
    vault_binding: &'static str,
    credential_present: bool,
    credential_reason: &'static str,
    mfa_enabled: bool,
    mfa_methods: Vec<MfaMethod>,
    recovery_control_confirmed: bool,
    status: String,
    created_at_unix_ms: i64,
    updated_at_unix_ms: i64,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RegisterIdentityRequest {
    provider: String,
    account_label: String,
    login_hint: String,
    owner_proof_method: OwnerProofMethod,
    mfa_enabled: bool,
    #[serde(default)]
    mfa_methods: Vec<MfaMethod>,
    recovery_control_confirmed: bool,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UpdateMfaRequest {
    id: String,
    mfa_enabled: bool,
    #[serde(default)]
    mfa_methods: Vec<MfaMethod>,
    recovery_control_confirmed: bool,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct IdentityRegistryStatus {
    identities: Vec<IdentityRecord>,
    secret_input_supported: bool,
    vault_backend: &'static str,
    credential_provisioning: &'static str,
}

pub struct IdentityState {
    conn: Mutex<Connection>,
}
impl IdentityState {
    pub fn active_provider(&self, id: &str) -> Result<String, String> {
        self.conn
            .lock()
            .map_err(|_| "identity state unavailable")?
            .query_row(
                "SELECT provider FROM identities WHERE id=?1 AND status='active'",
                [id],
                |r| r.get(0),
            )
            .map_err(|_| "active identity not found".into())
    }
    #[cfg(test)]
    pub(crate) fn test_with_provider(provider: &str) -> (Self, String) {
        let state = Self::from_connection(Connection::open_in_memory().unwrap()).unwrap();
        let record = state
            .register(RegisterIdentityRequest {
                provider: provider.into(),
                account_label: "test".into(),
                login_hint: "owner".into(),
                owner_proof_method: OwnerProofMethod::OwnerAttestation,
                mfa_enabled: false,
                mfa_methods: vec![],
                recovery_control_confirmed: true,
            })
            .unwrap();
        (state, record.id)
    }
    pub fn load(config_dir: &Path) -> Result<Self, String> {
        std::fs::create_dir_all(config_dir).map_err(|_| "create identity directory failed")?;
        Self::from_connection(
            Connection::open(config_dir.join("identities.sqlite3"))
                .map_err(|_| "open identity database failed")?,
        )
    }
    fn from_connection(conn: Connection) -> Result<Self, String> {
        conn.execute_batch("PRAGMA journal_mode=WAL; CREATE TABLE IF NOT EXISTS identities(id TEXT PRIMARY KEY,provider TEXT NOT NULL,account_label TEXT NOT NULL,login_hint TEXT NOT NULL,owner_proof_method TEXT NOT NULL,owner_verified_at INTEGER NOT NULL,vault_reference TEXT NOT NULL UNIQUE,mfa_enabled INTEGER NOT NULL,mfa_methods_json TEXT NOT NULL,recovery_control_confirmed INTEGER NOT NULL,status TEXT NOT NULL,created_at INTEGER NOT NULL,updated_at INTEGER NOT NULL);").map_err(|_|"migrate identity database failed")?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
    fn register(&self, r: RegisterIdentityRequest) -> Result<IdentityRecord, String> {
        validate(&r.provider, "provider")?;
        validate(&r.account_label, "account label")?;
        validate(&r.login_hint, "login hint")?;
        validate_mfa(r.mfa_enabled, &r.mfa_methods)?;
        let id = Uuid::new_v4().to_string();
        let now = now_ms();
        let vault_reference = format!("wincred:saiki.identity.{id}");
        let methods =
            serde_json::to_string(&r.mfa_methods).map_err(|_| "serialize MFA metadata failed")?;
        self.conn
            .lock()
            .map_err(|_| "identity state unavailable")?
            .execute(
                "INSERT INTO identities VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9,?10,'active',?6,?6)",
                params![
                    id,
                    r.provider,
                    r.account_label,
                    r.login_hint,
                    proof_name(&r.owner_proof_method),
                    now,
                    vault_reference,
                    r.mfa_enabled,
                    methods,
                    r.recovery_control_confirmed
                ],
            )
            .map_err(|_| "persist identity metadata failed")?;
        self.get(&id)
    }
    fn update_mfa(&self, r: UpdateMfaRequest) -> Result<IdentityRecord, String> {
        validate_mfa(r.mfa_enabled, &r.mfa_methods)?;
        let methods =
            serde_json::to_string(&r.mfa_methods).map_err(|_| "serialize MFA metadata failed")?;
        let changed=self.conn.lock().map_err(|_|"identity state unavailable")?.execute("UPDATE identities SET mfa_enabled=?2,mfa_methods_json=?3,recovery_control_confirmed=?4,updated_at=?5 WHERE id=?1 AND status='active'",params![r.id,r.mfa_enabled,methods,r.recovery_control_confirmed,now_ms()]).map_err(|_|"update MFA metadata failed")?;
        if changed == 0 {
            return Err("active identity not found".into());
        }
        self.get(&r.id)
    }
    fn archive(&self, id: &str) -> Result<bool, String> {
        Ok(self.conn.lock().map_err(|_|"identity state unavailable")?.execute("UPDATE identities SET status='archived',updated_at=?2 WHERE id=?1 AND status='active'",params![id,now_ms()]).map_err(|_|"archive identity failed")?>0)
    }
    fn get(&self, id: &str) -> Result<IdentityRecord, String> {
        let guard = self.conn.lock().map_err(|_| "identity state unavailable")?;
        guard.query_row("SELECT id,provider,account_label,login_hint,owner_proof_method,owner_verified_at,vault_reference,mfa_enabled,mfa_methods_json,recovery_control_confirmed,status,created_at,updated_at FROM identities WHERE id=?1",[id],identity_row).map_err(|_|"identity not found".into())
    }
    fn status(&self) -> Result<IdentityRegistryStatus, String> {
        let guard = self.conn.lock().map_err(|_| "identity state unavailable")?;
        let mut q=guard.prepare("SELECT id,provider,account_label,login_hint,owner_proof_method,owner_verified_at,vault_reference,mfa_enabled,mfa_methods_json,recovery_control_confirmed,status,created_at,updated_at FROM identities ORDER BY created_at DESC").map_err(|_|"read identity registry failed")?;
        let identities = q
            .query_map([], identity_row)
            .map_err(|_| "read identity registry failed")?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "read identity registry failed")?;
        Ok(IdentityRegistryStatus {
            identities,
            secret_input_supported: false,
            vault_backend: "windows_credential_manager_reference",
            credential_provisioning: "unavailable",
        })
    }
}
fn identity_row(r: &rusqlite::Row<'_>) -> rusqlite::Result<IdentityRecord> {
    let proof: String = r.get(4)?;
    let methods: String = r.get(8)?;
    Ok(IdentityRecord {
        id: r.get(0)?,
        provider: r.get(1)?,
        account_label: r.get(2)?,
        login_hint: r.get(3)?,
        owner_proof_method: match proof.as_str() {
            "provider_session" => OwnerProofMethod::ProviderSession,
            "recovery_control" => OwnerProofMethod::RecoveryControl,
            _ => OwnerProofMethod::OwnerAttestation,
        },
        owner_verified_at_unix_ms: r.get(5)?,
        vault_reference: r.get(6)?,
        vault_binding: "reference_only",
        credential_present: false,
        credential_reason: "No credential provisioning flow exists in Phase 20.",
        mfa_enabled: r.get(7)?,
        mfa_methods: serde_json::from_str(&methods).unwrap_or_default(),
        recovery_control_confirmed: r.get(9)?,
        status: r.get(10)?,
        created_at_unix_ms: r.get(11)?,
        updated_at_unix_ms: r.get(12)?,
    })
}
fn proof_name(v: &OwnerProofMethod) -> &'static str {
    match v {
        OwnerProofMethod::OwnerAttestation => "owner_attestation",
        OwnerProofMethod::ProviderSession => "provider_session",
        OwnerProofMethod::RecoveryControl => "recovery_control",
    }
}
fn validate(v: &str, label: &str) -> Result<(), String> {
    if v.trim().is_empty() || v.len() > MAX_TEXT || v.chars().any(|c| c.is_control()) {
        Err(format!("{label} must be non-empty, bounded text"))
    } else {
        Ok(())
    }
}
fn validate_mfa(enabled: bool, methods: &[MfaMethod]) -> Result<(), String> {
    if methods.len() > 4 {
        return Err("too many MFA methods".into());
    }
    if enabled && methods.is_empty() {
        return Err("enabled MFA requires at least one method".into());
    }
    if !enabled && !methods.is_empty() {
        return Err("disabled MFA cannot declare methods".into());
    }
    Ok(())
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
pub fn get_identity_registry(
    state: State<'_, IdentityState>,
) -> Result<IdentityRegistryStatus, String> {
    state.status()
}
#[tauri::command]
pub fn register_identity(
    state: State<'_, IdentityState>,
    request: RegisterIdentityRequest,
) -> Result<IdentityRecord, String> {
    state.register(request)
}
#[tauri::command]
pub fn update_identity_mfa(
    state: State<'_, IdentityState>,
    request: UpdateMfaRequest,
) -> Result<IdentityRecord, String> {
    state.update_mfa(request)
}
#[tauri::command]
pub fn archive_identity(state: State<'_, IdentityState>, id: String) -> Result<bool, String> {
    state.archive(&id)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn state() -> IdentityState {
        IdentityState::from_connection(Connection::open_in_memory().unwrap()).unwrap()
    }
    #[test]
    fn credential_shaped_fields_are_rejected() {
        let payload = serde_json::json!({"provider":"GitHub","accountLabel":"work","loginHint":"owner@example.com","ownerProofMethod":"owner_attestation","mfaEnabled":false,"mfaMethods":[],"recoveryControlConfirmed":true,"password":"secret"});
        assert!(serde_json::from_value::<RegisterIdentityRequest>(payload).is_err());
    }
    #[test]
    fn serialized_registry_contains_no_credentials() {
        let s = state();
        s.register(RegisterIdentityRequest {
            provider: "GitHub".into(),
            account_label: "owner".into(),
            login_hint: "owner@example.com".into(),
            owner_proof_method: OwnerProofMethod::ProviderSession,
            mfa_enabled: true,
            mfa_methods: vec![MfaMethod::Authenticator],
            recovery_control_confirmed: true,
        })
        .unwrap();
        let json = serde_json::to_string(&s.status().unwrap()).unwrap();
        assert!(!json.contains("password"));
        assert!(!json.contains("token"));
        assert!(!json.contains("recoveryCode"));
        assert!(json.contains("credentialPresent\":false"));
    }
    #[test]
    fn vault_reference_is_generated_and_reference_only() {
        let s = state();
        let record = s
            .register(RegisterIdentityRequest {
                provider: "GitHub".into(),
                account_label: "owner".into(),
                login_hint: "owner".into(),
                owner_proof_method: OwnerProofMethod::OwnerAttestation,
                mfa_enabled: false,
                mfa_methods: vec![],
                recovery_control_confirmed: false,
            })
            .unwrap();
        assert!(
            record
                .vault_reference
                .starts_with("wincred:saiki.identity.")
        );
        assert!(!record.credential_present);
        assert_eq!(record.vault_binding, "reference_only");
    }
    #[test]
    fn mfa_metadata_is_consistent() {
        assert!(
            state()
                .register(RegisterIdentityRequest {
                    provider: "x".into(),
                    account_label: "x".into(),
                    login_hint: "x".into(),
                    owner_proof_method: OwnerProofMethod::OwnerAttestation,
                    mfa_enabled: true,
                    mfa_methods: vec![],
                    recovery_control_confirmed: false
                })
                .is_err()
        );
    }
}
