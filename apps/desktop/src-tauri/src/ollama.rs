use futures_util::StreamExt;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Duration,
};
use tauri::{AppHandle, Emitter, Manager, State};

const BASE_URL: &str = "http://127.0.0.1:11434";
const MAX_PROMPT_BYTES: usize = 32 * 1024;
const MAX_TOOLS: usize = 16;
pub const STREAM_EVENT: &str = "saiki://ollama-stream";

pub struct OllamaState {
    client: reqwest::Client,
    active: Mutex<HashMap<String, Arc<tokio::sync::Notify>>>,
}

impl OllamaState {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::builder()
                .connect_timeout(Duration::from_secs(2))
                .build()
                .expect("Ollama client"),
            active: Mutex::new(HashMap::new()),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaModel {
    pub name: String,
    pub size: u64,
    pub digest: String,
    #[serde(rename(serialize = "modifiedAt", deserialize = "modified_at"))]
    pub modified_at: String,
}

#[derive(Deserialize)]
struct TagsResponse {
    models: Vec<OllamaModel>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ToolDefinition {
    pub name: String,
    pub description: String,
    pub parameters: serde_json::Value,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ChatRequest {
    pub request_id: String,
    pub model: String,
    pub prompt: String,
    #[serde(default)]
    pub tools: Vec<ToolDefinition>,
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,
}
fn default_timeout() -> u64 {
    120_000
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChatStarted {
    pub request_id: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum StreamEvent {
    Chunk {
        request_id: String,
        content: String,
    },
    ToolProposal {
        request_id: String,
        proposal: ToolProposal,
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

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct ToolProposal {
    pub name: String,
    pub arguments: serde_json::Value,
    pub trust: &'static str,
}

#[derive(Serialize)]
struct ApiChat<'a> {
    model: &'a str,
    messages: [ApiMessage<'a>; 1],
    stream: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tools: Vec<ApiTool<'a>>,
}
#[derive(Serialize)]
struct ApiMessage<'a> {
    role: &'static str,
    content: &'a str,
}
#[derive(Serialize)]
struct ApiTool<'a> {
    r#type: &'static str,
    function: ApiFunction<'a>,
}
#[derive(Serialize)]
struct ApiFunction<'a> {
    name: &'a str,
    description: &'a str,
    parameters: &'a serde_json::Value,
}
#[derive(Deserialize)]
struct ApiChunk {
    #[serde(default)]
    message: ApiChunkMessage,
    #[serde(default)]
    done: bool,
}
#[derive(Default, Deserialize)]
struct ApiChunkMessage {
    #[serde(default)]
    content: String,
    #[serde(default)]
    tool_calls: Vec<ApiToolCall>,
}
#[derive(Deserialize)]
struct ApiToolCall {
    function: ApiCalledFunction,
}
#[derive(Deserialize)]
struct ApiCalledFunction {
    name: String,
    #[serde(default)]
    arguments: serde_json::Value,
}

fn validate(request: &ChatRequest) -> Result<(), String> {
    if uuid::Uuid::parse_str(&request.request_id).is_err() {
        return Err("requestId must be a UUID".into());
    }
    if request.model.trim().is_empty() || request.model.len() > 200 {
        return Err("model must be 1-200 characters".into());
    }
    if request.prompt.trim().is_empty() || request.prompt.len() > MAX_PROMPT_BYTES {
        return Err("prompt must be 1-32768 bytes".into());
    }
    if request.tools.len() > MAX_TOOLS {
        return Err("at most 16 tool definitions are allowed".into());
    }
    if !(1_000..=600_000).contains(&request.timeout_ms) {
        return Err("timeoutMs must be between 1000 and 600000".into());
    }
    for tool in &request.tools {
        if tool.name.is_empty()
            || tool.name.len() > 100
            || tool.description.len() > 1000
            || !tool.parameters.is_object()
        {
            return Err("invalid tool definition".into());
        }
    }
    Ok(())
}

#[tauri::command]
pub async fn discover_ollama_models(
    state: State<'_, OllamaState>,
) -> Result<Vec<OllamaModel>, String> {
    let response = tokio::time::timeout(
        Duration::from_secs(5),
        state.client.get(format!("{BASE_URL}/api/tags")).send(),
    )
    .await
    .map_err(|_| "Ollama discovery timed out".to_string())?
    .map_err(|_| "Local Ollama is unavailable at 127.0.0.1:11434".to_string())?;
    if !response.status().is_success() {
        return Err(format!(
            "Ollama discovery failed with HTTP {}",
            response.status()
        ));
    }
    response
        .json::<TagsResponse>()
        .await
        .map(|v| v.models)
        .map_err(|_| "Ollama returned an invalid model list".into())
}

#[tauri::command]
pub async fn start_ollama_chat(
    app: AppHandle,
    state: State<'_, OllamaState>,
    request: ChatRequest,
) -> Result<ChatStarted, String> {
    validate(&request)?;
    let request_id = request.request_id.clone();
    let cancel = Arc::new(tokio::sync::Notify::new());
    {
        let mut active = state
            .active
            .lock()
            .map_err(|_| "model request state unavailable")?;
        if active.contains_key(&request_id) {
            return Err("requestId is already active".into());
        }
        active.insert(request_id.clone(), cancel.clone());
    }
    let client = state.client.clone();
    let id = request_id.clone();
    tauri::async_runtime::spawn(async move {
        let outcome = stream_chat(&app, &client, &id, &request, cancel).await;
        if let Err(event) = outcome {
            let _ = app.emit(STREAM_EVENT, event);
        }
        if let Some(state) = app.try_state::<OllamaState>() {
            if let Ok(mut active) = state.active.lock() {
                active.remove(&id);
            }
        }
    });
    Ok(ChatStarted { request_id })
}

#[tauri::command]
pub fn cancel_ollama_chat(
    state: State<'_, OllamaState>,
    request_id: String,
) -> Result<bool, String> {
    let active = state
        .active
        .lock()
        .map_err(|_| "model request state unavailable")?;
    if let Some(cancel) = active.get(&request_id) {
        cancel.notify_waiters();
        Ok(true)
    } else {
        Ok(false)
    }
}

async fn stream_chat(
    app: &AppHandle,
    client: &reqwest::Client,
    id: &str,
    request: &ChatRequest,
    cancel: Arc<tokio::sync::Notify>,
) -> Result<(), StreamEvent> {
    let tools = request
        .tools
        .iter()
        .map(|t| ApiTool {
            r#type: "function",
            function: ApiFunction {
                name: &t.name,
                description: &t.description,
                parameters: &t.parameters,
            },
        })
        .collect();
    let body = ApiChat {
        model: &request.model,
        messages: [ApiMessage {
            role: "user",
            content: &request.prompt,
        }],
        stream: true,
        tools,
    };
    let operation = async {
        let response = client
            .post(format!("{BASE_URL}/api/chat"))
            .json(&body)
            .send()
            .await
            .map_err(|_| "Local Ollama connection failed".to_string())?;
        if !response.status().is_success() {
            return Err(format!(
                "Ollama chat failed with HTTP {}",
                response.status()
            ));
        }
        let mut bytes = response.bytes_stream();
        let mut pending = Vec::new();
        loop {
            tokio::select! {
                _ = cancel.notified() => { let _ = app.emit(STREAM_EVENT, StreamEvent::Finished { request_id: id.into(), outcome: "cancelled" }); return Ok(()); }
                next = bytes.next() => match next {
                    Some(Ok(part)) => { pending.extend_from_slice(&part); while let Some(pos) = pending.iter().position(|b| *b == b'\n') { let line: Vec<u8> = pending.drain(..=pos).collect(); if line.iter().all(|b| b.is_ascii_whitespace()) { continue; } let chunk: ApiChunk = serde_json::from_slice(&line).map_err(|_| "Ollama returned an invalid stream frame".to_string())?; emit_chunk(app, id, chunk); } }
                    Some(Err(_)) => return Err("Ollama stream failed".into()),
                    None => return Ok(()),
                }
            }
        }
    };
    match tokio::time::timeout(Duration::from_millis(request.timeout_ms), operation).await {
        Ok(Ok(())) => Ok(()),
        Ok(Err(message)) => Err(StreamEvent::Failed {
            request_id: id.into(),
            outcome: "failed",
            message,
        }),
        Err(_) => Err(timeout_failure(id)),
    }
}

fn timeout_failure(request_id: &str) -> StreamEvent {
    StreamEvent::Failed {
        request_id: request_id.into(),
        outcome: "unavailable",
        message: "Local model request timed out; no cloud fallback was attempted.".into(),
    }
}

fn emit_chunk(app: &AppHandle, id: &str, chunk: ApiChunk) {
    if !chunk.message.content.is_empty() {
        let _ = app.emit(
            STREAM_EVENT,
            StreamEvent::Chunk {
                request_id: id.into(),
                content: chunk.message.content,
            },
        );
    }
    for call in chunk.message.tool_calls {
        let _ = app.emit(
            STREAM_EVENT,
            StreamEvent::ToolProposal {
                request_id: id.into(),
                proposal: ToolProposal {
                    name: call.function.name,
                    arguments: call.function.arguments,
                    trust: "untrusted_model_output",
                },
            },
        );
    }
    if chunk.done {
        let _ = app.emit(
            STREAM_EVENT,
            StreamEvent::Finished {
                request_id: id.into(),
                outcome: "succeeded",
            },
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_oversized_and_short_timeout_requests() {
        let request = ChatRequest {
            request_id: uuid::Uuid::new_v4().to_string(),
            model: "qwen".into(),
            prompt: "x".repeat(MAX_PROMPT_BYTES + 1),
            tools: vec![],
            timeout_ms: 999,
        };
        assert!(validate(&request).is_err());
    }
    #[test]
    fn tool_proposal_is_explicitly_untrusted() {
        let proposal = ToolProposal {
            name: "read".into(),
            arguments: serde_json::json!({"path":"x"}),
            trust: "untrusted_model_output",
        };
        assert_eq!(
            serde_json::to_value(proposal).unwrap()["trust"],
            "untrusted_model_output"
        );
    }

    #[test]
    fn timeout_is_unavailable_and_never_claims_fallback() {
        let value = serde_json::to_value(timeout_failure("request")).unwrap();
        assert_eq!(value["outcome"], "unavailable");
        assert_eq!(
            value["message"],
            "Local model request timed out; no cloud fallback was attempted."
        );
    }
}
