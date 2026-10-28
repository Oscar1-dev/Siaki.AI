import { invoke } from "@tauri-apps/api/core";
import {
  IPC_COMMANDS,
  isShellStatus,
  isOverlayStatus,
  validateFrontendLog,
  type CommandMap,
  type FrontendLogEntry,
  type ShellStatus,
  type OverlayStatus,
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
