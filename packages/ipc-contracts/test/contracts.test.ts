import assert from "node:assert/strict";
import { test } from "node:test";
import { IPC_COMMANDS, isOverlayStatus, isShellStatus, validateFrontendLog } from "../src/index.ts";

test("command names are stable and explicit", () => {
  assert.deepEqual(Object.values(IPC_COMMANDS), ["get_shell_status", "write_frontend_log", "request_app_exit", "get_overlay_status", "move_overlay_to_monitor", "set_overlay_paused"]);
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
