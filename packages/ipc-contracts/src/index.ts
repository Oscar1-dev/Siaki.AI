export const IPC_COMMANDS = {
  getShellStatus: "get_shell_status",
  writeFrontendLog: "write_frontend_log",
  requestAppExit: "request_app_exit",
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

export interface CommandMap {
  readonly get_shell_status: { readonly args: undefined; readonly result: ShellStatus };
  readonly write_frontend_log: { readonly args: { readonly entry: FrontendLogEntry }; readonly result: undefined };
  readonly request_app_exit: { readonly args: { readonly request: ExitRequest }; readonly result: undefined };
}

export function isShellStatus(value: unknown): value is ShellStatus {
  if (!value || typeof value !== "object") return false;
  const v = value as Record<string, unknown>;
  return v.appName === "SAIKI" && typeof v.version === "string" && v.platform === "windows" &&
    ["starting", "ready", "exiting"].includes(String(v.lifecycle)) && typeof v.sessionId === "string" &&
    typeof v.startedAtUnixMs === "number";
}

export function validateFrontendLog(entry: FrontendLogEntry): void {
  if (!entry.event.trim() || entry.event.length > 80) throw new Error("Log event must be 1-80 characters.");
  if (!entry.message.trim() || entry.message.length > 500) throw new Error("Log message must be 1-500 characters.");
  if (entry.context && Object.keys(entry.context).length > 20) throw new Error("Log context is limited to 20 fields.");
}
