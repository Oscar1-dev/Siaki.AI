//! Evidence-bound business experiments. No financial execution or inferred revenue.

use rusqlite::{Connection, OptionalExtension, params};
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
pub enum ExperimentStage {
    Intake,
    Research,
    Validation,
    Pilot,
    Completed,
    Stopped,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KpiDirection {
    AtLeast,
    AtMost,
}
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum KpiKind {
    RevenueMinor,
    ConversionRate,
    Count,
    DurationMs,
    Custom,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceSource {
    OwnerMeasurement,
    ProviderReport,
    VerifiedToolResult,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ResearchNote {
    id: String,
    experiment_id: String,
    content: String,
    source_url: Option<String>,
    trust: String,
    created_at_unix_ms: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KpiObservation {
    id: String,
    kpi_id: String,
    value: f64,
    source: EvidenceSource,
    evidence_reference: String,
    observed_at_unix_ms: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct KpiDefinition {
    id: String,
    experiment_id: String,
    name: String,
    kind: KpiKind,
    direction: KpiDirection,
    target: f64,
    unit: String,
    observations: Vec<KpiObservation>,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExperimentEvaluation {
    id: String,
    experiment_id: String,
    stage: ExperimentStage,
    recommendation: String,
    reason: String,
    evidence_count: i64,
    observed_revenue_minor: Option<i64>,
    revenue_evidence: String,
    created_at_unix_ms: i64,
}
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BusinessExperiment {
    id: String,
    idea: String,
    hypothesis: String,
    stage: ExperimentStage,
    budget_currency: String,
    budget_limit_minor: i64,
    actual_spend_minor: Option<i64>,
    spend_tracking: &'static str,
    created_at_unix_ms: i64,
    updated_at_unix_ms: i64,
    notes: Vec<ResearchNote>,
    kpis: Vec<KpiDefinition>,
    evaluations: Vec<ExperimentEvaluation>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExperimentSnapshot {
    experiments: Vec<BusinessExperiment>,
    financial_execution: &'static str,
    revenue_policy: &'static str,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CreateExperimentRequest {
    idea: String,
    hypothesis: String,
    budget_currency: String,
    budget_limit_minor: i64,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AddResearchNoteRequest {
    experiment_id: String,
    content: String,
    #[serde(default)]
    source_url: Option<String>,
    trust: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AddKpiRequest {
    experiment_id: String,
    name: String,
    kind: KpiKind,
    direction: KpiDirection,
    target: f64,
    unit: String,
}
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RecordObservationRequest {
    kpi_id: String,
    value: f64,
    source: EvidenceSource,
    evidence_reference: String,
    observed_at_unix_ms: i64,
}

pub struct ExperimentState {
    conn: Mutex<Connection>,
}
impl ExperimentState {
    pub fn load(config_dir: &Path) -> Result<Self, String> {
        std::fs::create_dir_all(config_dir).map_err(|_| "create experiment directory failed")?;
        Self::from_connection(
            Connection::open(config_dir.join("experiments.sqlite3"))
                .map_err(|_| "open experiment database failed")?,
        )
    }
    fn from_connection(conn: Connection) -> Result<Self, String> {
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON; CREATE TABLE IF NOT EXISTS experiments(id TEXT PRIMARY KEY,idea TEXT NOT NULL,hypothesis TEXT NOT NULL,stage TEXT NOT NULL,budget_currency TEXT NOT NULL,budget_limit_minor INTEGER NOT NULL,created_at INTEGER NOT NULL,updated_at INTEGER NOT NULL); CREATE TABLE IF NOT EXISTS research_notes(id TEXT PRIMARY KEY,experiment_id TEXT NOT NULL REFERENCES experiments(id),content TEXT NOT NULL,source_url TEXT,trust TEXT NOT NULL,created_at INTEGER NOT NULL); CREATE TABLE IF NOT EXISTS kpis(id TEXT PRIMARY KEY,experiment_id TEXT NOT NULL REFERENCES experiments(id),name TEXT NOT NULL,kind TEXT NOT NULL,direction TEXT NOT NULL,target REAL NOT NULL,unit TEXT NOT NULL,created_at INTEGER NOT NULL); CREATE TABLE IF NOT EXISTS kpi_observations(id TEXT PRIMARY KEY,kpi_id TEXT NOT NULL REFERENCES kpis(id),value REAL NOT NULL,source TEXT NOT NULL,evidence_reference TEXT NOT NULL,observed_at INTEGER NOT NULL); CREATE TABLE IF NOT EXISTS experiment_evaluations(id TEXT PRIMARY KEY,experiment_id TEXT NOT NULL REFERENCES experiments(id),stage TEXT NOT NULL,recommendation TEXT NOT NULL,reason TEXT NOT NULL,evidence_count INTEGER NOT NULL,observed_revenue_minor INTEGER,revenue_evidence TEXT NOT NULL,created_at INTEGER NOT NULL);").map_err(|_|"migrate experiment database failed")?;
        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
    fn create(&self, r: CreateExperimentRequest) -> Result<BusinessExperiment, String> {
        validate(&r.idea, "idea", 4096)?;
        validate(&r.hypothesis, "hypothesis", 4096)?;
        if !["GBP", "USD", "EUR"].contains(&r.budget_currency.as_str()) {
            return Err("budget currency must be GBP, USD, or EUR".into());
        }
        if r.budget_limit_minor < 0 {
            return Err("budget limit cannot be negative".into());
        }
        let id = Uuid::new_v4().to_string();
        let now = now_ms();
        self.conn
            .lock()
            .map_err(|_| "experiment state unavailable")?
            .execute(
                "INSERT INTO experiments VALUES(?1,?2,?3,'intake',?4,?5,?6,?6)",
                params![
                    id,
                    r.idea,
                    r.hypothesis,
                    r.budget_currency,
                    r.budget_limit_minor,
                    now
                ],
            )
            .map_err(|_| "persist experiment failed")?;
        self.get(&id)
    }
    fn add_note(&self, r: AddResearchNoteRequest) -> Result<ResearchNote, String> {
        validate(&r.content, "research note", 16_384)?;
        if !["owner", "untrusted_web_content", "verified_source"].contains(&r.trust.as_str()) {
            return Err("invalid research trust classification".into());
        }
        if let Some(url) = &r.source_url {
            validate_http_url(url)?
        }
        ensure_active(&self.conn, &r.experiment_id)?;
        let note = ResearchNote {
            id: Uuid::new_v4().to_string(),
            experiment_id: r.experiment_id,
            content: r.content,
            source_url: r.source_url,
            trust: r.trust,
            created_at_unix_ms: now_ms(),
        };
        self.conn
            .lock()
            .map_err(|_| "experiment state unavailable")?
            .execute(
                "INSERT INTO research_notes VALUES(?1,?2,?3,?4,?5,?6)",
                params![
                    note.id,
                    note.experiment_id,
                    note.content,
                    note.source_url,
                    note.trust,
                    note.created_at_unix_ms
                ],
            )
            .map_err(|_| "persist research note failed")?;
        Ok(note)
    }
    fn add_kpi(&self, r: AddKpiRequest) -> Result<KpiDefinition, String> {
        validate(&r.name, "KPI name", 512)?;
        validate(&r.unit, "KPI unit", 64)?;
        if !r.target.is_finite() {
            return Err("KPI target must be finite".into());
        }
        ensure_active(&self.conn, &r.experiment_id)?;
        if r.kind == KpiKind::RevenueMinor && r.unit != "currency_minor" {
            return Err("revenue KPI unit must be currency_minor".into());
        }
        let kpi = KpiDefinition {
            id: Uuid::new_v4().to_string(),
            experiment_id: r.experiment_id,
            name: r.name,
            kind: r.kind,
            direction: r.direction,
            target: r.target,
            unit: r.unit,
            observations: vec![],
        };
        self.conn
            .lock()
            .map_err(|_| "experiment state unavailable")?
            .execute(
                "INSERT INTO kpis VALUES(?1,?2,?3,?4,?5,?6,?7,?8)",
                params![
                    kpi.id,
                    kpi.experiment_id,
                    kpi.name,
                    kind_name(&kpi.kind),
                    direction_name(&kpi.direction),
                    kpi.target,
                    kpi.unit,
                    now_ms()
                ],
            )
            .map_err(|_| "persist KPI failed")?;
        Ok(kpi)
    }
    fn observe(&self, r: RecordObservationRequest) -> Result<KpiObservation, String> {
        if !r.value.is_finite() {
            return Err("observation value must be finite".into());
        }
        validate(&r.evidence_reference, "evidence reference", 1024)?;
        if r.observed_at_unix_ms <= 0 || r.observed_at_unix_ms > now_ms() + 300_000 {
            return Err("observation timestamp is invalid".into());
        }
        let guard = self
            .conn
            .lock()
            .map_err(|_| "experiment state unavailable")?;
        let exists:bool=guard.query_row("SELECT EXISTS(SELECT 1 FROM kpis k JOIN experiments e ON e.id=k.experiment_id WHERE k.id=?1 AND e.stage NOT IN ('completed','stopped'))",[&r.kpi_id],|row|row.get(0)).map_err(|_|"read KPI failed")?;
        if !exists {
            return Err("active KPI not found".into());
        }
        let observation = KpiObservation {
            id: Uuid::new_v4().to_string(),
            kpi_id: r.kpi_id,
            value: r.value,
            source: r.source,
            evidence_reference: r.evidence_reference,
            observed_at_unix_ms: r.observed_at_unix_ms,
        };
        guard
            .execute(
                "INSERT INTO kpi_observations VALUES(?1,?2,?3,?4,?5,?6)",
                params![
                    observation.id,
                    observation.kpi_id,
                    observation.value,
                    source_name(&observation.source),
                    observation.evidence_reference,
                    observation.observed_at_unix_ms
                ],
            )
            .map_err(|_| "persist KPI observation failed")?;
        Ok(observation)
    }
    fn evaluate(&self, id: &str) -> Result<ExperimentEvaluation, String> {
        let experiment = self.get(id)?;
        if matches!(
            experiment.stage,
            ExperimentStage::Completed | ExperimentStage::Stopped
        ) {
            return Err("terminal experiment cannot be evaluated".into());
        }
        let latest: Vec<(&KpiDefinition, Option<&KpiObservation>)> = experiment
            .kpis
            .iter()
            .map(|k| (k, k.observations.last()))
            .collect();
        let evidence_count = experiment.notes.len() as i64
            + latest.iter().filter(|(_, o)| o.is_some()).count() as i64;
        let all_observed = !latest.is_empty() && latest.iter().all(|(_, o)| o.is_some());
        let all_met = all_observed
            && latest.iter().all(|(k, o)| {
                let value = o.unwrap().value;
                match k.direction {
                    KpiDirection::AtLeast => value >= k.target,
                    KpiDirection::AtMost => value <= k.target,
                }
            });
        let (recommendation, reason) = if experiment.notes.is_empty() || !all_observed {
            (
                "insufficient_evidence",
                "Research notes and an observation for every KPI are required before advancing.",
            )
        } else if all_met {
            (
                "advance",
                "All defined KPIs meet their targets using recorded evidence.",
            )
        } else {
            ("hold", "Recorded evidence does not meet every KPI target.")
        };
        let revenue = latest
            .iter()
            .find(|(k, _)| k.kind == KpiKind::RevenueMinor)
            .and_then(|(_, o)| *o);
        let observed_revenue_minor = revenue.map(|o| o.value.round() as i64);
        let revenue_evidence = revenue
            .map(|o| source_name(&o.source))
            .unwrap_or("none")
            .to_string();
        let evaluation = ExperimentEvaluation {
            id: Uuid::new_v4().to_string(),
            experiment_id: id.into(),
            stage: experiment.stage,
            recommendation: recommendation.into(),
            reason: reason.into(),
            evidence_count,
            observed_revenue_minor,
            revenue_evidence,
            created_at_unix_ms: now_ms(),
        };
        self.conn
            .lock()
            .map_err(|_| "experiment state unavailable")?
            .execute(
                "INSERT INTO experiment_evaluations VALUES(?1,?2,?3,?4,?5,?6,?7,?8,?9)",
                params![
                    evaluation.id,
                    evaluation.experiment_id,
                    stage_name(&evaluation.stage),
                    evaluation.recommendation,
                    evaluation.reason,
                    evaluation.evidence_count,
                    evaluation.observed_revenue_minor,
                    evaluation.revenue_evidence,
                    evaluation.created_at_unix_ms
                ],
            )
            .map_err(|_| "persist evaluation failed")?;
        Ok(evaluation)
    }
    fn advance(&self, id: &str) -> Result<BusinessExperiment, String> {
        let guard = self
            .conn
            .lock()
            .map_err(|_| "experiment state unavailable")?;
        let(stage,recommendation):(String,Option<String>)=guard.query_row("SELECT e.stage,(SELECT recommendation FROM experiment_evaluations v WHERE v.experiment_id=e.id ORDER BY v.created_at DESC,v.id DESC LIMIT 1) FROM experiments e WHERE e.id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?))).map_err(|_|"experiment not found")?;
        if recommendation.as_deref() != Some("advance") {
            return Err("latest stage-gate evaluation does not recommend advance".into());
        }
        let next = match stage.as_str() {
            "intake" => "research",
            "research" => "validation",
            "validation" => "pilot",
            "pilot" => "completed",
            _ => return Err("experiment cannot advance".into()),
        };
        guard
            .execute(
                "UPDATE experiments SET stage=?2,updated_at=?3 WHERE id=?1",
                params![id, next, now_ms()],
            )
            .map_err(|_| "advance experiment failed")?;
        drop(guard);
        self.get(id)
    }
    fn stop(&self, id: &str) -> Result<bool, String> {
        Ok(self.conn.lock().map_err(|_|"experiment state unavailable")?.execute("UPDATE experiments SET stage='stopped',updated_at=?2 WHERE id=?1 AND stage NOT IN ('completed','stopped')",params![id,now_ms()]).map_err(|_|"stop experiment failed")?>0)
    }
    fn get(&self, id: &str) -> Result<BusinessExperiment, String> {
        let guard = self
            .conn
            .lock()
            .map_err(|_| "experiment state unavailable")?;
        read_experiment(&guard, id)
    }
    fn snapshot(&self) -> Result<ExperimentSnapshot, String> {
        let guard = self
            .conn
            .lock()
            .map_err(|_| "experiment state unavailable")?;
        let ids = {
            let mut q = guard
                .prepare("SELECT id FROM experiments ORDER BY created_at DESC")
                .map_err(|_| "read experiments failed")?;
            q.query_map([], |r| r.get::<_, String>(0))
                .map_err(|_| "read experiments failed")?
                .collect::<Result<Vec<_>, _>>()
                .map_err(|_| "read experiments failed")?
        };
        let experiments = ids
            .iter()
            .map(|id| read_experiment(&guard, id))
            .collect::<Result<Vec<_>, _>>()?;
        Ok(ExperimentSnapshot {
            experiments,
            financial_execution: "unavailable",
            revenue_policy: "observed_evidence_only_never_inferred",
        })
    }
}
fn read_experiment(conn: &Connection, id: &str) -> Result<BusinessExperiment, String> {
    let base:Option<(String,String,String,String,i64,i64)>=conn.query_row("SELECT idea,hypothesis,stage,budget_currency,budget_limit_minor,created_at FROM experiments WHERE id=?1",[id],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?,r.get(4)?,r.get(5)?))).optional().map_err(|_|"read experiment failed")?;
    let Some((idea, hypothesis, stage, currency, budget, created)) = base else {
        return Err("experiment not found".into());
    };
    let updated = conn
        .query_row(
            "SELECT updated_at FROM experiments WHERE id=?1",
            [id],
            |r| r.get(0),
        )
        .map_err(|_| "read experiment failed")?;
    let notes = {
        let mut q=conn.prepare("SELECT id,content,source_url,trust,created_at FROM research_notes WHERE experiment_id=?1 ORDER BY created_at,id").map_err(|_|"read notes failed")?;
        q.query_map([id], |r| {
            Ok(ResearchNote {
                id: r.get(0)?,
                experiment_id: id.into(),
                content: r.get(1)?,
                source_url: r.get(2)?,
                trust: r.get(3)?,
                created_at_unix_ms: r.get(4)?,
            })
        })
        .map_err(|_| "read notes failed")?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "read notes failed")?
    };
    let kpis = {
        let mut q=conn.prepare("SELECT id,name,kind,direction,target,unit FROM kpis WHERE experiment_id=?1 ORDER BY created_at,id").map_err(|_|"read KPIs failed")?;
        let rows = q
            .query_map([id], |r| {
                Ok((
                    r.get::<_, String>(0)?,
                    r.get::<_, String>(1)?,
                    r.get::<_, String>(2)?,
                    r.get::<_, String>(3)?,
                    r.get::<_, f64>(4)?,
                    r.get::<_, String>(5)?,
                ))
            })
            .map_err(|_| "read KPIs failed")?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|_| "read KPIs failed")?;
        rows.into_iter().map(|(kid,name,kind,direction,target,unit)|{let mut oq=conn.prepare("SELECT id,value,source,evidence_reference,observed_at FROM kpi_observations WHERE kpi_id=?1 ORDER BY observed_at,id").map_err(|_|"read observations failed")?;let observations=oq.query_map([&kid],|r|Ok(KpiObservation{id:r.get(0)?,kpi_id:kid.clone(),value:r.get(1)?,source:parse_source(&r.get::<_,String>(2)?),evidence_reference:r.get(3)?,observed_at_unix_ms:r.get(4)?})).map_err(|_|"read observations failed")?.collect::<Result<Vec<_>,_>>().map_err(|_|"read observations failed")?;Ok(KpiDefinition{id:kid,experiment_id:id.into(),name,kind:parse_kind(&kind),direction:if direction=="at_most"{KpiDirection::AtMost}else{KpiDirection::AtLeast},target,unit,observations})}).collect::<Result<Vec<_>,String>>()?
    };
    let evaluations = {
        let mut q=conn.prepare("SELECT id,stage,recommendation,reason,evidence_count,observed_revenue_minor,revenue_evidence,created_at FROM experiment_evaluations WHERE experiment_id=?1 ORDER BY created_at,id").map_err(|_|"read evaluations failed")?;
        q.query_map([id], |r| {
            Ok(ExperimentEvaluation {
                id: r.get(0)?,
                experiment_id: id.into(),
                stage: parse_stage(&r.get::<_, String>(1)?),
                recommendation: r.get(2)?,
                reason: r.get(3)?,
                evidence_count: r.get(4)?,
                observed_revenue_minor: r.get(5)?,
                revenue_evidence: r.get(6)?,
                created_at_unix_ms: r.get(7)?,
            })
        })
        .map_err(|_| "read evaluations failed")?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|_| "read evaluations failed")?
    };
    Ok(BusinessExperiment {
        id: id.into(),
        idea,
        hypothesis,
        stage: parse_stage(&stage),
        budget_currency: currency,
        budget_limit_minor: budget,
        actual_spend_minor: None,
        spend_tracking: "unavailable",
        created_at_unix_ms: created,
        updated_at_unix_ms: updated,
        notes,
        kpis,
        evaluations,
    })
}
fn ensure_active(lock: &Mutex<Connection>, id: &str) -> Result<(), String> {
    let active:bool=lock.lock().map_err(|_|"experiment state unavailable")?.query_row("SELECT EXISTS(SELECT 1 FROM experiments WHERE id=?1 AND stage NOT IN ('completed','stopped'))",[id],|r|r.get(0)).map_err(|_|"read experiment failed")?;
    if active {
        Ok(())
    } else {
        Err("active experiment not found".into())
    }
}
fn validate(v: &str, label: &str, max: usize) -> Result<(), String> {
    if v.trim().is_empty() || v.len() > max || v.chars().any(|c| c.is_control() && c != '\n') {
        Err(format!("{label} must be non-empty bounded text"))
    } else {
        Ok(())
    }
}
fn validate_http_url(v: &str) -> Result<(), String> {
    if v.len() > 4096
        || (!v.starts_with("https://") && !v.starts_with("http://"))
        || v.chars().any(char::is_whitespace)
    {
        Err("source URL must be bounded HTTP or HTTPS".into())
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
fn stage_name(v: &ExperimentStage) -> &'static str {
    match v {
        ExperimentStage::Intake => "intake",
        ExperimentStage::Research => "research",
        ExperimentStage::Validation => "validation",
        ExperimentStage::Pilot => "pilot",
        ExperimentStage::Completed => "completed",
        ExperimentStage::Stopped => "stopped",
    }
}
fn parse_stage(v: &str) -> ExperimentStage {
    match v {
        "research" => ExperimentStage::Research,
        "validation" => ExperimentStage::Validation,
        "pilot" => ExperimentStage::Pilot,
        "completed" => ExperimentStage::Completed,
        "stopped" => ExperimentStage::Stopped,
        _ => ExperimentStage::Intake,
    }
}
fn kind_name(v: &KpiKind) -> &'static str {
    match v {
        KpiKind::RevenueMinor => "revenue_minor",
        KpiKind::ConversionRate => "conversion_rate",
        KpiKind::Count => "count",
        KpiKind::DurationMs => "duration_ms",
        KpiKind::Custom => "custom",
    }
}
fn parse_kind(v: &str) -> KpiKind {
    match v {
        "revenue_minor" => KpiKind::RevenueMinor,
        "conversion_rate" => KpiKind::ConversionRate,
        "duration_ms" => KpiKind::DurationMs,
        "custom" => KpiKind::Custom,
        _ => KpiKind::Count,
    }
}
fn direction_name(v: &KpiDirection) -> &'static str {
    match v {
        KpiDirection::AtLeast => "at_least",
        KpiDirection::AtMost => "at_most",
    }
}
fn source_name(v: &EvidenceSource) -> &'static str {
    match v {
        EvidenceSource::OwnerMeasurement => "owner_measurement",
        EvidenceSource::ProviderReport => "provider_report",
        EvidenceSource::VerifiedToolResult => "verified_tool_result",
    }
}
fn parse_source(v: &str) -> EvidenceSource {
    match v {
        "provider_report" => EvidenceSource::ProviderReport,
        "verified_tool_result" => EvidenceSource::VerifiedToolResult,
        _ => EvidenceSource::OwnerMeasurement,
    }
}
#[tauri::command]
pub fn get_experiments(state: State<'_, ExperimentState>) -> Result<ExperimentSnapshot, String> {
    state.snapshot()
}
#[tauri::command]
pub fn create_experiment(
    state: State<'_, ExperimentState>,
    request: CreateExperimentRequest,
) -> Result<BusinessExperiment, String> {
    state.create(request)
}
#[tauri::command]
pub fn add_experiment_note(
    state: State<'_, ExperimentState>,
    request: AddResearchNoteRequest,
) -> Result<ResearchNote, String> {
    state.add_note(request)
}
#[tauri::command]
pub fn add_experiment_kpi(
    state: State<'_, ExperimentState>,
    request: AddKpiRequest,
) -> Result<KpiDefinition, String> {
    state.add_kpi(request)
}
#[tauri::command]
pub fn record_kpi_observation(
    state: State<'_, ExperimentState>,
    request: RecordObservationRequest,
) -> Result<KpiObservation, String> {
    state.observe(request)
}
#[tauri::command]
pub fn evaluate_experiment(
    state: State<'_, ExperimentState>,
    id: String,
) -> Result<ExperimentEvaluation, String> {
    state.evaluate(&id)
}
#[tauri::command]
pub fn advance_experiment(
    state: State<'_, ExperimentState>,
    id: String,
) -> Result<BusinessExperiment, String> {
    state.advance(&id)
}
#[tauri::command]
pub fn stop_experiment(state: State<'_, ExperimentState>, id: String) -> Result<bool, String> {
    state.stop(&id)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn state() -> ExperimentState {
        ExperimentState::from_connection(Connection::open_in_memory().unwrap()).unwrap()
    }
    fn create(s: &ExperimentState) -> BusinessExperiment {
        s.create(CreateExperimentRequest {
            idea: "Tool".into(),
            hypothesis: "Owners will adopt it".into(),
            budget_currency: "GBP".into(),
            budget_limit_minor: 10_000,
        })
        .unwrap()
    }
    #[test]
    fn evaluation_never_fabricates_revenue() {
        let s = state();
        let e = create(&s);
        let result = s.evaluate(&e.id).unwrap();
        assert_eq!(result.recommendation, "insufficient_evidence");
        assert_eq!(result.observed_revenue_minor, None);
        assert_eq!(result.revenue_evidence, "none");
        assert_eq!(
            s.snapshot().unwrap().experiments[0].actual_spend_minor,
            None
        );
    }
    #[test]
    fn observed_revenue_is_labeled_by_source() {
        let s = state();
        let e = create(&s);
        s.add_note(AddResearchNoteRequest {
            experiment_id: e.id.clone(),
            content: "Provider export reviewed".into(),
            source_url: None,
            trust: "verified_source".into(),
        })
        .unwrap();
        let k = s
            .add_kpi(AddKpiRequest {
                experiment_id: e.id,
                name: "Revenue".into(),
                kind: KpiKind::RevenueMinor,
                direction: KpiDirection::AtLeast,
                target: 1000.0,
                unit: "currency_minor".into(),
            })
            .unwrap();
        s.observe(RecordObservationRequest {
            kpi_id: k.id,
            value: 1250.0,
            source: EvidenceSource::ProviderReport,
            evidence_reference: "provider-report-2026-10".into(),
            observed_at_unix_ms: now_ms(),
        })
        .unwrap();
        let result = s.evaluate(&k.experiment_id).unwrap();
        assert_eq!(result.observed_revenue_minor, Some(1250));
        assert_eq!(result.revenue_evidence, "provider_report");
        assert_eq!(result.recommendation, "advance");
    }
    #[test]
    fn stage_cannot_advance_without_evidence_gate() {
        let s = state();
        let e = create(&s);
        assert!(s.advance(&e.id).is_err());
        s.evaluate(&e.id).unwrap();
        assert!(s.advance(&e.id).is_err());
    }
    #[test]
    fn budget_is_declaration_not_spend() {
        let s = state();
        let e = create(&s);
        assert_eq!(e.budget_limit_minor, 10_000);
        assert_eq!(e.actual_spend_minor, None);
        assert_eq!(e.spend_tracking, "unavailable");
        assert_eq!(s.snapshot().unwrap().financial_execution, "unavailable");
    }
}
