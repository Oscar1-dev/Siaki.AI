//! Terminal and developer tools, owned by the Rust core.
//!
//! Real repo analysis (Git status/diff), a bounded shell runner that is both
//! allowlisted and **policy-gated** (every command is checked against the
//! Phase 12 policy engine before it runs — this is the first place the policy
//! engine gates a real side effect), per-command cancellation, an
//! owner-configured VS Code bridge, and a Codex activity adapter that is
//! reported as an explicit `unavailable` capability (no supported local source
//! is wired, and nothing is fabricated).

use crate::policy::{Decision, PolicyEngine};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Component, Path, PathBuf},
    process::Stdio,
    sync::{Arc, Mutex},
    time::Duration,
};
use tauri::State;
use tokio::sync::Notify;

const MAX_OUTPUT_BYTES: usize = 64 * 1024;
const MAX_ARGS: usize = 32;
const MAX_ARG_BYTES: usize = 512;
const DEFAULT_TIMEOUT_MS: u64 = 60_000;
const CODEX_UNAVAILABLE: &str =
    "No supported local Codex activity source is available in this build; no activity is shown.";

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DevToolsConfig {
    #[serde(default)]
    repo_root: Option<String>,
    #[serde(default)]
    allowed_commands: Vec<String>,
    #[serde(default)]
    vscode_path: Option<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Capability {
    available: bool,
    reason: &'static str,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DevToolsStatus {
    repo_root: Option<String>,
    repo_configured: bool,
    is_git_repo: bool,
    allowed_commands: Vec<String>,
    vscode_configured: bool,
    vscode_present: bool,
    codex: Capability,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitStatusEntry {
    status: String,
    path: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitStatus {
    branch: String,
    entries: Vec<GitStatusEntry>,
    clean: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GitDiff {
    diff: String,
    truncated: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandResult {
    request_id: String,
    command: String,
    outcome: &'static str,
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
    truncated: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConfigureDevToolsRequest {
    #[serde(default)]
    repo_root: Option<String>,
    #[serde(default)]
    vscode_path: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct RunCommandRequest {
    request_id: String,
    command: String,
    #[serde(default)]
    args: Vec<String>,
    #[serde(default = "default_timeout")]
    timeout_ms: u64,
}
fn default_timeout() -> u64 {
    DEFAULT_TIMEOUT_MS
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DiffRequest {
    #[serde(default)]
    path: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct OpenRequest {
    path: String,
}

pub struct DevToolsState {
    config_path: PathBuf,
    config: Mutex<DevToolsConfig>,
    active: Mutex<HashMap<String, Arc<Notify>>>,
}

impl DevToolsState {
    pub fn load(config_dir: &Path) -> Result<Self, String> {
        let config_path = config_dir.join("devtools-config.json");
        let config = match fs::read(&config_path) {
            Ok(bytes) => {
                serde_json::from_slice(&bytes).map_err(|_| "devtools configuration is invalid")?
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => DevToolsConfig::default(),
            Err(_) => return Err("devtools configuration is unavailable".into()),
        };
        Ok(Self {
            config_path,
            config: Mutex::new(config),
            active: Mutex::new(HashMap::new()),
        })
    }

    fn repo_root(&self) -> Result<PathBuf, String> {
        let config = self
            .config
            .lock()
            .map_err(|_| "devtools state unavailable")?;
        let root = config
            .repo_root
            .as_deref()
            .ok_or("a repository root must be configured first")?;
        Ok(PathBuf::from(root))
    }

    fn status(&self) -> Result<DevToolsStatus, String> {
        let config = self
            .config
            .lock()
            .map_err(|_| "devtools state unavailable")?;
        let is_git_repo = config
            .repo_root
            .as_deref()
            .map(|root| Path::new(root).join(".git").exists())
            .unwrap_or(false);
        let vscode_present = config
            .vscode_path
            .as_deref()
            .map(|path| Path::new(path).is_file())
            .unwrap_or(false);
        Ok(DevToolsStatus {
            repo_root: config.repo_root.clone(),
            repo_configured: config.repo_root.is_some(),
            is_git_repo,
            allowed_commands: config.allowed_commands.clone(),
            vscode_configured: config.vscode_path.is_some(),
            vscode_present,
            codex: Capability {
                available: false,
                reason: CODEX_UNAVAILABLE,
            },
        })
    }
}

fn signal_cancel(active: &Mutex<HashMap<String, Arc<Notify>>>, id: &str) -> bool {
    if let Ok(map) = active.lock() {
        if let Some(cancel) = map.get(id) {
            cancel.notify_waiters();
            return true;
        }
    }
    false
}

fn authorize_command(policy: &PolicyEngine, command: &str, args: &[String]) -> Result<(), String> {
    let decision = policy.authorize(
        "devtools",
        "shell.exec",
        command,
        &serde_json::json!({ "command": command, "args": args }),
    )?;
    match decision {
        Decision::Allow => Ok(()),
        Decision::Deny { reason } => Err(format!("denied by policy: {reason}")),
        Decision::ApprovalRequired { reason, .. } => {
            Err(format!("owner approval required: {reason}"))
        }
    }
}

fn truncate(bytes: Vec<u8>) -> (String, bool) {
    let truncated = bytes.len() > MAX_OUTPUT_BYTES;
    let slice = if truncated {
        &bytes[..MAX_OUTPUT_BYTES]
    } else {
        &bytes[..]
    };
    (String::from_utf8_lossy(slice).into_owned(), truncated)
}

fn parse_git_status(stdout: &str) -> GitStatus {
    let mut branch = "unknown".to_owned();
    let mut entries = Vec::new();
    for line in stdout.lines() {
        if let Some(rest) = line.strip_prefix("## ") {
            branch = rest.split("...").next().unwrap_or(rest).trim().to_owned();
        } else if line.len() > 3 {
            entries.push(GitStatusEntry {
                status: line[..2].trim().to_owned(),
                path: line[3..].trim().to_owned(),
            });
        }
    }
    GitStatus {
        branch,
        clean: entries.is_empty(),
        entries,
    }
}

/// Reject a path that would escape the repo root (same confinement rule as the
/// file tools), so the VS Code bridge cannot be aimed outside the workspace.
fn confine(root: &Path, rel: &str) -> Result<PathBuf, String> {
    let rel_path = Path::new(rel);
    if rel.trim().is_empty() || rel_path.is_absolute() {
        return Err("path must be relative to the repository".into());
    }
    if rel_path.components().any(|c| {
        matches!(
            c,
            Component::ParentDir | Component::Prefix(_) | Component::RootDir
        )
    }) {
        return Err("path must not contain '..' or a drive/root".into());
    }
    let canonical_root = root
        .canonicalize()
        .map_err(|_| "repository root unavailable")?;
    Ok(canonical_root.join(rel_path))
}

fn persist(path: &Path, config: &DevToolsConfig) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or("devtools configuration path is invalid")?;
    fs::create_dir_all(parent).map_err(|_| "create devtools configuration directory failed")?;
    let temporary = path.with_extension("json.tmp");
    fs::write(
        &temporary,
        serde_json::to_vec_pretty(config).map_err(|_| "serialize devtools configuration failed")?,
    )
    .map_err(|_| "write devtools configuration failed")?;
    fs::rename(&temporary, path).map_err(|_| "commit devtools configuration failed".to_string())
}

#[tauri::command]
pub fn get_devtools_status(state: State<'_, DevToolsState>) -> Result<DevToolsStatus, String> {
    state.status()
}

#[tauri::command]
pub fn configure_devtools(
    state: State<'_, DevToolsState>,
    request: ConfigureDevToolsRequest,
) -> Result<DevToolsStatus, String> {
    if let Some(root) = request.repo_root.as_deref() {
        let trimmed = root.trim();
        if !trimmed.is_empty() && !Path::new(trimmed).is_dir() {
            return Err("repository root must be an existing directory".into());
        }
    }
    {
        let mut config = state
            .config
            .lock()
            .map_err(|_| "devtools state unavailable")?;
        if let Some(root) = request.repo_root {
            let trimmed = root.trim();
            config.repo_root = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_owned())
            };
        }
        if let Some(path) = request.vscode_path {
            let trimmed = path.trim();
            config.vscode_path = if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_owned())
            };
        }
        persist(&state.config_path, &config)?;
    }
    state.status()
}

#[tauri::command]
pub fn add_allowed_command(
    state: State<'_, DevToolsState>,
    command: String,
) -> Result<DevToolsStatus, String> {
    let name = command.trim().to_lowercase();
    if name.is_empty()
        || name.len() > 64
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.')
    {
        return Err("command name must be 1-64 alphanumeric characters".into());
    }
    {
        let mut config = state
            .config
            .lock()
            .map_err(|_| "devtools state unavailable")?;
        if !config.allowed_commands.contains(&name) {
            config.allowed_commands.push(name);
        }
        persist(&state.config_path, &config)?;
    }
    state.status()
}

#[tauri::command]
pub fn remove_allowed_command(
    state: State<'_, DevToolsState>,
    command: String,
) -> Result<DevToolsStatus, String> {
    let name = command.trim().to_lowercase();
    {
        let mut config = state
            .config
            .lock()
            .map_err(|_| "devtools state unavailable")?;
        config.allowed_commands.retain(|existing| existing != &name);
        persist(&state.config_path, &config)?;
    }
    state.status()
}

#[tauri::command]
pub fn git_status(state: State<'_, DevToolsState>) -> Result<GitStatus, String> {
    let root = state.repo_root()?;
    let output = std::process::Command::new("git")
        .arg("-C")
        .arg(&root)
        .args(["status", "--porcelain=v1", "--branch"])
        .output()
        .map_err(|_| "git is not available on PATH")?;
    if !output.status.success() {
        return Err("git status failed; is this a repository?".into());
    }
    Ok(parse_git_status(&String::from_utf8_lossy(&output.stdout)))
}

#[tauri::command]
pub fn git_diff(state: State<'_, DevToolsState>, request: DiffRequest) -> Result<GitDiff, String> {
    let root = state.repo_root()?;
    let mut command = std::process::Command::new("git");
    command.arg("-C").arg(&root).args(["diff", "--no-color"]);
    if let Some(path) = request.path.as_deref() {
        let safe = confine(&root, path)?;
        command.arg("--").arg(safe);
    }
    let output = command
        .output()
        .map_err(|_| "git is not available on PATH")?;
    if !output.status.success() {
        return Err("git diff failed".into());
    }
    let (diff, truncated) = truncate(output.stdout);
    Ok(GitDiff { diff, truncated })
}

#[tauri::command]
pub async fn run_command(
    policy: State<'_, PolicyEngine>,
    devtools: State<'_, DevToolsState>,
    request: RunCommandRequest,
) -> Result<CommandResult, String> {
    if uuid::Uuid::parse_str(&request.request_id).is_err() {
        return Err("requestId must be a UUID".into());
    }
    if request.args.len() > MAX_ARGS || request.args.iter().any(|arg| arg.len() > MAX_ARG_BYTES) {
        return Err("too many or oversized arguments".into());
    }
    if !(1_000..=600_000).contains(&request.timeout_ms) {
        return Err("timeoutMs must be between 1000 and 600000".into());
    }
    let command = request.command.trim().to_lowercase();
    let (root, allowed) = {
        let config = devtools
            .config
            .lock()
            .map_err(|_| "devtools state unavailable")?;
        let root = config
            .repo_root
            .as_deref()
            .ok_or("a repository root must be configured first")?
            .to_owned();
        (
            PathBuf::from(root),
            config.allowed_commands.contains(&command),
        )
    };
    if !allowed {
        return Err("that command is not on the allowlist".into());
    }
    // Approval check: the policy engine must authorize shell execution.
    if let Err(reason) = authorize_command(&policy, &command, &request.args) {
        return Ok(CommandResult {
            request_id: request.request_id,
            command,
            outcome: "denied",
            exit_code: None,
            stdout: String::new(),
            stderr: reason,
            truncated: false,
        });
    }

    let cancel = Arc::new(Notify::new());
    {
        let mut active = devtools
            .active
            .lock()
            .map_err(|_| "devtools state unavailable")?;
        if active.contains_key(&request.request_id) {
            return Err("requestId is already active".into());
        }
        active.insert(request.request_id.clone(), cancel.clone());
    }

    let child = tokio::process::Command::new(&command)
        .args(&request.args)
        .current_dir(&root)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .kill_on_drop(true)
        .spawn();

    let result = match child {
        Err(_) => CommandResult {
            request_id: request.request_id.clone(),
            command: command.clone(),
            outcome: "unavailable",
            exit_code: None,
            stdout: String::new(),
            stderr: "failed to start the command".into(),
            truncated: false,
        },
        Ok(child) => {
            // Racing the process against cancel/timeout; if a non-process branch
            // wins, the wait_with_output future (owning the child, kill_on_drop)
            // is dropped here, which kills the child.
            tokio::select! {
                _ = cancel.notified() => finished_result(&request.request_id, &command, "cancelled", None, Vec::new(), Vec::new()),
                _ = tokio::time::sleep(Duration::from_millis(request.timeout_ms)) => finished_result(&request.request_id, &command, "unavailable", None, Vec::new(), b"command timed out".to_vec()),
                output = child.wait_with_output() => match output {
                    Ok(output) => {
                        let outcome = if output.status.success() { "succeeded" } else { "failed" };
                        finished_result(&request.request_id, &command, outcome, output.status.code(), output.stdout, output.stderr)
                    }
                    Err(_) => finished_result(&request.request_id, &command, "failed", None, Vec::new(), b"command execution failed".to_vec()),
                }
            }
        }
    };

    if let Ok(mut active) = devtools.active.lock() {
        active.remove(&request.request_id);
    }
    Ok(result)
}

fn finished_result(
    request_id: &str,
    command: &str,
    outcome: &'static str,
    exit_code: Option<i32>,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
) -> CommandResult {
    let (stdout, out_trunc) = truncate(stdout);
    let (stderr, err_trunc) = truncate(stderr);
    CommandResult {
        request_id: request_id.to_owned(),
        command: command.to_owned(),
        outcome,
        exit_code,
        stdout,
        stderr,
        truncated: out_trunc || err_trunc,
    }
}

#[tauri::command]
pub fn cancel_command(state: State<'_, DevToolsState>, request_id: String) -> Result<bool, String> {
    Ok(signal_cancel(&state.active, &request_id))
}

#[tauri::command]
pub fn open_in_vscode(
    state: State<'_, DevToolsState>,
    request: OpenRequest,
) -> Result<bool, String> {
    let (root, vscode) = {
        let config = state
            .config
            .lock()
            .map_err(|_| "devtools state unavailable")?;
        let root = config
            .repo_root
            .as_deref()
            .ok_or("a repository root must be configured first")?
            .to_owned();
        let vscode = config
            .vscode_path
            .clone()
            .ok_or("the VS Code path is not configured")?;
        (PathBuf::from(root), vscode)
    };
    if !Path::new(&vscode).is_file() {
        return Err("the configured VS Code executable no longer exists".into());
    }
    let target = confine(&root, &request.path)?;
    std::process::Command::new(&vscode)
        .arg(&target)
        .spawn()
        .map_err(|_| "launching VS Code failed")?;
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_git_porcelain_status() {
        let sample = "## main...origin/main\n M src/lib.rs\n?? new.txt\n";
        let status = parse_git_status(sample);
        assert_eq!(status.branch, "main");
        assert!(!status.clean);
        assert_eq!(status.entries.len(), 2);
        assert_eq!(status.entries[0].path, "src/lib.rs");
        assert_eq!(status.entries[1].status, "??");
    }

    #[test]
    fn clean_repo_reports_clean() {
        let status = parse_git_status("## main\n");
        assert_eq!(status.branch, "main");
        assert!(status.clean);
        assert!(status.entries.is_empty());
    }

    #[test]
    fn confine_rejects_escapes() {
        let root = tempfile::tempdir().unwrap();
        assert!(confine(root.path(), "../x").is_err());
        assert!(confine(root.path(), "/etc/passwd").is_err());
        assert!(confine(root.path(), "src/lib.rs").is_ok());
    }

    #[test]
    fn shell_requires_policy_authorization_by_default() {
        // A fresh policy engine grants nothing; shell.exec must be refused.
        let dir = tempfile::tempdir().unwrap();
        let policy = PolicyEngine::load(dir.path()).unwrap();
        let result = authorize_command(&policy, "git", &["status".to_owned()]);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("approval required"));
    }

    #[test]
    fn cancellation_registry_signals_known_ids_only() {
        let active: Mutex<HashMap<String, Arc<Notify>>> = Mutex::new(HashMap::new());
        active
            .lock()
            .unwrap()
            .insert("id-1".into(), Arc::new(Notify::new()));
        assert!(signal_cancel(&active, "id-1"));
        assert!(!signal_cancel(&active, "missing"));
    }

    #[test]
    fn real_git_repo_analysis() {
        // Exercise real repo analysis where git is available; skip cleanly if not.
        if std::process::Command::new("git")
            .arg("--version")
            .output()
            .is_err()
        {
            return;
        }
        let dir = tempfile::tempdir().unwrap();
        let root = dir.path();
        let run = |args: &[&str]| {
            std::process::Command::new("git")
                .arg("-C")
                .arg(root)
                .args(args)
                .output()
                .unwrap()
        };
        run(&["init", "-q"]);
        run(&["config", "user.email", "t@example.com"]);
        run(&["config", "user.name", "Test"]);
        std::fs::write(root.join("file.txt"), "one\n").unwrap();
        run(&["add", "."]);
        run(&["commit", "-qm", "init"]);
        std::fs::write(root.join("file.txt"), "two\n").unwrap();
        let output = run(&["status", "--porcelain=v1", "--branch"]);
        let status = parse_git_status(&String::from_utf8_lossy(&output.stdout));
        assert!(!status.clean);
        assert!(status.entries.iter().any(|entry| entry.path == "file.txt"));
    }

    #[test]
    fn config_persists_across_reopen() {
        let dir = tempfile::tempdir().unwrap();
        {
            let state = DevToolsState::load(dir.path()).unwrap();
            {
                let mut config = state.config.lock().unwrap();
                config.allowed_commands.push("git".into());
                config.repo_root = Some(dir.path().to_str().unwrap().to_owned());
                persist(&state.config_path, &config).unwrap();
            }
        }
        let reopened = DevToolsState::load(dir.path()).unwrap();
        let status = reopened.status().unwrap();
        assert!(status.repo_configured);
        assert!(status.allowed_commands.contains(&"git".to_owned()));
        assert!(!status.codex.available);
    }
}
