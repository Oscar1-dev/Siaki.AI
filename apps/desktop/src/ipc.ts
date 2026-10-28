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
