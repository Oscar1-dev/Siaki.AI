use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager, State};
use zeroize::Zeroize;

const RESPONSES_URL: &str = "https://api.openai.com/v1/responses";
const KEYRING_SERVICE: &str = "SAIKI.OpenAI";
const KEYRING_ACCOUNT: &str = "owner-api-key";
const MAX_PROMPT_BYTES: usize = 32 * 1024;
pub const STREAM_EVENT: &str = "saiki://openai-stream";

pub struct CloudState {
    client: reqwest::Client,
    config_path: PathBuf,
    config: Mutex<CloudConfig>,
    active: Mutex<HashMap<String, Arc<tokio::sync::Notify>>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct CloudConfig {
    enabled: bool,
    model: String,
}

impl CloudState {
    pub fn load(config_dir: &Path) -> Result<Self, String> {
        let config_path = config_dir.join("cloud-provider.json");
        let config = match fs::read(&config_path) {
            Ok(bytes) => serde_json::from_slice(&bytes)
                .map_err(|_| "cloud provider configuration is invalid")?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => CloudConfig::default(),
            Err(_) => return Err("cloud provider configuration is unavailable".into()),
        };
        let client = reqwest::Client::builder()
            .connect_timeout(Duration::from_secs(5))
            .build()
            .map_err(|_| "OpenAI client initialization failed")?;
        Ok(Self {
            client,
            config_path,
            config: Mutex::new(config),
            active: Mutex::new(HashMap::new()),
        })
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CloudStatus {
    enabled: bool,
    model: String,
    credential_present: bool,
    credential_store: &'static str,
    endpoint: &'static str,
    disclosure: CloudDisclosure,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CloudDisclosure {
    destination: &'static str,
    data: [&'static str; 2],
    storage_requested: bool,
    billing: &'static str,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ConfigureCloudRequest {
    enabled: bool,
    model: String,
    disclosure_accepted: bool,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct CloudChatRequest {
    request_id: String,
    prompt: String,
    #[serde(default = "default_timeout")]
    timeout_ms: u64,
}
fn default_timeout() -> u64 {
    120_000
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatStarted {
    request_id: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
enum StreamEvent {
    Chunk {
        request_id: String,
        content: String,
    },
    Finished {
        request_id: String,
        outcome: &'static str,
    },
    Failed {
        request_id: String,
        outcome: &'static str,
        message: String,
    },
}

fn keyring_entry() -> Result<keyring::Entry, String> {
    keyring::Entry::new(KEYRING_SERVICE, KEYRING_ACCOUNT)
        .map_err(|_| "Windows Credential Manager is unavailable".into())
}

fn credential_present() -> Result<bool, String> {
    match keyring_entry()?.get_password() {
        Ok(mut secret) => {
            let present = !secret.trim().is_empty();
            secret.zeroize();
            Ok(present)
        }
        Err(keyring::Error::NoEntry) => Ok(false),
        Err(_) => Err("OpenAI credential status is unavailable".into()),
    }
}

fn status(config: &CloudConfig) -> Result<CloudStatus, String> {
    Ok(CloudStatus {
        enabled: config.enabled,
        model: config.model.clone(),
        credential_present: credential_present()?,
        credential_store: "windows_credential_manager",
        endpoint: RESPONSES_URL,
        disclosure: CloudDisclosure {
            destination: "OpenAI API (api.openai.com)",
            data: [
                "the message entered in this view",
                "the selected model identifier",
            ],
            storage_requested: false,
            billing: "API usage may incur charges under the owner's OpenAI API account; ChatGPT subscriptions are not used or assumed.",
        },
    })
}

fn persist(path: &Path, config: &CloudConfig) -> Result<(), String> {
    let parent = path
        .parent()
        .ok_or("cloud provider configuration path is invalid")?;
    fs::create_dir_all(parent)
        .map_err(|_| "create cloud provider configuration directory failed")?;
    let temporary = path.with_extension("json.tmp");
    fs::write(
        &temporary,
        serde_json::to_vec_pretty(config)
            .map_err(|_| "serialize cloud provider configuration failed")?,
    )
    .map_err(|_| "write cloud provider configuration failed")?;
    fs::rename(&temporary, path)
        .map_err(|_| "commit cloud provider configuration failed".to_string())
}

#[tauri::command]
pub fn get_cloud_status(state: State<'_, CloudState>) -> Result<CloudStatus, String> {
    let config = state
        .config
        .lock()
        .map_err(|_| "cloud provider state unavailable")?;
    status(&config)
}

#[tauri::command]
pub fn import_openai_key_from_environment(
    state: State<'_, CloudState>,
) -> Result<CloudStatus, String> {
    let mut secret = std::env::var("OPENAI_API_KEY")
        .map_err(|_| "OPENAI_API_KEY is not set for this process")?;
    if secret.trim().len() < 20 || secret.len() > 512 {
        secret.zeroize();
        return Err("OPENAI_API_KEY has an invalid length".into());
    }
    let result = keyring_entry()?
        .set_password(&secret)
        .map_err(|_| "store OpenAI credential in Windows Credential Manager failed".to_string());
    secret.zeroize();
    result?;
    let config = state
        .config
        .lock()
        .map_err(|_| "cloud provider state unavailable")?;
    status(&config)
}

#[tauri::command]
pub fn delete_openai_key(state: State<'_, CloudState>) -> Result<CloudStatus, String> {
    match keyring_entry()?.delete_credential() {
        Ok(()) | Err(keyring::Error::NoEntry) => {}
        Err(_) => return Err("delete OpenAI credential failed".into()),
    }
    let mut config = state
        .config
        .lock()
        .map_err(|_| "cloud provider state unavailable")?;
    config.enabled = false;
    persist(&state.config_path, &config)?;
    status(&config)
}

#[tauri::command]
pub fn configure_cloud_provider(
    state: State<'_, CloudState>,
    request: ConfigureCloudRequest,
) -> Result<CloudStatus, String> {
    let model = request.model.trim();
    if model.len() > 200 {
        return Err("model must not exceed 200 characters".into());
    }
    if request.enabled {
        if !request.disclosure_accepted {
            return Err("the owner must accept the current cloud data disclosure".into());
        }
        if model.is_empty() {
            return Err("a model identifier is required".into());
        }
        if !credential_present()? {
            return Err("an OpenAI API credential is required".into());
        }
    }
    let mut config = state
        .config
        .lock()
        .map_err(|_| "cloud provider state unavailable")?;
    config.enabled = request.enabled;
    config.model = model.to_owned();
    persist(&state.config_path, &config)?;
    status(&config)
}

#[tauri::command]
pub async fn start_cloud_chat(
    app: AppHandle,
    state: State<'_, CloudState>,
    request: CloudChatRequest,
) -> Result<ChatStarted, String> {
    if uuid::Uuid::parse_str(&request.request_id).is_err() {
        return Err("requestId must be a UUID".into());
    }
    if request.prompt.trim().is_empty() || request.prompt.len() > MAX_PROMPT_BYTES {
        return Err("prompt must be 1-32768 bytes".into());
    }
    if !(1_000..=600_000).contains(&request.timeout_ms) {
        return Err("timeoutMs must be between 1000 and 600000".into());
    }
    let model = {
        let config = state
            .config
            .lock()
            .map_err(|_| "cloud provider state unavailable")?;
        if !config.enabled {
            return Err("OpenAI provider is disabled".into());
        }
        config.model.clone()
    };
    let mut api_key = keyring_entry()?
        .get_password()
        .map_err(|_| "OpenAI credential is unavailable".to_string())?;
    let cancel = Arc::new(tokio::sync::Notify::new());
    {
        let mut active = state
            .active
            .lock()
            .map_err(|_| "cloud request state unavailable")?;
        if active.contains_key(&request.request_id) {
            api_key.zeroize();
            return Err("requestId is already active".into());
        }
        active.insert(request.request_id.clone(), cancel.clone());
    }
    let client = state.client.clone();
    let id = request.request_id.clone();
    tauri::async_runtime::spawn(async move {
        let input = StreamInput {
            id: &id,
            model: &model,
            prompt: &request.prompt,
            timeout_ms: request.timeout_ms,
            api_key: &api_key,
        };
        let result = stream_response(&app, &client, input, cancel).await;
        api_key.zeroize();
        if let Err(event) = result {
            let _ = app.emit(STREAM_EVENT, event);
        }
        if let Some(state) = app.try_state::<CloudState>() {
            if let Ok(mut active) = state.active.lock() {
                active.remove(&id);
            }
        }
    });
    Ok(ChatStarted {
        request_id: request.request_id,
    })
}

#[tauri::command]
pub fn cancel_cloud_chat(state: State<'_, CloudState>, request_id: String) -> Result<bool, String> {
    let active = state
        .active
        .lock()
        .map_err(|_| "cloud request state unavailable")?;
    if let Some(cancel) = active.get(&request_id) {
        cancel.notify_waiters();
        Ok(true)
    } else {
        Ok(false)
    }
}

#[derive(Serialize)]
struct ResponseBody<'a> {
    model: &'a str,
    input: &'a str,
    stream: bool,
    store: bool,
}
struct StreamInput<'a> {
    id: &'a str,
    model: &'a str,
    prompt: &'a str,
    timeout_ms: u64,
    api_key: &'a str,
}
async fn stream_response(
    app: &AppHandle,
    client: &reqwest::Client,
    input: StreamInput<'_>,
    cancel: Arc<tokio::sync::Notify>,
) -> Result<(), StreamEvent> {
    let operation = async {
        let response = client
            .post(RESPONSES_URL)
            .bearer_auth(input.api_key)
            .json(&ResponseBody {
                model: input.model,
                input: input.prompt,
                stream: true,
                store: false,
            })
            .send()
            .await
            .map_err(|_| "OpenAI API connection failed".to_string())?;
        let status = response.status();
        if !status.is_success() {
            return Err(match status.as_u16() {
                401 | 403 => "OpenAI rejected the credential or account access".into(),
                429 => "OpenAI rate or account budget limit reached".into(),
                _ => format!("OpenAI request failed with HTTP {status}"),
            });
        }
        let mut stream = response.bytes_stream();
        let mut pending = Vec::new();
        loop {
            tokio::select! {
                _ = cancel.notified() => { let _ = app.emit(STREAM_EVENT, StreamEvent::Finished { request_id: input.id.into(), outcome: "cancelled" }); return Ok(()); }
                next = stream.next() => match next {
                    Some(Ok(bytes)) => { pending.extend_from_slice(&bytes); while let Some(pos) = pending.windows(2).position(|w| w == b"\n\n") { let frame: Vec<u8> = pending.drain(..pos + 2).collect(); if let Some(data) = frame.split(|b| *b == b'\n').find_map(|line| line.strip_prefix(b"data: ")) { handle_event(app, input.id, data)?; } } }
                    Some(Err(_)) => return Err("OpenAI response stream failed".into()),
                    None => return Ok(()),
                }
            }
        }
    };
    match tokio::time::timeout(Duration::from_millis(input.timeout_ms), operation).await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(message)) => Err(StreamEvent::Failed {
            request_id: input.id.into(),
            outcome: "failed",
            message,
        }),
        Err(_) => Err(StreamEvent::Failed {
            request_id: input.id.into(),
            outcome: "unavailable",
            message: "OpenAI request timed out; no provider success was recorded.".into(),
        }),
    }
}

fn handle_event(app: &AppHandle, id: &str, data: &[u8]) -> Result<(), String> {
    let value: serde_json::Value =
        serde_json::from_slice(data).map_err(|_| "OpenAI returned an invalid stream event")?;
    match value["type"].as_str() {
        Some("response.output_text.delta") => {
            if let Some(delta) = value["delta"].as_str() {
                let _ = app.emit(
                    STREAM_EVENT,
                    StreamEvent::Chunk {
                        request_id: id.into(),
                        content: delta.into(),
                    },
                );
            }
        }
        Some("response.completed") => {
            let _ = app.emit(
                STREAM_EVENT,
                StreamEvent::Finished {
                    request_id: id.into(),
                    outcome: "succeeded",
                },
            );
        }
        Some("response.failed") | Some("error") => {
            return Err("OpenAI reported a failed response".into());
        }
        _ => {}
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn config_defaults_disabled() {
        let config = CloudConfig::default();
        assert!(!config.enabled);
        assert!(config.model.is_empty());
    }
    #[test]
    fn response_body_disables_provider_storage() {
        let value = serde_json::to_value(ResponseBody {
            model: "test",
            input: "hello",
            stream: true,
            store: false,
        })
        .unwrap();
        assert_eq!(value["store"], false);
        assert_eq!(value["stream"], true);
    }
    #[test]
    fn disclosure_names_destination_and_billing() {
        let disclosure = CloudDisclosure {
            destination: "OpenAI API (api.openai.com)",
            data: ["message", "model"],
            storage_requested: false,
            billing: "API usage may incur charges",
        };
        let value = serde_json::to_value(disclosure).unwrap();
        assert_eq!(value["destination"], "OpenAI API (api.openai.com)");
        assert_eq!(value["storageRequested"], false);
    }
}
