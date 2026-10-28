import assert from "node:assert/strict";
import { test } from "node:test";
import { IPC_COMMANDS, isOllamaModelList, isOllamaStreamEvent, isOverlayStatus, isShellStatus, isWorkspaceState, validateFrontendLog } from "../src/index.ts";

test("command names are stable and explicit", () => {
  assert.deepEqual(Object.values(IPC_COMMANDS), ["get_shell_status", "write_frontend_log", "request_app_exit", "get_overlay_status", "move_overlay_to_monitor", "set_overlay_paused", "get_workspace_state", "set_workspace_mode", "set_ui_preferences", "discover_ollama_models", "start_ollama_chat", "cancel_ollama_chat"]);
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
