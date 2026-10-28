mod logging;

use serde::{Deserialize, Serialize};
use std::{
    collections::BTreeMap,
    sync::Mutex,
    time::{Duration, SystemTime, UNIX_EPOCH},
};
use tauri::{Manager, RunEvent, State};

#[derive(Debug)]
struct ShellState {
    session_id: String,
    started_at_unix_ms: u64,
    lifecycle: Mutex<ShellLifecycle>,
}

#[derive(Clone, Copy, Debug, Serialize)]
#[serde(rename_all = "snake_case")]
enum ShellLifecycle {
    Starting,
    Ready,
    Exiting,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ShellStatus {
    app_name: &'static str,
    version: &'static str,
    platform: &'static str,
    lifecycle: ShellLifecycle,
    session_id: String,
    started_at_unix_ms: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct FrontendLogEntry {
    level: LogLevel,
    event: String,
    message: String,
    #[serde(default)]
    context: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ExitRequest {
    reason: ExitReason,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ExitReason {
    OwnerRequest,
    SmokeTest,
}

#[tauri::command]
fn get_shell_status(state: State<'_, ShellState>) -> Result<ShellStatus, String> {
    let lifecycle = *state
        .lifecycle
        .lock()
        .map_err(|_| "shell state unavailable".to_owned())?;
    Ok(ShellStatus {
        app_name: "SAIKI",
        version: env!("CARGO_PKG_VERSION"),
        platform: "windows",
        lifecycle,
        session_id: state.session_id.clone(),
        started_at_unix_ms: state.started_at_unix_ms,
    })
}

#[tauri::command]
fn write_frontend_log(entry: FrontendLogEntry) -> Result<(), String> {
    let event = logging::sanitize_field(&entry.event, 80)?;
    let message = logging::sanitize_field(&entry.message, 500)?;
    if entry.context.len() > 20 {
        return Err("log context exceeds 20 fields".into());
    }
    let context = logging::redact_context(entry.context);
    match entry.level {
        LogLevel::Debug => {
            log::debug!(target: "saiki.renderer", "event={event} message={message} context={context}")
        }
        LogLevel::Info => {
            log::info!(target: "saiki.renderer", "event={event} message={message} context={context}")
        }
        LogLevel::Warn => {
            log::warn!(target: "saiki.renderer", "event={event} message={message} context={context}")
        }
        LogLevel::Error => {
            log::error!(target: "saiki.renderer", "event={event} message={message} context={context}")
        }
    }
    Ok(())
}

#[tauri::command]
fn request_app_exit(
    app: tauri::AppHandle,
    state: State<'_, ShellState>,
    request: ExitRequest,
) -> Result<(), String> {
    *state
        .lifecycle
        .lock()
        .map_err(|_| "shell state unavailable".to_owned())? = ShellLifecycle::Exiting;
    log::info!(target: "saiki.lifecycle", "event=shell.exit_requested reason={:?}", request.reason);
    app.exit(0);
    Ok(())
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

pub fn run() {
    let app = tauri::Builder::default()
        .setup(|app| {
            let log_dir = app.path().app_log_dir().map_err(|error| format!("resolve log directory: {error}"))?;
            logging::init(&log_dir).map_err(|error| format!("initialize logging: {error}"))?;
            let state = ShellState {
                session_id: uuid::Uuid::new_v4().to_string(),
                started_at_unix_ms: now_unix_ms(),
                lifecycle: Mutex::new(ShellLifecycle::Starting),
            };
            *state.lifecycle.lock().map_err(|_| "initialize shell lifecycle")? = ShellLifecycle::Ready;
            log::info!(target: "saiki.lifecycle", "event=shell.ready version={}", env!("CARGO_PKG_VERSION"));
            app.manage(state);

            if std::env::args().any(|arg| arg == "--smoke-test") {
                let handle = app.handle().clone();
                std::thread::spawn(move || {
                    std::thread::sleep(Duration::from_millis(1200));
                    log::info!(target: "saiki.lifecycle", "event=shell.smoke_exit");
                    handle.exit(0);
                });
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![get_shell_status, write_frontend_log, request_app_exit])
        .build(tauri::generate_context!())
        .expect("failed to build SAIKI native shell");

    app.run(|_, event| match event {
        RunEvent::ExitRequested { .. } => {
            log::info!(target: "saiki.lifecycle", "event=shell.exit_requested_by_runtime")
        }
        RunEvent::Exit => log::info!(target: "saiki.lifecycle", "event=shell.exited_cleanly"),
        _ => {}
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn status_serializes_to_typescript_shape() {
        let status = ShellStatus {
            app_name: "SAIKI",
            version: "0.1.0",
            platform: "windows",
            lifecycle: ShellLifecycle::Ready,
            session_id: "test".into(),
            started_at_unix_ms: 1,
        };
        let json = serde_json::to_value(status).unwrap();
        assert_eq!(json["appName"], "SAIKI");
        assert_eq!(json["lifecycle"], "ready");
        assert_eq!(json["startedAtUnixMs"], 1);
    }

    #[test]
    fn ipc_rejects_unknown_fields() {
        let value = r#"{"reason":"owner_request","modelApproved":true}"#;
        assert!(serde_json::from_str::<ExitRequest>(value).is_err());
    }
}
