use serde::{Deserialize, Serialize};
use std::sync::Mutex;
use tauri::{AppHandle, LogicalSize, Manager, State};

const APPROVALS_UNAVAILABLE: &str =
    "Policy-gated execution is not implemented in the completed phases.";

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkspaceMode {
    Floating,
    Full,
}

#[derive(Clone, Copy, Debug)]
pub struct WorkspaceRuntime {
    pub mode: WorkspaceMode,
    pub reduced_motion: bool,
    pub speech_bubbles: bool,
}

#[derive(Debug)]
pub struct WorkspaceState(pub Mutex<WorkspaceRuntime>);

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalInbox {
    availability: &'static str,
    reason: &'static str,
    pending: Vec<ApprovalSummary>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ApprovalSummary {
    id: String,
    action: String,
    target: String,
    payload_digest: String,
    expires_at_unix_ms: u64,
    status: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WorkspaceStatus {
    mode: WorkspaceMode,
    reduced_motion: bool,
    speech_bubbles: bool,
    approvals: ApprovalInbox,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct UiPreferences {
    reduced_motion: bool,
    speech_bubbles: bool,
}

fn status(runtime: WorkspaceRuntime) -> WorkspaceStatus {
    WorkspaceStatus {
        mode: runtime.mode,
        reduced_motion: runtime.reduced_motion,
        speech_bubbles: runtime.speech_bubbles,
        approvals: ApprovalInbox {
            availability: "unavailable",
            reason: APPROVALS_UNAVAILABLE,
            pending: Vec::new(),
        },
    }
}

#[tauri::command]
pub fn get_workspace_state(state: State<'_, WorkspaceState>) -> Result<WorkspaceStatus, String> {
    state
        .0
        .lock()
        .map(|runtime| status(*runtime))
        .map_err(|_| "workspace state unavailable".to_owned())
}

#[tauri::command]
pub fn set_workspace_mode(
    app: AppHandle,
    state: State<'_, WorkspaceState>,
    mode: WorkspaceMode,
) -> Result<WorkspaceStatus, String> {
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "workspace window unavailable".to_owned())?;
    let size = match mode {
        WorkspaceMode::Floating => LogicalSize::new(560.0, 300.0),
        WorkspaceMode::Full => LogicalSize::new(1040.0, 720.0),
    };
    window
        .set_size(size)
        .map_err(|error| format!("resize workspace: {error}"))?;
    window
        .center()
        .map_err(|error| format!("center workspace: {error}"))?;
    let mut runtime = state
        .0
        .lock()
        .map_err(|_| "workspace state unavailable".to_owned())?;
    runtime.mode = mode;
    log::info!(target: "saiki.workspace", "event=workspace.mode_changed mode={mode:?}");
    Ok(status(*runtime))
}

#[tauri::command]
pub fn set_ui_preferences(
    state: State<'_, WorkspaceState>,
    preferences: UiPreferences,
) -> Result<WorkspaceStatus, String> {
    let mut runtime = state
        .0
        .lock()
        .map_err(|_| "workspace state unavailable".to_owned())?;
    runtime.reduced_motion = preferences.reduced_motion;
    runtime.speech_bubbles = preferences.speech_bubbles;
    log::info!(target: "saiki.workspace", "event=workspace.preferences_changed reduced_motion={} speech_bubbles={}", runtime.reduced_motion, runtime.speech_bubbles);
    Ok(status(*runtime))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn approval_inbox_is_truthfully_unavailable_and_empty() {
        let value = serde_json::to_value(status(WorkspaceRuntime {
            mode: WorkspaceMode::Full,
            reduced_motion: false,
            speech_bubbles: true,
        }))
        .unwrap();
        assert_eq!(value["approvals"]["availability"], "unavailable");
        assert_eq!(value["approvals"]["pending"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn preferences_reject_unknown_authority_fields() {
        assert!(
            serde_json::from_str::<UiPreferences>(
                r#"{"reducedMotion":false,"speechBubbles":true,"modelApproved":true}"#
            )
            .is_err()
        );
    }
}
