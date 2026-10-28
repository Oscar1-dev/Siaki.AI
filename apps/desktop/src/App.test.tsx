import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, expect, test, vi } from "vitest";
import { App } from "./App";

const invoke = vi.fn();
vi.mock("@tauri-apps/api/core", () => ({ invoke: (...args: unknown[]) => invoke(...args) }));
vi.mock("@tauri-apps/api/window", () => ({ getCurrentWindow: () => ({ label: "main" }) }));
vi.mock("@tauri-apps/api/event", () => ({ listen: () => Promise.resolve(() => undefined) }));
const shell = { appName: "SAIKI", version: "0.1.0", platform: "windows", lifecycle: "ready", sessionId: "123456789", startedAtUnixMs: 1 };
const overlay = { paused: false, activeMonitorIndex: 0, pauseHotkey: "Ctrl+Shift+F12", monitors: [{ index: 0, name: "DISPLAY1", position: { x: 0, y: 0 }, size: { width: 2560, height: 1440 }, scaleFactor: 1 }] };
const workspace = { mode: "floating", reducedMotion: false, speechBubbles: true, approvals: { availability: "unavailable", reason: "Policy-gated execution is not implemented in the completed phases.", pending: [] } };
const cloud = { enabled: false, model: "", credentialPresent: false, credentialStore: "windows_credential_manager", endpoint: "https://api.openai.com/v1/responses", disclosure: { destination: "OpenAI API (api.openai.com)", data: ["the message", "the model"], storageRequested: false, billing: "API usage may incur charges; ChatGPT subscriptions are not assumed." } };
const conversation = { persona: { name: "Saiki", authority: "none", tone: ["dry"], preamble: "This persona controls tone only.", expressions: ["neutral", "talking"] }, messages: [{ id: "t1", role: "tool_result", content: "deploy exit code 1", createdAtUnixMs: 1 }], limits: { maxStoredMessages: 200, maxContextMessages: 24, maxContextBytes: 24576, maxContentBytes: 32768 } };

afterEach(cleanup);

beforeEach(() => {
  invoke.mockReset();
  invoke.mockImplementation((command: string) => command === "get_shell_status" ? Promise.resolve(shell) : command === "get_overlay_status" ? Promise.resolve(overlay) : command === "get_workspace_state" ? Promise.resolve(workspace) : command === "set_workspace_mode" ? Promise.resolve({ ...workspace, mode: "full" }) : command === "set_ui_preferences" ? Promise.resolve({ ...workspace, mode: "full", reducedMotion: true }) : command === "get_cloud_status" ? Promise.resolve(cloud) : command === "get_conversation" ? Promise.resolve(conversation) : Promise.resolve(undefined));
});

test("conversation persona is non-authoritative and tool results render verbatim", async () => {
  render(<App />);
  fireEvent.click(await screen.findByRole("button", { name: "Open workspace" }));
  fireEvent.click(await screen.findByRole("button", { name: /conversation/i }));
  expect(await screen.findByRole("heading", { name: "Talk to Saiki" })).toBeInTheDocument();
  expect(screen.getByText("AUTHORITY: NONE")).toBeInTheDocument();
  expect(screen.getByText("Tool result (factual)")).toBeInTheDocument();
  expect(screen.getByText("deploy exit code 1")).toBeInTheDocument();
});

test("cloud provider is disabled and discloses external transfer without a credential", async () => {
  render(<App />);
  fireEvent.click(await screen.findByRole("button", { name: "Open workspace" }));
  fireEvent.click(await screen.findByRole("button", { name: /models/i }));
  expect(await screen.findByRole("heading", { name: "OpenAI API" })).toBeInTheDocument();
  expect(screen.getByText("DISABLED")).toBeInTheDocument();
  expect(screen.getByText(/ChatGPT subscriptions are not assumed/i)).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Enable OpenAI" })).toBeDisabled();
});

test("renders verified floating overlay state", async () => {
  render(<App />);
  expect(await screen.findByText("CLICK-THROUGH")).toBeInTheDocument();
  expect(screen.getByText(/DISPLAY1/)).toBeInTheDocument();
  expect(screen.getByText(/Ctrl\+Shift\+F12/)).toBeInTheDocument();
});

test("full workspace controls are real or explicitly unavailable", async () => {
  render(<App />);
  fireEvent.click(await screen.findByRole("button", { name: "Open workspace" }));
  expect(await screen.findByRole("heading", { name: "Activity" })).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: /settings/i }));
  fireEvent.click(screen.getByRole("checkbox", { name: "Reduced motion" }));
  await waitFor(() => expect(invoke).toHaveBeenCalledWith("set_ui_preferences", { preferences: { reducedMotion: true, speechBubbles: true } }));
  fireEvent.click(screen.getByRole("button", { name: /approvals/i }));
  expect(screen.getByText("UNAVAILABLE")).toBeInTheDocument();
  expect(screen.queryByRole("button", { name: /approve/i })).not.toBeInTheDocument();
});

test("reports invalid overlay contracts honestly", async () => {
  invoke.mockImplementation((command: string) => command === "get_shell_status" ? Promise.resolve(shell) : command === "get_workspace_state" ? Promise.resolve(workspace) : Promise.resolve({ paused: false }));
  render(<App />);
  expect(await screen.findByText(/Unavailable:/)).toBeInTheDocument();
  expect(screen.getByText(/invalid overlay status contract/i)).toBeInTheDocument();
});
