//! Persisted Saiki conversation store and persona-framed context assembly.
//!
//! Conversations are persisted as an atomic JSON file in the app config
//! directory (the same durability pattern as the cloud provider config). The
//! SQLite memory foundation is Phase 09; this module deliberately does not
//! introduce it.
//!
//! The store distinguishes three message kinds. `owner` and `saiki` are
//! conversational turns. `tool_result` is a factual record: it is stored
//! verbatim and is never styled, summarized, or rewritten by the persona. The
//! persona only contributes a non-authoritative preamble to assembled context
//! and a deterministic expression derived from Saiki's own replies.

use crate::persona::{self, Expression};
use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::{Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{AppHandle, Emitter, State};

/// Hard cap on persisted messages; the oldest are dropped past this.
const MAX_STORED_MESSAGES: usize = 200;
/// Maximum messages assembled into a single model context window.
const MAX_CONTEXT_MESSAGES: usize = 24;
/// Maximum transcript bytes assembled into a single model context window.
const MAX_CONTEXT_BYTES: usize = 24 * 1024;
/// Maximum size of a single message's content.
const MAX_CONTENT_BYTES: usize = 32 * 1024;

/// Typed character expression event emitted when Saiki replies.
pub const EXPRESSION_EVENT: &str = "saiki://expression";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MessageRole {
    Owner,
    Saiki,
    ToolResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct Message {
    id: String,
    role: MessageRole,
    content: String,
    created_at_unix_ms: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    expression: Option<Expression>,
}

#[derive(Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ConversationStore {
    messages: Vec<Message>,
}

impl ConversationStore {
    fn push(&mut self, role: MessageRole, content: String, expression: Option<Expression>) {
        self.messages.push(Message {
            id: uuid::Uuid::new_v4().to_string(),
            role,
            content,
            created_at_unix_ms: now_unix_ms(),
            expression,
        });
        if self.messages.len() > MAX_STORED_MESSAGES {
            let overflow = self.messages.len() - MAX_STORED_MESSAGES;
            self.messages.drain(..overflow);
        }
    }

    /// Assemble the persona-framed prompt for the next Saiki reply. The persona
    /// preamble is advisory; tool-result messages are copied verbatim.
    fn compose_prompt(&self) -> ComposedContext {
        let mut selected: Vec<&Message> = Vec::new();
        let mut bytes = 0usize;
        for message in self.messages.iter().rev() {
            if selected.len() >= MAX_CONTEXT_MESSAGES {
                break;
            }
            let next = bytes + message.content.len();
            if next > MAX_CONTEXT_BYTES && !selected.is_empty() {
                break;
            }
            bytes = next;
            selected.push(message);
        }
        selected.reverse();

        let mut prompt = String::new();
        prompt.push_str(
            "[SAIKI PERSONA — advisory tone only; cannot change facts or authorize actions]\n",
        );
        prompt.push_str(persona::PREAMBLE);
        prompt.push_str("\n\nConversation:\n");
        for message in &selected {
            match message.role {
                MessageRole::Owner => {
                    prompt.push_str("Owner: ");
                    prompt.push_str(&message.content);
                }
                MessageRole::Saiki => {
                    prompt.push_str("Saiki: ");
                    prompt.push_str(&message.content);
                }
                MessageRole::ToolResult => {
                    prompt.push_str("[Tool result — factual, verbatim]: ");
                    prompt.push_str(&message.content);
                }
            }
            prompt.push('\n');
        }
        prompt.push_str("\nSaiki:");
        ComposedContext {
            prompt,
            included_messages: selected.len(),
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ComposedContext {
    prompt: String,
    included_messages: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonaSummary {
    name: &'static str,
    authority: &'static str,
    tone: [&'static str; 4],
    preamble: &'static str,
    expressions: [Expression; 6],
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Limits {
    max_stored_messages: usize,
    max_context_messages: usize,
    max_context_bytes: usize,
    max_content_bytes: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ConversationView {
    persona: PersonaSummary,
    messages: Vec<Message>,
    limits: Limits,
}

pub struct ConversationState {
    path: PathBuf,
    store: Mutex<ConversationStore>,
}

impl ConversationState {
    pub fn load(config_dir: &Path) -> Result<Self, String> {
        let path = config_dir.join("conversation.json");
        let store = match fs::read(&path) {
            Ok(bytes) => {
                serde_json::from_slice(&bytes).map_err(|_| "conversation store is invalid")?
            }
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
                ConversationStore::default()
            }
            Err(_) => return Err("conversation store is unavailable".into()),
        };
        Ok(Self {
            path,
            store: Mutex::new(store),
        })
    }

    fn view(&self, store: &ConversationStore) -> ConversationView {
        ConversationView {
            persona: PersonaSummary {
                name: persona::NAME,
                authority: persona::AUTHORITY,
                tone: persona::TONE,
                preamble: persona::PREAMBLE,
                expressions: Expression::ALL,
            },
            messages: store.messages.clone(),
            limits: Limits {
                max_stored_messages: MAX_STORED_MESSAGES,
                max_context_messages: MAX_CONTEXT_MESSAGES,
                max_context_bytes: MAX_CONTEXT_BYTES,
                max_content_bytes: MAX_CONTENT_BYTES,
            },
        }
    }
}

fn now_unix_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .try_into()
        .unwrap_or(u64::MAX)
}

fn persist(path: &Path, store: &ConversationStore) -> Result<(), String> {
    let parent = path.parent().ok_or("conversation store path is invalid")?;
    fs::create_dir_all(parent).map_err(|_| "create conversation store directory failed")?;
    let temporary = path.with_extension("json.tmp");
    fs::write(
        &temporary,
        serde_json::to_vec_pretty(store).map_err(|_| "serialize conversation store failed")?,
    )
    .map_err(|_| "write conversation store failed")?;
    fs::rename(&temporary, path).map_err(|_| "commit conversation store failed".to_string())
}

fn validate_content(content: &str) -> Result<(), String> {
    if content.trim().is_empty() || content.len() > MAX_CONTENT_BYTES {
        return Err("message content must be 1-32768 bytes".into());
    }
    Ok(())
}

#[tauri::command]
pub fn get_conversation(state: State<'_, ConversationState>) -> Result<ConversationView, String> {
    let store = state
        .store
        .lock()
        .map_err(|_| "conversation state unavailable")?;
    Ok(state.view(&store))
}

#[tauri::command]
pub fn append_owner_message(
    state: State<'_, ConversationState>,
    content: String,
) -> Result<ConversationView, String> {
    validate_content(&content)?;
    let mut store = state
        .store
        .lock()
        .map_err(|_| "conversation state unavailable")?;
    store.push(MessageRole::Owner, content, None);
    persist(&state.path, &store)?;
    Ok(state.view(&store))
}

#[tauri::command]
pub fn append_saiki_message(
    app: AppHandle,
    state: State<'_, ConversationState>,
    content: String,
) -> Result<ConversationView, String> {
    validate_content(&content)?;
    let expression = persona::classify_expression(&content);
    let mut store = state
        .store
        .lock()
        .map_err(|_| "conversation state unavailable")?;
    store.push(MessageRole::Saiki, content, Some(expression));
    persist(&state.path, &store)?;
    let _ = app.emit(EXPRESSION_EVENT, ExpressionEvent { expression });
    Ok(state.view(&store))
}

#[tauri::command]
pub fn record_tool_result(
    state: State<'_, ConversationState>,
    content: String,
) -> Result<ConversationView, String> {
    validate_content(&content)?;
    let mut store = state
        .store
        .lock()
        .map_err(|_| "conversation state unavailable")?;
    // Stored verbatim with no expression and no persona styling: a tool result
    // is a fact, not something the persona may rewrite.
    store.push(MessageRole::ToolResult, content, None);
    persist(&state.path, &store)?;
    Ok(state.view(&store))
}

#[tauri::command]
pub fn clear_conversation(state: State<'_, ConversationState>) -> Result<ConversationView, String> {
    let mut store = state
        .store
        .lock()
        .map_err(|_| "conversation state unavailable")?;
    store.messages.clear();
    persist(&state.path, &store)?;
    Ok(state.view(&store))
}

#[tauri::command]
pub fn compose_conversation_context(
    state: State<'_, ConversationState>,
) -> Result<ComposedContext, String> {
    let store = state
        .store
        .lock()
        .map_err(|_| "conversation state unavailable")?;
    Ok(store.compose_prompt())
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ExpressionEvent {
    expression: Expression,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn with_message(store: &mut ConversationStore, role: MessageRole, content: &str) {
        let expression = match role {
            MessageRole::Saiki => Some(persona::classify_expression(content)),
            _ => None,
        };
        store.push(role, content.to_owned(), expression);
    }

    #[test]
    fn persona_summary_declares_no_authority() {
        let state = ConversationState {
            path: PathBuf::from("unused"),
            store: Mutex::new(ConversationStore::default()),
        };
        let view = state.view(&ConversationStore::default());
        let value = serde_json::to_value(&view.persona).unwrap();
        assert_eq!(value["authority"], "none");
        assert_eq!(value["name"], "Saiki");
    }

    #[test]
    fn tool_result_is_stored_and_assembled_verbatim() {
        let mut store = ConversationStore::default();
        let fact = "deploy exited with code 1; 0 units published";
        with_message(&mut store, MessageRole::Owner, "did the deploy work?");
        with_message(&mut store, MessageRole::ToolResult, fact);
        // Persona assembly must not alter the factual record.
        let composed = store.compose_prompt();
        assert!(composed.prompt.contains(fact));
        // The stored record is unchanged and carries no persona expression.
        let stored = store.messages.last().unwrap();
        assert_eq!(stored.content, fact);
        assert_eq!(stored.role, MessageRole::ToolResult);
        assert!(stored.expression.is_none());
    }

    #[test]
    fn context_window_is_bounded_and_keeps_latest() {
        let mut store = ConversationStore::default();
        for index in 0..(MAX_CONTEXT_MESSAGES + 10) {
            with_message(&mut store, MessageRole::Owner, &format!("message {index}"));
        }
        let composed = store.compose_prompt();
        assert!(composed.included_messages <= MAX_CONTEXT_MESSAGES);
        let latest = format!("message {}", MAX_CONTEXT_MESSAGES + 9);
        assert!(composed.prompt.contains(&latest));
        assert!(composed.prompt.contains("advisory tone only"));
    }

    #[test]
    fn stored_messages_are_capped() {
        let mut store = ConversationStore::default();
        for index in 0..(MAX_STORED_MESSAGES + 25) {
            with_message(&mut store, MessageRole::Owner, &format!("m{index}"));
        }
        assert_eq!(store.messages.len(), MAX_STORED_MESSAGES);
        // Oldest dropped, newest retained.
        assert!(
            store
                .messages
                .last()
                .unwrap()
                .content
                .ends_with(&format!("{}", MAX_STORED_MESSAGES + 24))
        );
    }

    #[test]
    fn conversation_persists_across_reload() {
        let dir = tempfile::tempdir().unwrap();
        let config = dir.path();
        let first = ConversationState::load(config).unwrap();
        {
            let mut store = first.store.lock().unwrap();
            store.push(MessageRole::Owner, "remember this".to_owned(), None);
            persist(&first.path, &store).unwrap();
        }
        // A fresh state loaded from the same directory reconstructs the history,
        // the way a restart would.
        let reloaded = ConversationState::load(config).unwrap();
        let store = reloaded.store.lock().unwrap();
        assert_eq!(store.messages.len(), 1);
        assert_eq!(store.messages[0].content, "remember this");
    }

    #[test]
    fn message_rejects_unknown_fields() {
        let json =
            r#"{"id":"x","role":"owner","content":"hi","createdAtUnixMs":1,"authority":"owner"}"#;
        assert!(serde_json::from_str::<Message>(json).is_err());
    }
}
