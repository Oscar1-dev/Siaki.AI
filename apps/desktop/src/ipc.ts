import { invoke } from "@tauri-apps/api/core";
import {
  IPC_COMMANDS,
  isShellStatus,
  isOverlayStatus,
  isWorkspaceState,
  validateFrontendLog,
  type CommandMap,
  type FrontendLogEntry,
  type ShellStatus,
  type OverlayStatus,
  type WorkspaceMode,
  type WorkspaceState,
} from "@saiki/ipc-contracts";

async function invokeTyped<K extends keyof CommandMap>(
  command: K,
  ...args: CommandMap[K]["args"] extends undefined ? [] : [CommandMap[K]["args"]]
): Promise<CommandMap[K]["result"]> {
  return invoke(command, args[0] ?? {});
}

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
