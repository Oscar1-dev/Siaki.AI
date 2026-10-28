export const IPC_COMMANDS = {
  getShellStatus: "get_shell_status",
  writeFrontendLog: "write_frontend_log",
  requestAppExit: "request_app_exit",
  getOverlayStatus: "get_overlay_status",
  moveOverlayToMonitor: "move_overlay_to_monitor",
  setOverlayPaused: "set_overlay_paused",
  getWorkspaceState: "get_workspace_state",
  setWorkspaceMode: "set_workspace_mode",
  setUiPreferences: "set_ui_preferences",
  discoverOllamaModels: "discover_ollama_models",
  startOllamaChat: "start_ollama_chat",
  cancelOllamaChat: "cancel_ollama_chat",
  getCloudStatus: "get_cloud_status",
  importOpenAiKeyFromEnvironment: "import_openai_key_from_environment",
  deleteOpenAiKey: "delete_openai_key",
  configureCloudProvider: "configure_cloud_provider",
  startCloudChat: "start_cloud_chat",
  cancelCloudChat: "cancel_cloud_chat",
  getConversation: "get_conversation",
  appendOwnerMessage: "append_owner_message",
  appendSaikiMessage: "append_saiki_message",
  recordToolResult: "record_tool_result",
  clearConversation: "clear_conversation",
  composeConversationContext: "compose_conversation_context",
  teachMemory: "teach_memory",
  correctMemory: "correct_memory",
  forgetMemory: "forget_memory",
  searchMemory: "search_memory",
  listMemory: "list_memory",
  exportMemory: "export_memory",
  ingestDocument: "ingest_document",
  listDocuments: "list_documents",
  forgetDocument: "forget_document",
  retrieveKnowledge: "retrieve_knowledge",
  getVoiceStatus: "get_voice_status",
  configureVoice: "configure_voice",
  previewLipSync: "preview_lip_sync",
  startListening: "start_listening",
  stopListening: "stop_listening",
  interruptVoice: "interrupt_voice",
  cancelVoice: "cancel_voice",
  getPolicyState: "get_policy_state",
  evaluateAction: "evaluate_action",
  createGrant: "create_grant",
  revokeGrant: "revoke_grant",
  requestApproval: "request_approval",
  resolveApproval: "resolve_approval",
  setKillSwitch: "set_kill_switch",
  listPolicyAudit: "list_policy_audit",
  getSysToolsStatus: "get_systools_status",
  configureWorkspace: "configure_workspace",
  addAllowedApp: "add_allowed_app",
  removeAllowedApp: "remove_allowed_app",
  launchApp: "launch_app",
  readWorkspaceFile: "read_workspace_file",
  listWorkspaceDir: "list_workspace_dir",
  writeWorkspaceFile: "write_workspace_file",
  rollbackWrite: "rollback_write",
  getDevToolsStatus: "get_devtools_status",
  configureDevTools: "configure_devtools",
  addAllowedCommand: "add_allowed_command",
  removeAllowedCommand: "remove_allowed_command",
  gitStatus: "git_status",
  gitDiff: "git_diff",
  runCommand: "run_command",
  cancelCommand: "cancel_command",
  openInVscode: "open_in_vscode",
  getBrowserStatus: "get_browser_status",
  createBrowserTask: "create_browser_task",
  checkpointBrowserTask: "checkpoint_browser_task",
  listBrowserTasks: "list_browser_tasks",
  evaluateBrowserStep: "evaluate_browser_step",
  getSchedulerSnapshot: "get_scheduler_snapshot",
  createScheduledTask: "create_scheduled_task",
  cancelScheduledTask: "cancel_scheduled_task",
  acknowledgeNotification: "acknowledge_notification",
  getWorldSnapshot: "get_world_snapshot",
  listWorldEvents: "list_world_events",
  createWorldProject: "create_world_project",
  transitionWorldProject: "transition_world_project",
} as const;
export const REMINDER_EVENT = "saiki://reminder" as const;
export const WORLD_EVENT = "saiki://world-event" as const;

export const OLLAMA_STREAM_EVENT = "saiki://ollama-stream" as const;
export const OPENAI_STREAM_EVENT = "saiki://openai-stream" as const;
export const EXPRESSION_EVENT = "saiki://expression" as const;

export interface OllamaModel { readonly name: string; readonly size: number; readonly digest: string; readonly modifiedAt: string }
export interface ToolDefinition { readonly name: string; readonly description: string; readonly parameters: Readonly<Record<string, unknown>> }
export interface OllamaChatRequest { readonly requestId: string; readonly model: string; readonly prompt: string; readonly tools?: readonly ToolDefinition[]; readonly timeoutMs?: number }
export type OllamaStreamEvent =
  | { readonly kind: "chunk"; readonly request_id: string; readonly content: string }
  | { readonly kind: "tool_proposal"; readonly request_id: string; readonly proposal: { readonly name: string; readonly arguments: unknown; readonly trust: "untrusted_model_output" } }
  | { readonly kind: "finished"; readonly request_id: string; readonly outcome: "succeeded" | "cancelled" }
  | { readonly kind: "failed"; readonly request_id: string; readonly outcome: "failed" | "unavailable"; readonly message: string };

export interface CloudStatus {
  readonly enabled: boolean;
  readonly model: string;
  readonly credentialPresent: boolean;
  readonly credentialStore: "windows_credential_manager";
  readonly endpoint: "https://api.openai.com/v1/responses";
  readonly disclosure: { readonly destination: string; readonly data: readonly [string, string]; readonly storageRequested: false; readonly billing: string };
}
export interface ConfigureCloudRequest { readonly enabled: boolean; readonly model: string; readonly disclosureAccepted: boolean }
export interface CloudChatRequest { readonly requestId: string; readonly prompt: string; readonly timeoutMs?: number }
export type CloudStreamEvent =
  | { readonly kind: "chunk"; readonly request_id: string; readonly content: string }
  | { readonly kind: "finished"; readonly request_id: string; readonly outcome: "succeeded" | "cancelled" }
  | { readonly kind: "failed"; readonly request_id: string; readonly outcome: "failed" | "unavailable"; readonly message: string };

export type Expression = "neutral" | "talking" | "focused" | "annoyed" | "surprised" | "tired";
export type MessageRole = "owner" | "saiki" | "tool_result";
export interface ConversationMessage {
  readonly id: string;
  readonly role: MessageRole;
  readonly content: string;
  readonly createdAtUnixMs: number;
  readonly expression?: Expression;
}
export interface PersonaSummary {
  readonly name: string;
  readonly authority: "none";
  readonly tone: readonly string[];
  readonly preamble: string;
  readonly expressions: readonly Expression[];
}
export interface ConversationLimits {
  readonly maxStoredMessages: number;
  readonly maxContextMessages: number;
  readonly maxContextBytes: number;
  readonly maxContentBytes: number;
}
export interface ConversationView {
  readonly persona: PersonaSummary;
  readonly messages: readonly ConversationMessage[];
  readonly limits: ConversationLimits;
}
export interface ComposedContext {
  readonly prompt: string;
  readonly includedMessages: number;
}
export interface ExpressionEvent {
  readonly expression: Expression;
}

export type Sensitivity = "public" | "private" | "sensitive";
export type ClaimStatus = "active" | "superseded" | "forgotten";
export interface MemoryClaim {
  readonly id: string;
  readonly claim: string;
  readonly source: string;
  readonly confidence: number;
  readonly sensitivity: Sensitivity;
  readonly status: ClaimStatus;
  readonly supersedes: string | null;
  readonly createdAtUnixMs: number;
  readonly updatedAtUnixMs: number;
}
export interface MemoryEvent {
  readonly kind: string;
  readonly claimId: string;
  readonly detail: string;
  readonly atUnixMs: number;
}
export interface MemoryExport {
  readonly schemaVersion: number;
  readonly claims: readonly MemoryClaim[];
  readonly events: readonly MemoryEvent[];
}
export interface TeachMemoryRequest {
  readonly claim: string;
  readonly source: string;
  readonly confidence: number;
  readonly sensitivity: Sensitivity;
}
export interface CorrectMemoryRequest extends TeachMemoryRequest {
  readonly id: string;
}

export interface DocumentSummary {
  readonly id: string;
  readonly title: string;
  readonly path: string;
  readonly sensitivity: Sensitivity;
  readonly byteLen: number;
  readonly chunkCount: number;
  readonly ingestedAtUnixMs: number;
}
export interface Citation {
  readonly documentId: string;
  readonly title: string;
  readonly path: string;
  readonly sensitivity: Sensitivity;
  readonly chunkIndex: number;
  readonly score: number;
  readonly snippet: string;
}
export interface IngestDocumentRequest {
  readonly path: string;
  readonly sensitivity: Sensitivity;
  readonly title?: string;
}
export interface RetrieveKnowledgeRequest {
  readonly query: string;
  readonly includeSensitive?: boolean;
}

export type Viseme = "silence" | "pp" | "ff" | "th" | "dd" | "kk" | "ch" | "ss" | "nn" | "rr" | "aa" | "ee" | "ih" | "oh" | "ou";
export type VoiceSessionState = "idle" | "listening" | "speaking";
export interface VisemeFrame { readonly viseme: Viseme; readonly startMs: number; readonly durationMs: number }
export interface LipSync {
  readonly visemes: readonly VisemeFrame[];
  readonly estimatedDurationMs: number;
  readonly audioOutcome: "unavailable";
  readonly audioReason: string;
}
export interface VoiceEngineAsset { readonly configured: boolean; readonly present: boolean; readonly path: string | null }
export interface VoiceCapability { readonly available: boolean; readonly reason: string }
export interface VoiceStatus {
  readonly sttModel: VoiceEngineAsset;
  readonly ttsVoice: VoiceEngineAsset;
  readonly microphone: VoiceCapability;
  readonly playback: VoiceCapability;
  readonly listeningAvailable: boolean;
  readonly speechAudioAvailable: boolean;
  readonly lipSyncAvailable: boolean;
  readonly session: VoiceSessionState;
}
export interface ConfigureVoiceRequest { readonly sttModelPath?: string | null; readonly ttsVoicePath?: string | null }

export type RiskTier = "internal" | "low" | "external" | "financial" | "critical";
export type Decision =
  | { readonly decision: "allow" }
  | { readonly decision: "deny"; readonly reason: string }
  | { readonly decision: "approval_required"; readonly tier: RiskTier; readonly reason: string };
export interface PolicyGrant {
  readonly id: string;
  readonly capability: string;
  readonly resource: string;
  readonly purpose: string;
  readonly tier: RiskTier;
  readonly maxActions: number;
  readonly usedActions: number;
  readonly maxSpend: number | null;
  readonly spent: number;
  readonly expiresAtUnixMs: number;
  readonly createdAtUnixMs: number;
}
export interface PolicyApproval {
  readonly id: string;
  readonly actor: string;
  readonly capability: string;
  readonly resource: string;
  readonly argumentsDigest: string;
  readonly status: string;
  readonly tier: RiskTier;
  readonly requestedAtUnixMs: number;
  readonly expiresAtUnixMs: number;
}
export interface PolicyState {
  readonly killSwitchEngaged: boolean;
  readonly grants: readonly PolicyGrant[];
  readonly pendingApprovals: readonly PolicyApproval[];
}
export interface PolicyAuditEntry {
  readonly atUnixMs: number;
  readonly actor: string;
  readonly capability: string;
  readonly resource: string;
  readonly decision: string;
  readonly reason: string;
  readonly detail: string;
}
export interface EvaluateActionRequest {
  readonly actor: string;
  readonly capability: string;
  readonly resource: string;
  readonly arguments?: Readonly<Record<string, unknown>>;
}
export interface CreateGrantRequest {
  readonly capability: string;
  readonly resource: string;
  readonly purpose: string;
  readonly maxActions: number;
  readonly maxSpend?: number | null;
  readonly ttlMs: number;
}
export interface RequestApprovalRequest {
  readonly actor: string;
  readonly capability: string;
  readonly resource: string;
  readonly arguments?: Readonly<Record<string, unknown>>;
  readonly ttlMs: number;
}

export interface AllowedApp { readonly id: string; readonly name: string; readonly path: string }
export interface SystemMetrics { readonly cpuCount: number; readonly os: string; readonly arch: string; readonly pointerWidthBits: number }
export interface SysToolsStatus {
  readonly workspaceRoot: string | null;
  readonly workspaceConfigured: boolean;
  readonly workspacePresent: boolean;
  readonly allowedApps: readonly AllowedApp[];
  readonly metrics: SystemMetrics;
}
export interface DirEntryInfo { readonly name: string; readonly isDir: boolean }
export interface FileContent { readonly path: string; readonly content: string; readonly bytes: number }
export interface WriteResult { readonly path: string; readonly bytes: number; readonly previousExisted: boolean; readonly rollbackToken: string }
export interface AddAppRequest { readonly name: string; readonly path: string }
export interface WriteFileRequest { readonly path: string; readonly content: string }

export interface DevToolsStatus {
  readonly repoRoot: string | null;
  readonly repoConfigured: boolean;
  readonly isGitRepo: boolean;
  readonly allowedCommands: readonly string[];
  readonly vscodeConfigured: boolean;
  readonly vscodePresent: boolean;
  readonly codex: { readonly available: boolean; readonly reason: string };
}
export interface GitStatusEntry { readonly status: string; readonly path: string }
export interface GitStatus { readonly branch: string; readonly entries: readonly GitStatusEntry[]; readonly clean: boolean }
export interface GitDiff { readonly diff: string; readonly truncated: boolean }
export type CommandOutcome = "succeeded" | "failed" | "denied" | "cancelled" | "unavailable";
export interface CommandResult {
  readonly requestId: string;
  readonly command: string;
  readonly outcome: CommandOutcome;
  readonly exitCode: number | null;
  readonly stdout: string;
  readonly stderr: string;
  readonly truncated: boolean;
}
export interface ConfigureDevToolsRequest { readonly repoRoot?: string | null; readonly vscodePath?: string | null }
export interface RunCommandRequest { readonly requestId: string; readonly command: string; readonly args?: readonly string[]; readonly timeoutMs?: number }
export interface BrowserStatus { readonly availability: "available" | "unavailable"; readonly reason: string; readonly sidecarPresent: boolean; readonly enginePresent: boolean; readonly profileIsolation: "dedicated_non_owner_profile"; readonly contentTrust: "untrusted_web_content" }
export interface BrowserCheckpoint { readonly id: string; readonly sequence: number; readonly url: string; readonly title: string; readonly contentDigest: string; readonly trust: "untrusted_web_content"; readonly createdAtUnixMs: number }
export interface BrowserTask { readonly id: string; readonly objective: string; readonly startUrl: string; readonly state: "checkpointed"; readonly createdAtUnixMs: number; readonly checkpoints: readonly BrowserCheckpoint[] }
export interface CreateBrowserTaskRequest { readonly objective: string; readonly startUrl: string }
export interface CheckpointBrowserTaskRequest { readonly taskId: string; readonly url: string; readonly title: string; readonly pageText: string }
export type BrowserAction = "navigate" | "click" | "fill" | "extract";
export interface BrowserStepRequest { readonly taskId: string; readonly source: "owner" | "planner" | "page_content"; readonly action: BrowserAction; readonly target: string; readonly value?: string; readonly sensitive: boolean }
export interface BrowserStepResult { readonly outcome: "denied" | "approval_required" | "unavailable"; readonly reason: string; readonly executed: false }
export interface ScheduledTask { readonly id:string; readonly title:string; readonly message:string; readonly scheduleKind:"once"|"interval"; readonly nextRunAtUnixMs:number; readonly intervalMs:number|null; readonly state:"active"|"completed"|"cancelled"; readonly occurrenceCount:number; readonly createdAtUnixMs:number }
export interface ReminderNotification { readonly id:string; readonly taskId:string; readonly occurrenceAtUnixMs:number; readonly title:string; readonly message:string; readonly deliveryState:"pending"|"acknowledged"; readonly deliveryAttempts:number; readonly createdAtUnixMs:number }
export interface SchedulerSnapshot { readonly tasks:readonly ScheduledTask[]; readonly notifications:readonly ReminderNotification[]; readonly wakeTrigger:"native_core_1s_tick"; readonly retryPolicy:"durable_exponential_backoff_max_8" }
export interface CreateScheduleRequest { readonly title:string; readonly message:string; readonly runAtUnixMs:number; readonly intervalMs?:number|null }
export type ProjectLifecycle="planned"|"active"|"paused"|"completed"|"cancelled";
export interface WorldProject{readonly id:string;readonly title:string;readonly summary:string;readonly lifecycle:ProjectLifecycle;readonly version:number;readonly createdAtUnixMs:number;readonly updatedAtUnixMs:number}
export interface WorldEvent{readonly id:string;readonly entityId:string;readonly sequence:number;readonly entityVersion:number;readonly eventType:string;readonly payload:unknown;readonly actor:"owner";readonly occurredAtUnixMs:number}
export interface WorldSnapshot{readonly projects:readonly WorldProject[];readonly eventCount:number;readonly consistent:boolean;readonly inconsistencies:readonly string[]}
export interface CreateWorldProjectRequest{readonly title:string;readonly summary:string}
export interface TransitionWorldProjectRequest{readonly id:string;readonly expectedVersion:number;readonly lifecycle:ProjectLifecycle;readonly summary?:string}

export type ShellLifecycle = "starting" | "ready" | "exiting";
export type LogLevel = "debug" | "info" | "warn" | "error";

export interface ShellStatus {
  readonly appName: "SAIKI";
  readonly version: string;
  readonly platform: "windows";
  readonly lifecycle: ShellLifecycle;
  readonly sessionId: string;
  readonly startedAtUnixMs: number;
}

export interface FrontendLogEntry {
  readonly level: LogLevel;
  readonly event: string;
  readonly message: string;
  readonly context?: Readonly<Record<string, string | number | boolean | null>>;
}

export interface ExitRequest {
  readonly reason: "owner_request" | "smoke_test";
}

export interface MonitorDescriptor {
  readonly index: number;
  readonly name: string | null;
  readonly position: { readonly x: number; readonly y: number };
  readonly size: { readonly width: number; readonly height: number };
  readonly scaleFactor: number;
}

export interface OverlayStatus {
  readonly paused: boolean;
  readonly activeMonitorIndex: number;
  readonly monitors: readonly MonitorDescriptor[];
  readonly pauseHotkey: "Ctrl+Shift+F12";
}

export type WorkspaceMode = "floating" | "full";

export interface ApprovalSummary {
  readonly id: string;
  readonly action: string;
  readonly target: string;
  readonly payloadDigest: string;
  readonly expiresAtUnixMs: number;
  readonly status: "pending";
}

export interface WorkspaceState {
  readonly mode: WorkspaceMode;
  readonly reducedMotion: boolean;
  readonly speechBubbles: boolean;
  readonly approvals: {
    readonly availability: "unavailable";
    readonly reason: string;
    readonly pending: readonly ApprovalSummary[];
  };
}

export interface CommandMap {
  readonly get_shell_status: { readonly args: undefined; readonly result: ShellStatus };
  readonly write_frontend_log: { readonly args: { readonly entry: FrontendLogEntry }; readonly result: undefined };
  readonly request_app_exit: { readonly args: { readonly request: ExitRequest }; readonly result: undefined };
  readonly get_overlay_status: { readonly args: undefined; readonly result: OverlayStatus };
  readonly move_overlay_to_monitor: { readonly args: { readonly monitorIndex: number }; readonly result: OverlayStatus };
  readonly set_overlay_paused: { readonly args: { readonly paused: boolean }; readonly result: OverlayStatus };
  readonly get_workspace_state: { readonly args: undefined; readonly result: WorkspaceState };
  readonly set_workspace_mode: { readonly args: { readonly mode: WorkspaceMode }; readonly result: WorkspaceState };
  readonly set_ui_preferences: { readonly args: { readonly preferences: { readonly reducedMotion: boolean; readonly speechBubbles: boolean } }; readonly result: WorkspaceState };
  readonly discover_ollama_models: { readonly args: undefined; readonly result: readonly OllamaModel[] };
  readonly start_ollama_chat: { readonly args: { readonly request: OllamaChatRequest }; readonly result: { readonly requestId: string } };
  readonly cancel_ollama_chat: { readonly args: { readonly requestId: string }; readonly result: boolean };
  readonly get_cloud_status: { readonly args: undefined; readonly result: CloudStatus };
  readonly import_openai_key_from_environment: { readonly args: undefined; readonly result: CloudStatus };
  readonly delete_openai_key: { readonly args: undefined; readonly result: CloudStatus };
  readonly configure_cloud_provider: { readonly args: { readonly request: ConfigureCloudRequest }; readonly result: CloudStatus };
  readonly start_cloud_chat: { readonly args: { readonly request: CloudChatRequest }; readonly result: { readonly requestId: string } };
  readonly cancel_cloud_chat: { readonly args: { readonly requestId: string }; readonly result: boolean };
  readonly get_conversation: { readonly args: undefined; readonly result: ConversationView };
  readonly append_owner_message: { readonly args: { readonly content: string }; readonly result: ConversationView };
  readonly append_saiki_message: { readonly args: { readonly content: string }; readonly result: ConversationView };
  readonly record_tool_result: { readonly args: { readonly content: string }; readonly result: ConversationView };
  readonly clear_conversation: { readonly args: undefined; readonly result: ConversationView };
  readonly compose_conversation_context: { readonly args: undefined; readonly result: ComposedContext };
  readonly teach_memory: { readonly args: { readonly request: TeachMemoryRequest }; readonly result: MemoryClaim };
  readonly correct_memory: { readonly args: { readonly request: CorrectMemoryRequest }; readonly result: MemoryClaim };
  readonly forget_memory: { readonly args: { readonly id: string }; readonly result: boolean };
  readonly search_memory: { readonly args: { readonly query: string }; readonly result: readonly MemoryClaim[] };
  readonly list_memory: { readonly args: undefined; readonly result: readonly MemoryClaim[] };
  readonly export_memory: { readonly args: undefined; readonly result: MemoryExport };
  readonly ingest_document: { readonly args: { readonly request: IngestDocumentRequest }; readonly result: DocumentSummary };
  readonly list_documents: { readonly args: undefined; readonly result: readonly DocumentSummary[] };
  readonly forget_document: { readonly args: { readonly id: string }; readonly result: boolean };
  readonly retrieve_knowledge: { readonly args: { readonly request: RetrieveKnowledgeRequest }; readonly result: readonly Citation[] };
  readonly get_voice_status: { readonly args: undefined; readonly result: VoiceStatus };
  readonly configure_voice: { readonly args: { readonly request: ConfigureVoiceRequest }; readonly result: VoiceStatus };
  readonly preview_lip_sync: { readonly args: { readonly request: { readonly text: string } }; readonly result: LipSync };
  readonly start_listening: { readonly args: undefined; readonly result: VoiceStatus };
  readonly stop_listening: { readonly args: undefined; readonly result: VoiceStatus };
  readonly interrupt_voice: { readonly args: undefined; readonly result: VoiceStatus };
  readonly cancel_voice: { readonly args: undefined; readonly result: VoiceStatus };
  readonly get_policy_state: { readonly args: undefined; readonly result: PolicyState };
  readonly evaluate_action: { readonly args: { readonly request: EvaluateActionRequest }; readonly result: Decision };
  readonly create_grant: { readonly args: { readonly request: CreateGrantRequest }; readonly result: PolicyGrant };
  readonly revoke_grant: { readonly args: { readonly id: string }; readonly result: boolean };
  readonly request_approval: { readonly args: { readonly request: RequestApprovalRequest }; readonly result: PolicyApproval };
  readonly resolve_approval: { readonly args: { readonly request: { readonly id: string; readonly approve: boolean } }; readonly result: PolicyApproval };
  readonly set_kill_switch: { readonly args: { readonly engaged: boolean }; readonly result: PolicyState };
  readonly list_policy_audit: { readonly args: undefined; readonly result: readonly PolicyAuditEntry[] };
  readonly get_systools_status: { readonly args: undefined; readonly result: SysToolsStatus };
  readonly configure_workspace: { readonly args: { readonly root: string }; readonly result: SysToolsStatus };
  readonly add_allowed_app: { readonly args: { readonly request: AddAppRequest }; readonly result: SysToolsStatus };
  readonly remove_allowed_app: { readonly args: { readonly id: string }; readonly result: SysToolsStatus };
  readonly launch_app: { readonly args: { readonly id: string }; readonly result: boolean };
  readonly read_workspace_file: { readonly args: { readonly request: { readonly path: string } }; readonly result: FileContent };
  readonly list_workspace_dir: { readonly args: { readonly request: { readonly path: string } }; readonly result: readonly DirEntryInfo[] };
  readonly write_workspace_file: { readonly args: { readonly request: WriteFileRequest }; readonly result: WriteResult };
  readonly rollback_write: { readonly args: { readonly token: string }; readonly result: boolean };
  readonly get_devtools_status: { readonly args: undefined; readonly result: DevToolsStatus };
  readonly configure_devtools: { readonly args: { readonly request: ConfigureDevToolsRequest }; readonly result: DevToolsStatus };
  readonly add_allowed_command: { readonly args: { readonly command: string }; readonly result: DevToolsStatus };
  readonly remove_allowed_command: { readonly args: { readonly command: string }; readonly result: DevToolsStatus };
  readonly git_status: { readonly args: undefined; readonly result: GitStatus };
  readonly git_diff: { readonly args: { readonly request: { readonly path?: string } }; readonly result: GitDiff };
  readonly run_command: { readonly args: { readonly request: RunCommandRequest }; readonly result: CommandResult };
  readonly cancel_command: { readonly args: { readonly requestId: string }; readonly result: boolean };
  readonly open_in_vscode: { readonly args: { readonly request: { readonly path: string } }; readonly result: boolean };
  readonly get_browser_status: { readonly args: undefined; readonly result: BrowserStatus };
  readonly create_browser_task: { readonly args: { readonly request: CreateBrowserTaskRequest }; readonly result: BrowserTask };
  readonly checkpoint_browser_task: { readonly args: { readonly request: CheckpointBrowserTaskRequest }; readonly result: BrowserCheckpoint };
  readonly list_browser_tasks: { readonly args: undefined; readonly result: readonly BrowserTask[] };
  readonly evaluate_browser_step: { readonly args: { readonly request: BrowserStepRequest }; readonly result: BrowserStepResult };
  readonly get_scheduler_snapshot: { readonly args: undefined; readonly result: SchedulerSnapshot };
  readonly create_scheduled_task: { readonly args: { readonly request: CreateScheduleRequest }; readonly result: ScheduledTask };
  readonly cancel_scheduled_task: { readonly args: { readonly id:string }; readonly result:boolean };
  readonly acknowledge_notification: { readonly args: { readonly id:string }; readonly result:boolean };
  readonly get_world_snapshot:{readonly args:undefined;readonly result:WorldSnapshot};
  readonly list_world_events:{readonly args:{readonly afterSequence:number};readonly result:readonly WorldEvent[]};
  readonly create_world_project:{readonly args:{readonly request:CreateWorldProjectRequest};readonly result:WorldProject};
  readonly transition_world_project:{readonly args:{readonly request:TransitionWorldProjectRequest};readonly result:WorldProject};
}

export function isWorldProject(value:unknown):value is WorldProject{if(!value||typeof value!=="object")return false;const p=value as Record<string,unknown>;return typeof p.id==="string"&&typeof p.title==="string"&&["planned","active","paused","completed","cancelled"].includes(String(p.lifecycle))&&typeof p.version==="number";}
export function isWorldSnapshot(value:unknown):value is WorldSnapshot{if(!value||typeof value!=="object")return false;const w=value as Record<string,unknown>;return Array.isArray(w.projects)&&w.projects.every(isWorldProject)&&typeof w.eventCount==="number"&&typeof w.consistent==="boolean"&&Array.isArray(w.inconsistencies)&&w.inconsistencies.every(v=>typeof v==="string");}
export function isWorldEvent(value:unknown):value is WorldEvent{if(!value||typeof value!=="object")return false;const e=value as Record<string,unknown>;return typeof e.id==="string"&&typeof e.entityId==="string"&&typeof e.sequence==="number"&&typeof e.entityVersion==="number"&&typeof e.eventType==="string"&&e.actor==="owner"&&typeof e.occurredAtUnixMs==="number";}

export function isSchedulerSnapshot(value:unknown):value is SchedulerSnapshot {
  if(!value||typeof value!=="object")return false; const s=value as Record<string,unknown>;
  return Array.isArray(s.tasks)&&Array.isArray(s.notifications)&&s.wakeTrigger==="native_core_1s_tick"&&s.retryPolicy==="durable_exponential_backoff_max_8";
}
export function isReminderNotification(value:unknown):value is ReminderNotification {
  if(!value||typeof value!=="object")return false; const n=value as Record<string,unknown>;
  return typeof n.id==="string"&&typeof n.taskId==="string"&&typeof n.title==="string"&&typeof n.message==="string"&&(n.deliveryState==="pending"||n.deliveryState==="acknowledged")&&typeof n.deliveryAttempts==="number";
}

export function isBrowserStatus(value: unknown): value is BrowserStatus {
  if (!value || typeof value !== "object") return false;
  const b = value as Record<string, unknown>;
  return (b.availability === "available" || b.availability === "unavailable") && typeof b.reason === "string" && typeof b.sidecarPresent === "boolean" && typeof b.enginePresent === "boolean" && b.profileIsolation === "dedicated_non_owner_profile" && b.contentTrust === "untrusted_web_content";
}
export function isBrowserTask(value: unknown): value is BrowserTask {
  if (!value || typeof value !== "object") return false;
  const b = value as Record<string, unknown>;
  return typeof b.id === "string" && typeof b.objective === "string" && typeof b.startUrl === "string" && b.state === "checkpointed" && Array.isArray(b.checkpoints) && b.checkpoints.every((item) => Boolean(item) && typeof item === "object" && (item as Record<string, unknown>).trust === "untrusted_web_content" && typeof (item as Record<string, unknown>).contentDigest === "string");
}
export function isBrowserStepResult(value: unknown): value is BrowserStepResult {
  if (!value || typeof value !== "object") return false;
  const b = value as Record<string, unknown>;
  return ["denied", "approval_required", "unavailable"].includes(String(b.outcome)) && typeof b.reason === "string" && b.executed === false;
}

export function isDevToolsStatus(value: unknown): value is DevToolsStatus {
  if (!value || typeof value !== "object") return false;
  const d = value as Record<string, unknown>;
  if (typeof d.repoConfigured !== "boolean" || typeof d.isGitRepo !== "boolean" || !Array.isArray(d.allowedCommands)) return false;
  const codex = d.codex as Record<string, unknown> | undefined;
  return Boolean(codex) && typeof codex === "object" && typeof codex.available === "boolean" && typeof codex.reason === "string";
}

export function isGitStatus(value: unknown): value is GitStatus {
  if (!value || typeof value !== "object") return false;
  const g = value as Record<string, unknown>;
  return typeof g.branch === "string" && typeof g.clean === "boolean" && Array.isArray(g.entries) &&
    g.entries.every((e) => Boolean(e) && typeof e === "object" && typeof (e as Record<string, unknown>).status === "string" && typeof (e as Record<string, unknown>).path === "string");
}

export function isCommandResult(value: unknown): value is CommandResult {
  if (!value || typeof value !== "object") return false;
  const r = value as Record<string, unknown>;
  const outcomes = ["succeeded", "failed", "denied", "cancelled", "unavailable"];
  return typeof r.requestId === "string" && typeof r.command === "string" && outcomes.includes(String(r.outcome)) &&
    (r.exitCode === null || typeof r.exitCode === "number") && typeof r.stdout === "string" && typeof r.stderr === "string";
}

export function isSysToolsStatus(value: unknown): value is SysToolsStatus {
  if (!value || typeof value !== "object") return false;
  const s = value as Record<string, unknown>;
  if (typeof s.workspaceConfigured !== "boolean" || typeof s.workspacePresent !== "boolean" || !Array.isArray(s.allowedApps)) return false;
  if (s.workspaceRoot !== null && typeof s.workspaceRoot !== "string") return false;
  const m = s.metrics as Record<string, unknown> | undefined;
  return Boolean(m) && typeof m === "object" && typeof m.cpuCount === "number" && typeof m.os === "string" && typeof m.pointerWidthBits === "number";
}

export function isFileContent(value: unknown): value is FileContent {
  if (!value || typeof value !== "object") return false;
  const f = value as Record<string, unknown>;
  return typeof f.path === "string" && typeof f.content === "string" && typeof f.bytes === "number";
}

export function isDirEntryList(value: unknown): value is readonly DirEntryInfo[] {
  return Array.isArray(value) && value.every((e) => Boolean(e) && typeof e === "object" && typeof (e as Record<string, unknown>).name === "string" && typeof (e as Record<string, unknown>).isDir === "boolean");
}

export function isWriteResult(value: unknown): value is WriteResult {
  if (!value || typeof value !== "object") return false;
  const w = value as Record<string, unknown>;
  return typeof w.path === "string" && typeof w.bytes === "number" && typeof w.previousExisted === "boolean" && typeof w.rollbackToken === "string";
}

const RISK_TIERS: readonly RiskTier[] = ["internal", "low", "external", "financial", "critical"];

export function isDecision(value: unknown): value is Decision {
  if (!value || typeof value !== "object") return false;
  const d = value as Record<string, unknown>;
  if (d.decision === "allow") return true;
  if (d.decision === "deny") return typeof d.reason === "string";
  return d.decision === "approval_required" && RISK_TIERS.includes(d.tier as RiskTier) && typeof d.reason === "string";
}

export function isPolicyState(value: unknown): value is PolicyState {
  if (!value || typeof value !== "object") return false;
  const s = value as Record<string, unknown>;
  if (typeof s.killSwitchEngaged !== "boolean" || !Array.isArray(s.grants) || !Array.isArray(s.pendingApprovals)) return false;
  return s.grants.every((g) => Boolean(g) && typeof g === "object" && typeof (g as Record<string, unknown>).id === "string" && RISK_TIERS.includes((g as Record<string, unknown>).tier as RiskTier));
}

export function isPolicyAuditList(value: unknown): value is readonly PolicyAuditEntry[] {
  return Array.isArray(value) && value.every((e) => Boolean(e) && typeof e === "object" && typeof (e as Record<string, unknown>).decision === "string" && typeof (e as Record<string, unknown>).capability === "string");
}

const VISEMES: readonly Viseme[] = ["silence", "pp", "ff", "th", "dd", "kk", "ch", "ss", "nn", "rr", "aa", "ee", "ih", "oh", "ou"];

export function isVoiceStatus(value: unknown): value is VoiceStatus {
  if (!value || typeof value !== "object") return false;
  const v = value as Record<string, unknown>;
  const asset = (a: unknown): boolean => Boolean(a) && typeof a === "object" && typeof (a as Record<string, unknown>).configured === "boolean" && typeof (a as Record<string, unknown>).present === "boolean";
  const cap = (c: unknown): boolean => Boolean(c) && typeof c === "object" && typeof (c as Record<string, unknown>).available === "boolean" && typeof (c as Record<string, unknown>).reason === "string";
  return asset(v.sttModel) && asset(v.ttsVoice) && cap(v.microphone) && cap(v.playback) &&
    typeof v.listeningAvailable === "boolean" && typeof v.speechAudioAvailable === "boolean" &&
    typeof v.lipSyncAvailable === "boolean" && ["idle", "listening", "speaking"].includes(String(v.session));
}

export function isLipSync(value: unknown): value is LipSync {
  if (!value || typeof value !== "object") return false;
  const v = value as Record<string, unknown>;
  if (typeof v.estimatedDurationMs !== "number" || v.audioOutcome !== "unavailable" || typeof v.audioReason !== "string") return false;
  return Array.isArray(v.visemes) && v.visemes.every((frame) => {
    if (!frame || typeof frame !== "object") return false;
    const f = frame as Record<string, unknown>;
    return VISEMES.includes(f.viseme as Viseme) && typeof f.startMs === "number" && typeof f.durationMs === "number";
  });
}

export function isDocumentSummary(value: unknown): value is DocumentSummary {
  if (!value || typeof value !== "object") return false;
  const d = value as Record<string, unknown>;
  return typeof d.id === "string" && typeof d.title === "string" && typeof d.path === "string" &&
    SENSITIVITIES.includes(d.sensitivity as Sensitivity) && typeof d.byteLen === "number" &&
    typeof d.chunkCount === "number" && typeof d.ingestedAtUnixMs === "number";
}

export function isDocumentSummaryList(value: unknown): value is readonly DocumentSummary[] {
  return Array.isArray(value) && value.every(isDocumentSummary);
}

export function isCitationList(value: unknown): value is readonly Citation[] {
  return Array.isArray(value) && value.every((item) => {
    if (!item || typeof item !== "object") return false;
    const c = item as Record<string, unknown>;
    return typeof c.documentId === "string" && typeof c.title === "string" && typeof c.path === "string" &&
      SENSITIVITIES.includes(c.sensitivity as Sensitivity) && typeof c.chunkIndex === "number" &&
      typeof c.score === "number" && typeof c.snippet === "string";
  });
}

const SENSITIVITIES: readonly Sensitivity[] = ["public", "private", "sensitive"];
const CLAIM_STATUSES: readonly ClaimStatus[] = ["active", "superseded", "forgotten"];

export function isMemoryClaim(value: unknown): value is MemoryClaim {
  if (!value || typeof value !== "object") return false;
  const c = value as Record<string, unknown>;
  return typeof c.id === "string" && typeof c.claim === "string" && typeof c.source === "string" &&
    typeof c.confidence === "number" && SENSITIVITIES.includes(c.sensitivity as Sensitivity) &&
    CLAIM_STATUSES.includes(c.status as ClaimStatus) && (c.supersedes === null || typeof c.supersedes === "string") &&
    typeof c.createdAtUnixMs === "number" && typeof c.updatedAtUnixMs === "number";
}

export function isMemoryClaimList(value: unknown): value is readonly MemoryClaim[] {
  return Array.isArray(value) && value.every(isMemoryClaim);
}

export function isMemoryExport(value: unknown): value is MemoryExport {
  if (!value || typeof value !== "object") return false;
  const e = value as Record<string, unknown>;
  return typeof e.schemaVersion === "number" && isMemoryClaimList(e.claims) && Array.isArray(e.events);
}

const EXPRESSIONS: readonly Expression[] = ["neutral", "talking", "focused", "annoyed", "surprised", "tired"];

export function isConversationView(value: unknown): value is ConversationView {
  if (!value || typeof value !== "object") return false;
  const view = value as Record<string, unknown>;
  const persona = view.persona as Record<string, unknown> | undefined;
  if (!persona || persona.authority !== "none" || typeof persona.name !== "string" || typeof persona.preamble !== "string" || !Array.isArray(persona.tone) || !Array.isArray(persona.expressions)) return false;
  if (!view.limits || typeof view.limits !== "object") return false;
  if (!Array.isArray(view.messages)) return false;
  return view.messages.every((message) => {
    if (!message || typeof message !== "object") return false;
    const m = message as Record<string, unknown>;
    if (typeof m.id !== "string" || typeof m.content !== "string" || typeof m.createdAtUnixMs !== "number") return false;
    if (m.role !== "owner" && m.role !== "saiki" && m.role !== "tool_result") return false;
    return m.expression === undefined || EXPRESSIONS.includes(m.expression as Expression);
  });
}

export function isExpressionEvent(value: unknown): value is ExpressionEvent {
  if (!value || typeof value !== "object") return false;
  return EXPRESSIONS.includes((value as Record<string, unknown>).expression as Expression);
}

export function isCloudStatus(value: unknown): value is CloudStatus {
  if (!value || typeof value !== "object") return false;
  const status = value as Record<string, unknown>;
  if (typeof status.enabled !== "boolean" || typeof status.model !== "string" || typeof status.credentialPresent !== "boolean" || status.credentialStore !== "windows_credential_manager" || status.endpoint !== "https://api.openai.com/v1/responses") return false;
  if (!status.disclosure || typeof status.disclosure !== "object") return false;
  const disclosure = status.disclosure as Record<string, unknown>;
  return typeof disclosure.destination === "string" && Array.isArray(disclosure.data) && disclosure.data.length === 2 && disclosure.storageRequested === false && typeof disclosure.billing === "string";
}

export function isCloudStreamEvent(value: unknown): value is CloudStreamEvent {
  if (!value || typeof value !== "object") return false;
  const event = value as Record<string, unknown>;
  if (typeof event.request_id !== "string") return false;
  if (event.kind === "chunk") return typeof event.content === "string";
  if (event.kind === "finished") return event.outcome === "succeeded" || event.outcome === "cancelled";
  return event.kind === "failed" && (event.outcome === "failed" || event.outcome === "unavailable") && typeof event.message === "string";
}

export function isOllamaModelList(value: unknown): value is readonly OllamaModel[] {
  return Array.isArray(value) && value.every((model) => Boolean(model) && typeof model === "object" &&
    typeof (model as OllamaModel).name === "string" && typeof (model as OllamaModel).size === "number" &&
    typeof (model as OllamaModel).digest === "string" && typeof (model as OllamaModel).modifiedAt === "string");
}

export function isOllamaStreamEvent(value: unknown): value is OllamaStreamEvent {
  if (!value || typeof value !== "object") return false;
  const event = value as Record<string, unknown>;
  if (typeof event.request_id !== "string") return false;
  if (event.kind === "chunk") return typeof event.content === "string";
  if (event.kind === "finished") return event.outcome === "succeeded" || event.outcome === "cancelled";
  if (event.kind === "failed") return (event.outcome === "failed" || event.outcome === "unavailable") && typeof event.message === "string";
  if (event.kind === "tool_proposal" && event.proposal && typeof event.proposal === "object") return (event.proposal as Record<string, unknown>).trust === "untrusted_model_output";
  return false;
}

export function isWorkspaceState(value: unknown): value is WorkspaceState {
  if (!value || typeof value !== "object") return false;
  const v = value as Record<string, unknown>;
  if (!(["floating", "full"] as const).includes(v.mode as WorkspaceMode) || typeof v.reducedMotion !== "boolean" || typeof v.speechBubbles !== "boolean") return false;
  if (!v.approvals || typeof v.approvals !== "object") return false;
  const approvals = v.approvals as Record<string, unknown>;
  return approvals.availability === "unavailable" && typeof approvals.reason === "string" && Array.isArray(approvals.pending) && approvals.pending.length === 0;
}

export function isShellStatus(value: unknown): value is ShellStatus {
  if (!value || typeof value !== "object") return false;
  const v = value as Record<string, unknown>;
  return v.appName === "SAIKI" && typeof v.version === "string" && v.platform === "windows" &&
    ["starting", "ready", "exiting"].includes(String(v.lifecycle)) && typeof v.sessionId === "string" &&
    typeof v.startedAtUnixMs === "number";
}

export function isOverlayStatus(value: unknown): value is OverlayStatus {
  if (!value || typeof value !== "object") return false;
  const v = value as Record<string, unknown>;
  return typeof v.paused === "boolean" && Number.isInteger(v.activeMonitorIndex) &&
    v.pauseHotkey === "Ctrl+Shift+F12" && Array.isArray(v.monitors) && v.monitors.every((monitor) => {
      if (!monitor || typeof monitor !== "object") return false;
      const m = monitor as Record<string, unknown>;
      return Number.isInteger(m.index) && typeof m.scaleFactor === "number" && Boolean(m.position) && Boolean(m.size);
    });
}

export function validateFrontendLog(entry: FrontendLogEntry): void {
  if (!entry.event.trim() || entry.event.length > 80) throw new Error("Log event must be 1-80 characters.");
  if (!entry.message.trim() || entry.message.length > 500) throw new Error("Log message must be 1-500 characters.");
  if (entry.context && Object.keys(entry.context).length > 20) throw new Error("Log context is limited to 20 fields.");
}
