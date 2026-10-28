//! Windows and file tools, owned by the Rust core.
//!
//! All file operations are confined to a single owner-configured workspace
//! root: paths are rejected if absolute or containing `..`, and the deepest
//! existing ancestor of the target is canonicalized and required to stay
//! within the canonical root (so symlink escapes are caught). Writes capture
//! the prior bytes and return a rollback token, making an overwrite reversible
//! and a newly created file removable. Application launching is allowlist-only.
//!
//! Routing these operations through the Phase 12 policy executor is the
//! executor phase's responsibility; this module's own security boundary is
//! workspace confinement and the launch allowlist.

use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Component, Path, PathBuf},
    sync::Mutex,
};
use tauri::State;

const MAX_FILE_BYTES: usize = 1024 * 1024;
const MAX_LIST_ENTRIES: usize = 2000;
const MAX_PATH_BYTES: usize = 1024;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllowedApp {
    id: String,
    name: String,
    path: String,
}

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct SysToolsConfig {
    #[serde(default)]
    workspace_root: Option<String>,
    #[serde(default)]
    allowed_apps: Vec<AllowedApp>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SystemMetrics {
    cpu_count: usize,
    os: &'static str,
    arch: &'static str,
    pointer_width_bits: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SysToolsStatus {
    workspace_root: Option<String>,
    workspace_configured: bool,
    workspace_present: bool,
    allowed_apps: Vec<AllowedApp>,
    metrics: SystemMetrics,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DirEntryInfo {
    name: String,
    is_dir: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FileContent {
    path: String,
    content: String,
    bytes: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WriteResult {
    path: String,
    bytes: usize,
    previous_existed: bool,
    rollback_token: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PathRequest {
    path: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct WriteRequest {
    path: String,
    content: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct AddAppRequest {
    name: String,
    path: String,
}

struct RollbackEntry {
    path: PathBuf,
    previous: Option<Vec<u8>>,
}

pub struct SysToolsState {
    config_path: PathBuf,
    config: Mutex<SysToolsConfig>,
    rollbacks: Mutex<HashMap<String, RollbackEntry>>,
}

impl SysToolsState {
    pub fn load(config_dir: &Path) -> Result<Self, String> {
        let config_path = config_dir.join("systools-config.json");
        let config = match fs::read(&config_path) {
            Ok(bytes) => {
                serde_json::from_slice(&bytes).map_err(|_| "systools configuration is invalid")?
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => SysToolsConfig::default(),
            Err(_) => return Err("systools configuration is unavailable".into()),
        };
        Ok(Self {
            config_path,
            config: Mutex::new(config),
            rollbacks: Mutex::new(HashMap::new()),
        })
    }

    fn workspace_root(&self) -> Result<PathBuf, String> {
        let config = self
            .config
            .lock()
            .map_err(|_| "systools state unavailable")?;
        let root = config
            .workspace_root
            .as_deref()
            .ok_or("a workspace root must be configured before file operations")?;
        Ok(PathBuf::from(root))
    }

    fn status(&self) -> Result<SysToolsStatus, String> {
        let config = self
            .config
            .lock()
            .map_err(|_| "systools state unavailable")?;
        let workspace_present = config
            .workspace_root
            .as_deref()
            .map(|root| Path::new(root).is_dir())
            .unwrap_or(false);
        Ok(SysToolsStatus {
            workspace_root: config.workspace_root.clone(),
            workspace_configured: config.workspace_root.is_some(),
            workspace_present,
            allowed_apps: config.allowed_apps.clone(),
            metrics: metrics(),
        })
    }
}

/// Resolve `rel` inside `root`, refusing anything that would escape the
/// workspace. This is the single confinement chokepoint for every file
/// operation.
fn confine(root: &Path, rel: &str) -> Result<PathBuf, String> {
    if rel.trim().is_empty() || rel.len() > MAX_PATH_BYTES {
        return Err("path must be 1-1024 characters".into());
    }
    let rel_path = Path::new(rel);
    if rel_path.is_absolute() {
        return Err("path must be relative to the workspace".into());
    }
    if rel_path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::Prefix(_) | Component::RootDir
        )
    }) {
        return Err("path must not contain '..' or a drive/root".into());
    }
    let canonical_root = root
        .canonicalize()
        .map_err(|_| "the configured workspace root is unavailable")?;
    let candidate = canonical_root.join(rel_path);
    // Canonicalize the deepest existing ancestor (resolves symlinks) and verify
    // it stays within the workspace root.
    let mut probe = candidate.clone();
    loop {
        if probe.exists() {
            break;
        }
        match probe.parent() {
            Some(parent) => probe = parent.to_path_buf(),
            None => break,
        }
    }
    let canonical_probe = probe
        .canonicalize()
        .map_err(|_| "the resolved path is unavailable")?;
    if !canonical_probe.starts_with(&canonical_root) {
        return Err("path escapes the configured workspace".into());
    }
    Ok(candidate)
}

fn metrics() -> SystemMetrics {
    let cpu_count = std::thread::available_parallelism()
        .map(|value| value.get())
        .unwrap_or(1);
    SystemMetrics {
        cpu_count,
        os: std::env::consts::OS,
        arch: std::env::consts::ARCH,
        pointer_width_bits: usize::BITS as usize,
    }
}

fn persist(path: &Path, config: &SysToolsConfig) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or("systools configuration path is invalid")?;
    fs::create_dir_all(parent).map_err(|_| "create systools configuration directory failed")?;
    let temporary = path.with_extension("json.tmp");
    fs::write(
        &temporary,
        serde_json::to_vec_pretty(config).map_err(|_| "serialize systools configuration failed")?,
    )
    .map_err(|_| "write systools configuration failed")?;
    fs::rename(&temporary, path).map_err(|_| "commit systools configuration failed".to_string())
}

#[tauri::command]
pub fn get_systools_status(state: State<'_, SysToolsState>) -> Result<SysToolsStatus, String> {
    state.status()
}

#[tauri::command]
pub fn configure_workspace(
    state: State<'_, SysToolsState>,
    root: String,
) -> Result<SysToolsStatus, String> {
    let trimmed = root.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_PATH_BYTES {
        return Err("workspace root must be 1-1024 characters".into());
    }
    if !Path::new(trimmed).is_dir() {
        return Err("workspace root must be an existing directory".into());
    }
    {
        let mut config = state
            .config
            .lock()
            .map_err(|_| "systools state unavailable")?;
        config.workspace_root = Some(trimmed.to_owned());
        persist(&state.config_path, &config)?;
    }
    state.status()
}

#[tauri::command]
pub fn add_allowed_app(
    state: State<'_, SysToolsState>,
    request: AddAppRequest,
) -> Result<SysToolsStatus, String> {
    let name = request.name.trim();
    let path = request.path.trim();
    if name.is_empty() || name.len() > 100 {
        return Err("app name must be 1-100 characters".into());
    }
    if !Path::new(path).is_file() {
        return Err("app path must be an existing executable file".into());
    }
    {
        let mut config = state
            .config
            .lock()
            .map_err(|_| "systools state unavailable")?;
        config.allowed_apps.push(AllowedApp {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_owned(),
            path: path.to_owned(),
        });
        persist(&state.config_path, &config)?;
    }
    state.status()
}

#[tauri::command]
pub fn remove_allowed_app(
    state: State<'_, SysToolsState>,
    id: String,
) -> Result<SysToolsStatus, String> {
    {
        let mut config = state
            .config
            .lock()
            .map_err(|_| "systools state unavailable")?;
        config.allowed_apps.retain(|app| app.id != id);
        persist(&state.config_path, &config)?;
    }
    state.status()
}

#[tauri::command]
pub fn launch_app(state: State<'_, SysToolsState>, id: String) -> Result<bool, String> {
    let app_path = {
        let config = state
            .config
            .lock()
            .map_err(|_| "systools state unavailable")?;
        config
            .allowed_apps
            .iter()
            .find(|app| app.id == id)
            .map(|app| app.path.clone())
            .ok_or("that application is not on the allowlist")?
    };
    if !Path::new(&app_path).is_file() {
        return Err("the allowlisted application path no longer exists".into());
    }
    std::process::Command::new(&app_path)
        .spawn()
        .map_err(|_| "launching the application failed")?;
    log::info!(target: "saiki.systools", "event=systools.app_launched id={id}");
    Ok(true)
}

#[tauri::command]
pub fn read_workspace_file(
    state: State<'_, SysToolsState>,
    request: PathRequest,
) -> Result<FileContent, String> {
    let root = state.workspace_root()?;
    let absolute = confine(&root, &request.path)?;
    let metadata = fs::metadata(&absolute).map_err(|_| "file is not readable")?;
    if !metadata.is_file() {
        return Err("path is not a file".into());
    }
    if metadata.len() as usize > MAX_FILE_BYTES {
        return Err("file exceeds the 1 MB read limit".into());
    }
    let bytes = fs::read(&absolute).map_err(|_| "reading the file failed")?;
    let content = String::from_utf8(bytes).map_err(|_| "file is not UTF-8 text")?;
    Ok(FileContent {
        path: request.path,
        bytes: content.len(),
        content,
    })
}

#[tauri::command]
pub fn list_workspace_dir(
    state: State<'_, SysToolsState>,
    request: PathRequest,
) -> Result<Vec<DirEntryInfo>, String> {
    let root = state.workspace_root()?;
    // An empty-relative "." lists the root itself.
    let rel = if request.path.trim().is_empty() || request.path == "." {
        ".".to_owned()
    } else {
        request.path.clone()
    };
    let absolute = if rel == "." {
        root.canonicalize()
            .map_err(|_| "workspace root unavailable")?
    } else {
        confine(&root, &rel)?
    };
    let read = fs::read_dir(&absolute).map_err(|_| "directory is not readable")?;
    let mut entries = Vec::new();
    for entry in read.flatten() {
        if entries.len() >= MAX_LIST_ENTRIES {
            break;
        }
        let file_type = entry
            .file_type()
            .map_err(|_| "reading a directory entry failed")?;
        entries.push(DirEntryInfo {
            name: entry.file_name().to_string_lossy().into_owned(),
            is_dir: file_type.is_dir(),
        });
    }
    entries
        .sort_by(|a, b| (b.is_dir, a.name.to_lowercase()).cmp(&(a.is_dir, b.name.to_lowercase())));
    Ok(entries)
}

#[tauri::command]
pub fn write_workspace_file(
    state: State<'_, SysToolsState>,
    request: WriteRequest,
) -> Result<WriteResult, String> {
    if request.content.len() > MAX_FILE_BYTES {
        return Err("content exceeds the 1 MB write limit".into());
    }
    let root = state.workspace_root()?;
    let absolute = confine(&root, &request.path)?;
    if let Some(parent) = absolute.parent() {
        fs::create_dir_all(parent).map_err(|_| "creating the parent directory failed")?;
    }
    let previous = fs::read(&absolute).ok();
    let previous_existed = previous.is_some();
    fs::write(&absolute, request.content.as_bytes()).map_err(|_| "writing the file failed")?;
    let token = uuid::Uuid::new_v4().to_string();
    {
        let mut rollbacks = state
            .rollbacks
            .lock()
            .map_err(|_| "systools state unavailable")?;
        rollbacks.insert(
            token.clone(),
            RollbackEntry {
                path: absolute,
                previous,
            },
        );
    }
    Ok(WriteResult {
        bytes: request.content.len(),
        path: request.path,
        previous_existed,
        rollback_token: token,
    })
}

#[tauri::command]
pub fn rollback_write(state: State<'_, SysToolsState>, token: String) -> Result<bool, String> {
    let entry = {
        let mut rollbacks = state
            .rollbacks
            .lock()
            .map_err(|_| "systools state unavailable")?;
        rollbacks.remove(&token)
    };
    let Some(entry) = entry else {
        return Ok(false);
    };
    match entry.previous {
        Some(bytes) => {
            fs::write(&entry.path, bytes).map_err(|_| "restoring the previous content failed")?
        }
        None => {
            if entry.path.exists() {
                fs::remove_file(&entry.path).map_err(|_| "removing the created file failed")?;
            }
        }
    }
    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn state_with_root(root: &Path) -> SysToolsState {
        let dir = tempfile::tempdir().unwrap();
        let state = SysToolsState::load(dir.path()).unwrap();
        {
            let mut config = state.config.lock().unwrap();
            config.workspace_root = Some(root.to_str().unwrap().to_owned());
        }
        // The config dir is only needed at load; these tests never persist, so
        // it may drop here while the workspace root (owned by the caller) lives on.
        drop(dir);
        state
    }

    #[test]
    fn confine_rejects_escapes() {
        let root = tempfile::tempdir().unwrap();
        assert!(confine(root.path(), "../outside.txt").is_err());
        assert!(confine(root.path(), "sub/../../outside.txt").is_err());
        assert!(confine(root.path(), "C:/Windows/system32/x").is_err());
        assert!(confine(root.path(), "/etc/passwd").is_err());
        // A normal relative path within the workspace is accepted.
        assert!(confine(root.path(), "notes/today.txt").is_ok());
    }

    #[test]
    fn write_then_rollback_restores_previous_content() {
        let root = tempfile::tempdir().unwrap();
        std::fs::write(root.path().join("doc.txt"), b"original").unwrap();
        let state = state_with_root(root.path());
        let result = write_body(&state, "doc.txt", "replaced").unwrap();
        assert!(result.previous_existed);
        assert_eq!(
            std::fs::read_to_string(root.path().join("doc.txt")).unwrap(),
            "replaced"
        );
        assert!(rollback(&state, &result.rollback_token).unwrap());
        assert_eq!(
            std::fs::read_to_string(root.path().join("doc.txt")).unwrap(),
            "original"
        );
    }

    #[test]
    fn rollback_removes_a_newly_created_file() {
        let root = tempfile::tempdir().unwrap();
        let state = state_with_root(root.path());
        let result = write_body(&state, "new.txt", "fresh").unwrap();
        assert!(!result.previous_existed);
        assert!(root.path().join("new.txt").exists());
        assert!(rollback(&state, &result.rollback_token).unwrap());
        assert!(!root.path().join("new.txt").exists());
    }

    #[test]
    fn write_outside_workspace_is_refused() {
        let root = tempfile::tempdir().unwrap();
        let state = state_with_root(root.path());
        assert!(write_body(&state, "../escape.txt", "x").is_err());
    }

    #[test]
    fn file_ops_require_a_configured_workspace() {
        let dir = tempfile::tempdir().unwrap();
        let state = SysToolsState::load(dir.path()).unwrap();
        assert!(state.workspace_root().is_err());
    }

    #[test]
    fn config_persists_across_reopen() {
        let config_dir = tempfile::tempdir().unwrap();
        let workspace = tempfile::tempdir().unwrap();
        let app = workspace.path().join("tool.exe");
        std::fs::write(&app, b"stub").unwrap();
        {
            let state = SysToolsState::load(config_dir.path()).unwrap();
            {
                let mut config = state.config.lock().unwrap();
                config.workspace_root = Some(workspace.path().to_str().unwrap().to_owned());
                config.allowed_apps.push(AllowedApp {
                    id: "a1".into(),
                    name: "Tool".into(),
                    path: app.to_str().unwrap().to_owned(),
                });
                persist(&state.config_path, &config).unwrap();
            }
        }
        let reopened = SysToolsState::load(config_dir.path()).unwrap();
        let status = reopened.status().unwrap();
        assert!(status.workspace_configured);
        assert_eq!(status.allowed_apps.len(), 1);
        assert_eq!(status.allowed_apps[0].name, "Tool");
    }

    #[test]
    fn metrics_are_truthful() {
        let m = metrics();
        assert!(m.cpu_count >= 1);
        assert!(m.pointer_width_bits == 32 || m.pointer_width_bits == 64);
    }

    // Test helpers that exercise the confine + rollback logic without the Tauri
    // State wrapper.
    fn write_body(state: &SysToolsState, rel: &str, content: &str) -> Result<WriteResult, String> {
        let root = state.workspace_root()?;
        let absolute = confine(&root, rel)?;
        if let Some(parent) = absolute.parent() {
            fs::create_dir_all(parent).map_err(|_| "mkdir failed")?;
        }
        let previous = fs::read(&absolute).ok();
        let previous_existed = previous.is_some();
        fs::write(&absolute, content.as_bytes()).map_err(|_| "write failed")?;
        let token = uuid::Uuid::new_v4().to_string();
        state.rollbacks.lock().unwrap().insert(
            token.clone(),
            RollbackEntry {
                path: absolute,
                previous,
            },
        );
        Ok(WriteResult {
            bytes: content.len(),
            path: rel.to_owned(),
            previous_existed,
            rollback_token: token,
        })
    }

    fn rollback(state: &SysToolsState, token: &str) -> Result<bool, String> {
        let entry = state.rollbacks.lock().unwrap().remove(token);
        let Some(entry) = entry else { return Ok(false) };
        match entry.previous {
            Some(bytes) => fs::write(&entry.path, bytes).map_err(|_| "restore failed")?,
            None => {
                if entry.path.exists() {
                    fs::remove_file(&entry.path).map_err(|_| "remove failed")?;
                }
            }
        }
        Ok(true)
    }
}
