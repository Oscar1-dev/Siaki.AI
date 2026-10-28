import assert from "node:assert/strict";
import { test } from "node:test";
import { IPC_COMMANDS, isBrowserStatus, isBrowserTask, isBrowserStepResult, isCitationList, isCloudStatus, isCloudStreamEvent, isCommandResult, isConversationView, isDecision, isDevToolsStatus, isDocumentSummary, isExpressionEvent, isGitStatus, isLipSync, isMemoryClaim, isMemoryExport, isOllamaModelList, isOllamaStreamEvent, isOverlayStatus, isPlanningSnapshot, isPolicyState, isReminderNotification, isSchedulerSnapshot, isShellStatus, isSupervisorSnapshot, isSysToolsStatus, isWorldEvent, isWorldSnapshot, isWriteResult, isVoiceStatus, isWorkspaceState, validateFrontendLog } from "../src/index.ts";

test("command names are stable and explicit", () => {
  assert.deepEqual(Object.values(IPC_COMMANDS), ["get_shell_status", "write_frontend_log", "request_app_exit", "get_overlay_status", "move_overlay_to_monitor", "set_overlay_paused", "get_workspace_state", "set_workspace_mode", "set_ui_preferences", "discover_ollama_models", "start_ollama_chat", "cancel_ollama_chat", "get_cloud_status", "import_openai_key_from_environment", "delete_openai_key", "configure_cloud_provider", "start_cloud_chat", "cancel_cloud_chat", "get_conversation", "append_owner_message", "append_saiki_message", "record_tool_result", "clear_conversation", "compose_conversation_context", "teach_memory", "correct_memory", "forget_memory", "search_memory", "list_memory", "export_memory", "ingest_document", "list_documents", "forget_document", "retrieve_knowledge", "get_voice_status", "configure_voice", "preview_lip_sync", "start_listening", "stop_listening", "interrupt_voice", "cancel_voice", "get_policy_state", "evaluate_action", "create_grant", "revoke_grant", "request_approval", "resolve_approval", "set_kill_switch", "list_policy_audit", "get_systools_status", "configure_workspace", "add_allowed_app", "remove_allowed_app", "launch_app", "read_workspace_file", "list_workspace_dir", "write_workspace_file", "rollback_write", "get_devtools_status", "configure_devtools", "add_allowed_command", "remove_allowed_command", "git_status", "git_diff", "run_command", "cancel_command", "open_in_vscode", "get_browser_status", "create_browser_task", "checkpoint_browser_task", "list_browser_tasks", "evaluate_browser_step", "get_scheduler_snapshot", "create_scheduled_task", "cancel_scheduled_task", "acknowledge_notification", "get_world_snapshot", "list_world_events", "create_world_project", "transition_world_project", "get_supervisor_snapshot", "create_supervisor_goal", "set_supervisor_paused", "get_planning_snapshot", "create_planned_goal", "stop_planned_goal"]);
});

test("planning contract preserves reasons, evidence, and bounded capability",()=>{assert.equal(isPlanningSnapshot({goals:[],plans:[],tasks:[{selectionReason:"lowest risk",evidence:{consistent:true}}],reviews:[{verdict:"stop_success",evidenceDigest:"sha256:x"}],capability:"internal_world_review_only"}),true);assert.equal(isPlanningSnapshot({goals:[],plans:[],tasks:[],reviews:[],capability:"shell_exec"}),false);});

test("supervisor contract fixes quotas and internal-only capability",()=>{
 assert.equal(isSupervisorSnapshot({paused:false,goals:[],tasks:[],tasksPerWake:3,tasksPerHour:20,capability:"internal_world_integrity_audit_only"}),true);
 assert.equal(isSupervisorSnapshot({paused:false,goals:[],tasks:[],tasksPerWake:999,tasksPerHour:999,capability:"shell_exec"}),false);
});

test("world contracts preserve versioned owner events and consistency",()=>{
 const project={id:"p",title:"SAIKI",summary:"Build",lifecycle:"active",version:2,createdAtUnixMs:1,updatedAtUnixMs:2};
 assert.equal(isWorldSnapshot({projects:[project],eventCount:2,consistent:true,inconsistencies:[]}),true);
 assert.equal(isWorldEvent({id:"e",entityId:"p",sequence:2,entityVersion:2,eventType:"project_transitioned",payload:{},actor:"owner",occurredAtUnixMs:2}),true);
 assert.equal(isWorldEvent({id:"e",entityId:"p",sequence:2,entityVersion:2,eventType:"x",payload:{},actor:"model",occurredAtUnixMs:2}),false);
});

test("scheduler contracts expose durable wake and retry semantics", () => {
  const notification = { id:"n", taskId:"t", occurrenceAtUnixMs:1, title:"Reminder", message:"Move", deliveryState:"pending", deliveryAttempts:1, createdAtUnixMs:1 };
  assert.equal(isReminderNotification(notification), true);
  assert.equal(isSchedulerSnapshot({ tasks:[], notifications:[notification], wakeTrigger:"native_core_1s_tick", retryPolicy:"durable_exponential_backoff_max_8" }), true);
  assert.equal(isSchedulerSnapshot({ tasks:[], notifications:[], wakeTrigger:"renderer_timer", retryPolicy:"none" }), false);
});

test("browser contracts preserve untrusted provenance and never claim execution", () => {
  assert.equal(isBrowserStatus({ availability: "unavailable", reason: "missing", sidecarPresent: false, enginePresent: false, profileIsolation: "dedicated_non_owner_profile", contentTrust: "untrusted_web_content" }), true);
  assert.equal(isBrowserTask({ id: "t", objective: "research", startUrl: "https://example.com", state: "checkpointed", createdAtUnixMs: 1, checkpoints: [{ id: "c", sequence: 1, url: "https://example.com", title: "Example", contentDigest: "sha256:abc", trust: "untrusted_web_content", createdAtUnixMs: 2 }] }), true);
  assert.equal(isBrowserStepResult({ outcome: "denied", reason: "untrusted", executed: false }), true);
  assert.equal(isBrowserStepResult({ outcome: "succeeded", reason: "page said so", executed: true }), false);
});

test("devtools contracts validate status, git status, and command results", () => {
  const status = { repoRoot: null, repoConfigured: false, isGitRepo: false, allowedCommands: [], vscodeConfigured: false, vscodePresent: false, codex: { available: false, reason: "none" } };
  assert.equal(isDevToolsStatus(status), true);
  assert.equal(isDevToolsStatus({ ...status, codex: { available: false } }), false);
  assert.equal(isGitStatus({ branch: "main", clean: true, entries: [] }), true);
  assert.equal(isCommandResult({ requestId: "r", command: "git", outcome: "denied", exitCode: null, stdout: "", stderr: "owner approval required", truncated: false }), true);
  assert.equal(isCommandResult({ requestId: "r", command: "git", outcome: "yolo", exitCode: null, stdout: "", stderr: "", truncated: false }), false);
});

test("system-tools contracts validate status and write results", () => {
  const status = { workspaceRoot: null, workspaceConfigured: false, workspacePresent: false, allowedApps: [], metrics: { cpuCount: 8, os: "windows", arch: "x86_64", pointerWidthBits: 64 } };
  assert.equal(isSysToolsStatus(status), true);
  assert.equal(isSysToolsStatus({ ...status, metrics: { cpuCount: "many" } }), false);
  assert.equal(isWriteResult({ path: "a.txt", bytes: 3, previousExisted: true, rollbackToken: "t" }), true);
  assert.equal(isWriteResult({ path: "a.txt", bytes: 3, previousExisted: true }), false);
});

test("policy decisions and state validate; model output cannot be an allow by shape alone", () => {
  assert.equal(isDecision({ decision: "allow" }), true);
  assert.equal(isDecision({ decision: "deny", reason: "no grant" }), true);
  assert.equal(isDecision({ decision: "approval_required", tier: "critical", reason: "owner must approve" }), true);
  assert.equal(isDecision({ decision: "approval_required", tier: "superuser", reason: "x" }), false);
  assert.equal(isDecision({ decision: "allowed" }), false);
  assert.equal(isPolicyState({ killSwitchEngaged: false, grants: [], pendingApprovals: [] }), true);
  assert.equal(isPolicyState({ killSwitchEngaged: false, grants: [{ id: "g", tier: "bogus" }], pendingApprovals: [] }), false);
});

test("voice contracts report honest capabilities and a lip-sync timeline", () => {
  const status = { sttModel: { configured: false, present: false, path: null }, ttsVoice: { configured: false, present: false, path: null }, microphone: { available: false, reason: "no capture" }, playback: { available: false, reason: "no playback" }, listeningAvailable: false, speechAudioAvailable: false, lipSyncAvailable: true, session: "idle" };
  assert.equal(isVoiceStatus(status), true);
  assert.equal(isVoiceStatus({ ...status, session: "chatting" }), false);
  const lip = { visemes: [{ viseme: "pp", startMs: 0, durationMs: 60 }], estimatedDurationMs: 60, audioOutcome: "unavailable", audioReason: "no tts" };
  assert.equal(isLipSync(lip), true);
  assert.equal(isLipSync({ ...lip, visemes: [{ viseme: "zz", startMs: 0, durationMs: 60 }] }), false);
});

test("knowledge contracts validate documents and citations", () => {
  const doc = { id: "d1", title: "handbook", path: "/docs/handbook.txt", sensitivity: "private", byteLen: 100, chunkCount: 2, ingestedAtUnixMs: 1 };
  assert.equal(isDocumentSummary(doc), true);
  assert.equal(isDocumentSummary({ ...doc, sensitivity: "classified" }), false);
  const citation = { documentId: "d1", title: "handbook", path: "/docs/handbook.txt", sensitivity: "public", chunkIndex: 0, score: 0.42, snippet: "…" };
  assert.equal(isCitationList([citation]), true);
  assert.equal(isCitationList([{ ...citation, score: "high" }]), false);
});

test("memory contracts validate claims, provenance, and export shape", () => {
  const claim = { id: "m1", claim: "ships monday", source: "owner", confidence: 0.9, sensitivity: "private", status: "active", supersedes: null, createdAtUnixMs: 1, updatedAtUnixMs: 2 };
  assert.equal(isMemoryClaim(claim), true);
  assert.equal(isMemoryClaim({ ...claim, sensitivity: "classified" }), false);
  assert.equal(isMemoryClaim({ ...claim, status: "approved" }), false);
  assert.equal(isMemoryClaim({ ...claim, supersedes: "m0" }), true);
  assert.equal(isMemoryExport({ schemaVersion: 1, claims: [claim], events: [] }), true);
  assert.equal(isMemoryExport({ schemaVersion: 1, claims: [{ ...claim, confidence: "high" }], events: [] }), false);
});

test("conversation contracts carry a non-authoritative persona and typed messages", () => {
  const view = { persona: { name: "Saiki", authority: "none", tone: ["dry"], preamble: "tone only", expressions: ["neutral", "talking"] }, messages: [{ id: "1", role: "tool_result", content: "exit code 1", createdAtUnixMs: 1 }, { id: "2", role: "saiki", content: "Reported.", createdAtUnixMs: 2, expression: "talking" }], limits: { maxStoredMessages: 200, maxContextMessages: 24, maxContextBytes: 24576, maxContentBytes: 32768 } };
  assert.equal(isConversationView(view), true);
  assert.equal(isConversationView({ ...view, persona: { ...view.persona, authority: "owner" } }), false);
  assert.equal(isConversationView({ ...view, messages: [{ id: "3", role: "approved", content: "x", createdAtUnixMs: 3 }] }), false);
  assert.equal(isExpressionEvent({ expression: "annoyed" }), true);
  assert.equal(isExpressionEvent({ expression: "authorized" }), false);
});

test("cloud contracts require disclosure and explicit outcomes", () => {
  const status = { enabled: false, model: "", credentialPresent: false, credentialStore: "windows_credential_manager", endpoint: "https://api.openai.com/v1/responses", disclosure: { destination: "OpenAI API", data: ["message", "model"], storageRequested: false, billing: "API billing applies" } };
  assert.equal(isCloudStatus(status), true);
  assert.equal(isCloudStatus({ ...status, credentialStore: "renderer_local_storage" }), false);
  assert.equal(isCloudStreamEvent({ kind: "finished", request_id: "id", outcome: "succeeded" }), true);
  assert.equal(isCloudStreamEvent({ kind: "finished", request_id: "id", outcome: "authorized" }), false);
});

test("Ollama contracts reject malformed discovery and trusted tool results", () => {
  assert.equal(isOllamaModelList([{ name: "qwen3:4b", size: 1, digest: "abc", modifiedAt: "now" }]), true);
  assert.equal(isOllamaModelList([{ name: "qwen3:4b" }]), false);
  assert.equal(isOllamaStreamEvent({ kind: "tool_proposal", request_id: "id", proposal: { name: "write", arguments: {}, trust: "untrusted_model_output" } }), true);
  assert.equal(isOllamaStreamEvent({ kind: "tool_proposal", request_id: "id", proposal: { name: "write", arguments: {}, trust: "authorized" } }), false);
});

test("overlay status requires monitor and DPI metadata", () => {
  assert.equal(isOverlayStatus({ paused: false, activeMonitorIndex: 0, pauseHotkey: "Ctrl+Shift+F12", monitors: [] }), true);
  assert.equal(isOverlayStatus({ paused: false, activeMonitorIndex: 0, pauseHotkey: "F12", monitors: [] }), false);
});

test("shell status validator rejects incomplete values", () => {
  assert.equal(isShellStatus({ appName: "SAIKI" }), false);
  assert.equal(isShellStatus({ appName: "SAIKI", version: "0.1.0", platform: "windows", lifecycle: "ready", sessionId: "s", startedAtUnixMs: 1 }), true);
});

test("frontend log limits reject oversized model-controlled text", () => {
  assert.throws(() => validateFrontendLog({ level: "info", event: "x", message: "x".repeat(501) }));
  assert.doesNotThrow(() => validateFrontendLog({ level: "info", event: "shell.ready", message: "Renderer ready" }));
});

test("workspace state requires an honest unavailable approval inbox", () => {
  assert.equal(isWorkspaceState({ mode: "full", reducedMotion: false, speechBubbles: true, approvals: { availability: "unavailable", reason: "Policy executor is not implemented.", pending: [] } }), true);
  assert.equal(isWorkspaceState({ mode: "full", reducedMotion: false, speechBubbles: true, approvals: { availability: "available", pending: [{ id: "fake" }] } }), false);
});
