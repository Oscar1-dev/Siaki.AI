export const IPC_COMMANDS = {
  getShellStatus: "get_shell_status",
  writeFrontendLog: "write_frontend_log",
  requestAppExit: "request_app_exit",
  getOverlayStatus: "get_overlay_status",
  moveOverlayToMonitor: "move_overlay_to_monitor",
  setOverlayPaused: "set_overlay_paused",
} as const;

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

export interface CommandMap {
  readonly get_shell_status: { readonly args: undefined; readonly result: ShellStatus };
  readonly write_frontend_log: { readonly args: { readonly entry: FrontendLogEntry }; readonly result: undefined };
  readonly request_app_exit: { readonly args: { readonly request: ExitRequest }; readonly result: undefined };
  readonly get_overlay_status: { readonly args: undefined; readonly result: OverlayStatus };
  readonly move_overlay_to_monitor: { readonly args: { readonly monitorIndex: number }; readonly result: OverlayStatus };
  readonly set_overlay_paused: { readonly args: { readonly paused: boolean }; readonly result: OverlayStatus };
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
