import { invoke } from "@tauri-apps/api/core";
import {
  IPC_COMMANDS,
  isShellStatus,
  isOverlayStatus,
  isWorkspaceState,
  isOllamaModelList,
  isCloudStatus,
  isConversationView,
  isMemoryClaim,
  isMemoryClaimList,
  isMemoryExport,
  isDocumentSummary,
  isDocumentSummaryList,
  isCitationList,
  isVoiceStatus,
  isLipSync,
  isDecision,
  isPolicyState,
  isPolicyAuditList,
  isSysToolsStatus,
  isFileContent,
  isDirEntryList,
  isWriteResult,
  isDevToolsStatus,
  isGitStatus,
  isCommandResult,
  isBrowserStatus,
  isBrowserTask,
  isBrowserStepResult,
  isSchedulerSnapshot,
  isWorldEvent,
  isWorldProject,
  isWorldSnapshot,
  isSupervisorSnapshot,
  isPlanningSnapshot,
  isIdentityRegistryStatus,
  isWorkflowSnapshot,
  validateFrontendLog,
  type CommandMap,
  type FrontendLogEntry,
  type ShellStatus,
  type OverlayStatus,
  type WorkspaceMode,
  type WorkspaceState,
  type OllamaChatRequest,
  type OllamaModel,
  type CloudChatRequest,
  type CloudStatus,
  type ConfigureCloudRequest,
  type ComposedContext,
  type ConversationView,
  type CorrectMemoryRequest,
  type MemoryClaim,
  type MemoryExport,
  type TeachMemoryRequest,
  type Citation,
  type DocumentSummary,
  type IngestDocumentRequest,
  type RetrieveKnowledgeRequest,
  type ConfigureVoiceRequest,
  type LipSync,
  type VoiceStatus,
  type CreateGrantRequest,
  type Decision,
  type EvaluateActionRequest,
  type PolicyApproval,
  type PolicyAuditEntry,
  type PolicyGrant,
  type PolicyState,
  type RequestApprovalRequest,
  type AddAppRequest,
  type DirEntryInfo,
  type FileContent,
  type SysToolsStatus,
  type WriteFileRequest,
  type WriteResult,
  type CommandResult,
  type ConfigureDevToolsRequest,
  type DevToolsStatus,
  type GitDiff,
  type GitStatus,
  type RunCommandRequest,
  type BrowserStatus,
  type BrowserTask,
  type BrowserCheckpoint,
  type BrowserStepRequest,
  type BrowserStepResult,
  type CreateBrowserTaskRequest,
  type CheckpointBrowserTaskRequest,
  type SchedulerSnapshot,
  type ScheduledTask,
  type CreateScheduleRequest,
  type WorldEvent,
  type WorldProject,
  type WorldSnapshot,
  type CreateWorldProjectRequest,
  type TransitionWorldProjectRequest,
  type SupervisorGoal,
  type SupervisorSnapshot,
  type CreateSupervisorGoalRequest,
  type PlanningSnapshot,
  type PlannedGoal,
  type CreatePlannedGoalRequest,
  type IdentityRecord,
  type IdentityRegistryStatus,
  type RegisterIdentityRequest,
  type UpdateMfaRequest,
  type AccountWorkflow,
  type AccountWorkflowKind,
  type WorkflowSnapshot,
} from "@saiki/ipc-contracts";

async function invokeTyped<K extends keyof CommandMap>(
  command: K,
  ...args: CommandMap[K]["args"] extends undefined ? [] : [CommandMap[K]["args"]]
): Promise<CommandMap[K]["result"]> {
  return invoke(command, args[0] ?? {});
}

export async function discoverOllamaModels(): Promise<readonly OllamaModel[]> {
  const value: unknown = await invokeTyped(IPC_COMMANDS.discoverOllamaModels);
  if (!isOllamaModelList(value)) throw new Error("Native core returned an invalid Ollama model contract.");
  return value;
}

export async function startOllamaChat(request: OllamaChatRequest): Promise<{ readonly requestId: string }> {
  return invokeTyped(IPC_COMMANDS.startOllamaChat, { request });
}

export async function cancelOllamaChat(requestId: string): Promise<boolean> {
  return invokeTyped(IPC_COMMANDS.cancelOllamaChat, { requestId });
}

export async function getCloudStatus(): Promise<CloudStatus> {
  const value: unknown = await invokeTyped(IPC_COMMANDS.getCloudStatus);
  if (!isCloudStatus(value)) throw new Error("Native core returned an invalid cloud status contract.");
  return value;
}
export async function importOpenAiKeyFromEnvironment(): Promise<CloudStatus> { const value: unknown = await invokeTyped(IPC_COMMANDS.importOpenAiKeyFromEnvironment); if (!isCloudStatus(value)) throw new Error("Native core returned an invalid cloud status contract."); return value; }
export async function deleteOpenAiKey(): Promise<CloudStatus> { const value: unknown = await invokeTyped(IPC_COMMANDS.deleteOpenAiKey); if (!isCloudStatus(value)) throw new Error("Native core returned an invalid cloud status contract."); return value; }
export async function configureCloudProvider(request: ConfigureCloudRequest): Promise<CloudStatus> { const value: unknown = await invokeTyped(IPC_COMMANDS.configureCloudProvider, { request }); if (!isCloudStatus(value)) throw new Error("Native core returned an invalid cloud status contract."); return value; }
export async function startCloudChat(request: CloudChatRequest): Promise<{ readonly requestId: string }> { return invokeTyped(IPC_COMMANDS.startCloudChat, { request }); }
export async function cancelCloudChat(requestId: string): Promise<boolean> { return invokeTyped(IPC_COMMANDS.cancelCloudChat, { requestId }); }

function expectConversation(value: unknown): ConversationView { if (!isConversationView(value)) throw new Error("Native core returned an invalid conversation contract."); return value; }
export async function getConversation(): Promise<ConversationView> { return expectConversation(await invokeTyped(IPC_COMMANDS.getConversation)); }
export async function appendOwnerMessage(content: string): Promise<ConversationView> { return expectConversation(await invokeTyped(IPC_COMMANDS.appendOwnerMessage, { content })); }
export async function appendSaikiMessage(content: string): Promise<ConversationView> { return expectConversation(await invokeTyped(IPC_COMMANDS.appendSaikiMessage, { content })); }
export async function recordToolResult(content: string): Promise<ConversationView> { return expectConversation(await invokeTyped(IPC_COMMANDS.recordToolResult, { content })); }
export async function clearConversation(): Promise<ConversationView> { return expectConversation(await invokeTyped(IPC_COMMANDS.clearConversation)); }
export async function composeConversationContext(): Promise<ComposedContext> { return invokeTyped(IPC_COMMANDS.composeConversationContext); }

function expectClaim(value: unknown): MemoryClaim { if (!isMemoryClaim(value)) throw new Error("Native core returned an invalid memory claim contract."); return value; }
function expectClaimList(value: unknown): readonly MemoryClaim[] { if (!isMemoryClaimList(value)) throw new Error("Native core returned an invalid memory list contract."); return value; }
export async function teachMemory(request: TeachMemoryRequest): Promise<MemoryClaim> { return expectClaim(await invokeTyped(IPC_COMMANDS.teachMemory, { request })); }
export async function correctMemory(request: CorrectMemoryRequest): Promise<MemoryClaim> { return expectClaim(await invokeTyped(IPC_COMMANDS.correctMemory, { request })); }
export async function forgetMemory(id: string): Promise<boolean> { return invokeTyped(IPC_COMMANDS.forgetMemory, { id }); }
export async function searchMemory(query: string): Promise<readonly MemoryClaim[]> { return expectClaimList(await invokeTyped(IPC_COMMANDS.searchMemory, { query })); }
export async function listMemory(): Promise<readonly MemoryClaim[]> { return expectClaimList(await invokeTyped(IPC_COMMANDS.listMemory)); }
export async function exportMemory(): Promise<MemoryExport> { const value: unknown = await invokeTyped(IPC_COMMANDS.exportMemory); if (!isMemoryExport(value)) throw new Error("Native core returned an invalid memory export contract."); return value; }

export async function ingestDocument(request: IngestDocumentRequest): Promise<DocumentSummary> { const value: unknown = await invokeTyped(IPC_COMMANDS.ingestDocument, { request }); if (!isDocumentSummary(value)) throw new Error("Native core returned an invalid document contract."); return value; }
export async function listDocuments(): Promise<readonly DocumentSummary[]> { const value: unknown = await invokeTyped(IPC_COMMANDS.listDocuments); if (!isDocumentSummaryList(value)) throw new Error("Native core returned an invalid document list contract."); return value; }
export async function forgetDocument(id: string): Promise<boolean> { return invokeTyped(IPC_COMMANDS.forgetDocument, { id }); }
export async function retrieveKnowledge(request: RetrieveKnowledgeRequest): Promise<readonly Citation[]> { const value: unknown = await invokeTyped(IPC_COMMANDS.retrieveKnowledge, { request }); if (!isCitationList(value)) throw new Error("Native core returned an invalid citation contract."); return value; }

function expectVoice(value: unknown): VoiceStatus { if (!isVoiceStatus(value)) throw new Error("Native core returned an invalid voice status contract."); return value; }
export async function getVoiceStatus(): Promise<VoiceStatus> { return expectVoice(await invokeTyped(IPC_COMMANDS.getVoiceStatus)); }
export async function configureVoice(request: ConfigureVoiceRequest): Promise<VoiceStatus> { return expectVoice(await invokeTyped(IPC_COMMANDS.configureVoice, { request })); }
export async function startListening(): Promise<VoiceStatus> { return expectVoice(await invokeTyped(IPC_COMMANDS.startListening)); }
export async function stopListening(): Promise<VoiceStatus> { return expectVoice(await invokeTyped(IPC_COMMANDS.stopListening)); }
export async function interruptVoice(): Promise<VoiceStatus> { return expectVoice(await invokeTyped(IPC_COMMANDS.interruptVoice)); }
export async function cancelVoice(): Promise<VoiceStatus> { return expectVoice(await invokeTyped(IPC_COMMANDS.cancelVoice)); }
export async function previewLipSync(text: string): Promise<LipSync> { const value: unknown = await invokeTyped(IPC_COMMANDS.previewLipSync, { request: { text } }); if (!isLipSync(value)) throw new Error("Native core returned an invalid lip-sync contract."); return value; }

function expectPolicyState(value: unknown): PolicyState { if (!isPolicyState(value)) throw new Error("Native core returned an invalid policy state contract."); return value; }
export async function getPolicyState(): Promise<PolicyState> { return expectPolicyState(await invokeTyped(IPC_COMMANDS.getPolicyState)); }
export async function evaluateAction(request: EvaluateActionRequest): Promise<Decision> { const value: unknown = await invokeTyped(IPC_COMMANDS.evaluateAction, { request }); if (!isDecision(value)) throw new Error("Native core returned an invalid policy decision contract."); return value; }
export async function createGrant(request: CreateGrantRequest): Promise<PolicyGrant> { return invokeTyped(IPC_COMMANDS.createGrant, { request }); }
export async function revokeGrant(id: string): Promise<boolean> { return invokeTyped(IPC_COMMANDS.revokeGrant, { id }); }
export async function requestApproval(request: RequestApprovalRequest): Promise<PolicyApproval> { return invokeTyped(IPC_COMMANDS.requestApproval, { request }); }
export async function resolveApproval(id: string, approve: boolean): Promise<PolicyApproval> { return invokeTyped(IPC_COMMANDS.resolveApproval, { request: { id, approve } }); }
export async function setKillSwitch(engaged: boolean): Promise<PolicyState> { return expectPolicyState(await invokeTyped(IPC_COMMANDS.setKillSwitch, { engaged })); }
export async function listPolicyAudit(): Promise<readonly PolicyAuditEntry[]> { const value: unknown = await invokeTyped(IPC_COMMANDS.listPolicyAudit); if (!isPolicyAuditList(value)) throw new Error("Native core returned an invalid audit contract."); return value; }

function expectSysTools(value: unknown): SysToolsStatus { if (!isSysToolsStatus(value)) throw new Error("Native core returned an invalid system tools contract."); return value; }
export async function getSysToolsStatus(): Promise<SysToolsStatus> { return expectSysTools(await invokeTyped(IPC_COMMANDS.getSysToolsStatus)); }
export async function configureWorkspace(root: string): Promise<SysToolsStatus> { return expectSysTools(await invokeTyped(IPC_COMMANDS.configureWorkspace, { root })); }
export async function addAllowedApp(request: AddAppRequest): Promise<SysToolsStatus> { return expectSysTools(await invokeTyped(IPC_COMMANDS.addAllowedApp, { request })); }
export async function removeAllowedApp(id: string): Promise<SysToolsStatus> { return expectSysTools(await invokeTyped(IPC_COMMANDS.removeAllowedApp, { id })); }
export async function launchApp(id: string): Promise<boolean> { return invokeTyped(IPC_COMMANDS.launchApp, { id }); }
export async function readWorkspaceFile(path: string): Promise<FileContent> { const value: unknown = await invokeTyped(IPC_COMMANDS.readWorkspaceFile, { request: { path } }); if (!isFileContent(value)) throw new Error("Native core returned an invalid file contract."); return value; }
export async function listWorkspaceDir(path: string): Promise<readonly DirEntryInfo[]> { const value: unknown = await invokeTyped(IPC_COMMANDS.listWorkspaceDir, { request: { path } }); if (!isDirEntryList(value)) throw new Error("Native core returned an invalid directory contract."); return value; }
export async function writeWorkspaceFile(request: WriteFileRequest): Promise<WriteResult> { const value: unknown = await invokeTyped(IPC_COMMANDS.writeWorkspaceFile, { request }); if (!isWriteResult(value)) throw new Error("Native core returned an invalid write-result contract."); return value; }
export async function rollbackWrite(token: string): Promise<boolean> { return invokeTyped(IPC_COMMANDS.rollbackWrite, { token }); }

function expectDevTools(value: unknown): DevToolsStatus { if (!isDevToolsStatus(value)) throw new Error("Native core returned an invalid devtools contract."); return value; }
export async function getDevToolsStatus(): Promise<DevToolsStatus> { return expectDevTools(await invokeTyped(IPC_COMMANDS.getDevToolsStatus)); }
export async function configureDevTools(request: ConfigureDevToolsRequest): Promise<DevToolsStatus> { return expectDevTools(await invokeTyped(IPC_COMMANDS.configureDevTools, { request })); }
export async function addAllowedCommand(command: string): Promise<DevToolsStatus> { return expectDevTools(await invokeTyped(IPC_COMMANDS.addAllowedCommand, { command })); }
export async function removeAllowedCommand(command: string): Promise<DevToolsStatus> { return expectDevTools(await invokeTyped(IPC_COMMANDS.removeAllowedCommand, { command })); }
export async function gitStatus(): Promise<GitStatus> { const value: unknown = await invokeTyped(IPC_COMMANDS.gitStatus); if (!isGitStatus(value)) throw new Error("Native core returned an invalid git status contract."); return value; }
export async function gitDiff(path?: string): Promise<GitDiff> { return invokeTyped(IPC_COMMANDS.gitDiff, { request: path === undefined ? {} : { path } }); }
export async function runCommand(request: RunCommandRequest): Promise<CommandResult> { const value: unknown = await invokeTyped(IPC_COMMANDS.runCommand, { request }); if (!isCommandResult(value)) throw new Error("Native core returned an invalid command-result contract."); return value; }
export async function cancelCommand(requestId: string): Promise<boolean> { return invokeTyped(IPC_COMMANDS.cancelCommand, { requestId }); }
export async function openInVscode(path: string): Promise<boolean> { return invokeTyped(IPC_COMMANDS.openInVscode, { request: { path } }); }
export async function getBrowserStatus(): Promise<BrowserStatus> { const value: unknown = await invokeTyped(IPC_COMMANDS.getBrowserStatus); if (!isBrowserStatus(value)) throw new Error("Native core returned an invalid browser status contract."); return value; }
export async function createBrowserTask(request: CreateBrowserTaskRequest): Promise<BrowserTask> { const value: unknown = await invokeTyped(IPC_COMMANDS.createBrowserTask, { request }); if (!isBrowserTask(value)) throw new Error("Native core returned an invalid browser task contract."); return value; }
export async function checkpointBrowserTask(request: CheckpointBrowserTaskRequest): Promise<BrowserCheckpoint> { return invokeTyped(IPC_COMMANDS.checkpointBrowserTask, { request }); }
export async function listBrowserTasks(): Promise<readonly BrowserTask[]> { const value: unknown = await invokeTyped(IPC_COMMANDS.listBrowserTasks); if (!Array.isArray(value) || !value.every(isBrowserTask)) throw new Error("Native core returned invalid browser tasks."); return value; }
export async function evaluateBrowserStep(request: BrowserStepRequest): Promise<BrowserStepResult> { const value: unknown = await invokeTyped(IPC_COMMANDS.evaluateBrowserStep, { request }); if (!isBrowserStepResult(value)) throw new Error("Native core returned an invalid browser decision contract."); return value; }
export async function getSchedulerSnapshot():Promise<SchedulerSnapshot>{const value:unknown=await invokeTyped(IPC_COMMANDS.getSchedulerSnapshot);if(!isSchedulerSnapshot(value))throw new Error("Native core returned an invalid scheduler contract.");return value;}
export async function createScheduledTask(request:CreateScheduleRequest):Promise<ScheduledTask>{return invokeTyped(IPC_COMMANDS.createScheduledTask,{request});}
export async function cancelScheduledTask(id:string):Promise<boolean>{return invokeTyped(IPC_COMMANDS.cancelScheduledTask,{id});}
export async function acknowledgeNotification(id:string):Promise<boolean>{return invokeTyped(IPC_COMMANDS.acknowledgeNotification,{id});}
export async function getWorldSnapshot():Promise<WorldSnapshot>{const value:unknown=await invokeTyped(IPC_COMMANDS.getWorldSnapshot);if(!isWorldSnapshot(value))throw new Error("Native core returned an invalid world snapshot.");return value;}
export async function listWorldEvents(afterSequence=0):Promise<readonly WorldEvent[]>{const value:unknown=await invokeTyped(IPC_COMMANDS.listWorldEvents,{afterSequence});if(!Array.isArray(value)||!value.every(isWorldEvent))throw new Error("Native core returned invalid world events.");return value;}
export async function createWorldProject(request:CreateWorldProjectRequest):Promise<WorldProject>{const value:unknown=await invokeTyped(IPC_COMMANDS.createWorldProject,{request});if(!isWorldProject(value))throw new Error("Native core returned an invalid project.");return value;}
export async function transitionWorldProject(request:TransitionWorldProjectRequest):Promise<WorldProject>{const value:unknown=await invokeTyped(IPC_COMMANDS.transitionWorldProject,{request});if(!isWorldProject(value))throw new Error("Native core returned an invalid project transition.");return value;}
export async function getSupervisorSnapshot():Promise<SupervisorSnapshot>{const value:unknown=await invokeTyped(IPC_COMMANDS.getSupervisorSnapshot);if(!isSupervisorSnapshot(value))throw new Error("Native core returned an invalid supervisor snapshot.");return value;}
export async function createSupervisorGoal(request:CreateSupervisorGoalRequest):Promise<SupervisorGoal>{return invokeTyped(IPC_COMMANDS.createSupervisorGoal,{request});}
export async function setSupervisorPaused(paused:boolean):Promise<SupervisorSnapshot>{const value:unknown=await invokeTyped(IPC_COMMANDS.setSupervisorPaused,{paused});if(!isSupervisorSnapshot(value))throw new Error("Native core returned an invalid supervisor snapshot.");return value;}
export async function getPlanningSnapshot():Promise<PlanningSnapshot>{const value:unknown=await invokeTyped(IPC_COMMANDS.getPlanningSnapshot);if(!isPlanningSnapshot(value))throw new Error("Native core returned an invalid planning snapshot.");return value;}
export async function createPlannedGoal(request:CreatePlannedGoalRequest):Promise<PlannedGoal>{return invokeTyped(IPC_COMMANDS.createPlannedGoal,{request});}
export async function stopPlannedGoal(id:string):Promise<boolean>{return invokeTyped(IPC_COMMANDS.stopPlannedGoal,{id});}
export async function getIdentityRegistry():Promise<IdentityRegistryStatus>{const value:unknown=await invokeTyped(IPC_COMMANDS.getIdentityRegistry);if(!isIdentityRegistryStatus(value))throw new Error("Native core returned an invalid identity registry.");return value;}
export async function registerIdentity(request:RegisterIdentityRequest):Promise<IdentityRecord>{return invokeTyped(IPC_COMMANDS.registerIdentity,{request});}
export async function updateIdentityMfa(request:UpdateMfaRequest):Promise<IdentityRecord>{return invokeTyped(IPC_COMMANDS.updateIdentityMfa,{request});}
export async function archiveIdentity(id:string):Promise<boolean>{return invokeTyped(IPC_COMMANDS.archiveIdentity,{id});}
export async function getAccountWorkflows():Promise<WorkflowSnapshot>{const value:unknown=await invokeTyped(IPC_COMMANDS.getAccountWorkflows);if(!isWorkflowSnapshot(value))throw new Error("Native core returned invalid account workflows.");return value;}
export async function createAccountWorkflow(identityId:string,kind:AccountWorkflowKind):Promise<AccountWorkflow>{return invokeTyped(IPC_COMMANDS.createAccountWorkflow,{request:{identityId,kind}});}
export async function confirmAccountWorkflowStep(workflowId:string,stepId:string):Promise<AccountWorkflow>{return invokeTyped(IPC_COMMANDS.confirmAccountWorkflowStep,{request:{workflowId,stepId,ownerConfirmed:true}});}
export async function cancelAccountWorkflow(id:string):Promise<boolean>{return invokeTyped(IPC_COMMANDS.cancelAccountWorkflow,{id});}

export async function getOverlayStatus(): Promise<OverlayStatus> {
  const value: unknown = await invokeTyped(IPC_COMMANDS.getOverlayStatus);
  if (!isOverlayStatus(value)) throw new Error("Native shell returned an invalid overlay status contract.");
  return value;
}

export async function moveOverlayToMonitor(monitorIndex: number): Promise<OverlayStatus> {
  const value: unknown = await invokeTyped(IPC_COMMANDS.moveOverlayToMonitor, { monitorIndex });
  if (!isOverlayStatus(value)) throw new Error("Native shell returned an invalid overlay status contract.");
  return value;
}

export async function setOverlayPaused(paused: boolean): Promise<OverlayStatus> {
  const value: unknown = await invokeTyped(IPC_COMMANDS.setOverlayPaused, { paused });
  if (!isOverlayStatus(value)) throw new Error("Native shell returned an invalid overlay status contract.");
  return value;
}

export async function getWorkspaceState(): Promise<WorkspaceState> {
  const value: unknown = await invokeTyped(IPC_COMMANDS.getWorkspaceState);
  if (!isWorkspaceState(value)) throw new Error("Native shell returned an invalid workspace state contract.");
  return value;
}

export async function setWorkspaceMode(mode: WorkspaceMode): Promise<WorkspaceState> {
  const value: unknown = await invokeTyped(IPC_COMMANDS.setWorkspaceMode, { mode });
  if (!isWorkspaceState(value)) throw new Error("Native shell returned an invalid workspace state contract.");
  return value;
}

export async function setUiPreferences(reducedMotion: boolean, speechBubbles: boolean): Promise<WorkspaceState> {
  const value: unknown = await invokeTyped(IPC_COMMANDS.setUiPreferences, { preferences: { reducedMotion, speechBubbles } });
  if (!isWorkspaceState(value)) throw new Error("Native shell returned an invalid workspace state contract.");
  return value;
}

export async function getShellStatus(): Promise<ShellStatus> {
  const value: unknown = await invokeTyped(IPC_COMMANDS.getShellStatus);
  if (!isShellStatus(value)) throw new Error("Native shell returned an invalid status contract.");
  return value;
}

export async function writeFrontendLog(entry: FrontendLogEntry): Promise<void> {
  validateFrontendLog(entry);
  await invokeTyped(IPC_COMMANDS.writeFrontendLog, { entry });
}

export async function requestAppExit(): Promise<void> {
  await invokeTyped(IPC_COMMANDS.requestAppExit, { request: { reason: "owner_request" } });
}
