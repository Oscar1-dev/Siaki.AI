import { invoke } from "@tauri-apps/api/core";
import {
  IPC_COMMANDS,
  isShellStatus,
  isOverlayStatus,
  isWorkspaceState,
  isOllamaModelList,
  isCloudStatus,
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
