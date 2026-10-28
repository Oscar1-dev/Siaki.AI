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
const memoryClaim = { id: "m1", claim: "the owner prefers metric units", source: "owner", confidence: 0.8, sensitivity: "private", status: "active", supersedes: null, createdAtUnixMs: 1, updatedAtUnixMs: 1 };
const doc = { id: "d1", title: "handbook.txt", path: "C:/docs/handbook.txt", sensitivity: "private", byteLen: 120, chunkCount: 3, ingestedAtUnixMs: 1 };

afterEach(cleanup);

beforeEach(() => {
  invoke.mockReset();
  invoke.mockImplementation((command: string) => command === "get_shell_status" ? Promise.resolve(shell) : command === "get_overlay_status" ? Promise.resolve(overlay) : command === "get_workspace_state" ? Promise.resolve(workspace) : command === "set_workspace_mode" ? Promise.resolve({ ...workspace, mode: "full" }) : command === "set_ui_preferences" ? Promise.resolve({ ...workspace, mode: "full", reducedMotion: true }) : command === "get_cloud_status" ? Promise.resolve(cloud) : command === "get_conversation" ? Promise.resolve(conversation) : command === "list_memory" ? Promise.resolve([memoryClaim]) : command === "list_documents" ? Promise.resolve([doc]) : Promise.resolve(undefined));
});

test("knowledge view lists ingested documents with their real path", async () => {
  render(<App />);
  fireEvent.click(await screen.findByRole("button", { name: "Open workspace" }));
  fireEvent.click(await screen.findByRole("button", { name: /knowledge/i }));
  expect(await screen.findByRole("heading", { name: "Document knowledge" })).toBeInTheDocument();
  expect(screen.getByText("handbook.txt")).toBeInTheDocument();
  expect(screen.getByText("C:/docs/handbook.txt")).toBeInTheDocument();
  expect(screen.getByText("3 chunks")).toBeInTheDocument();
});

test("memory view shows persisted claims with provenance", async () => {
  render(<App />);
  fireEvent.click(await screen.findByRole("button", { name: "Open workspace" }));
  fireEvent.click(await screen.findByRole("button", { name: /memory/i }));
  expect(await screen.findByText("the owner prefers metric units")).toBeInTheDocument();
  expect(screen.getByText("source: owner")).toBeInTheDocument();
  expect(screen.getByText("confidence: 0.80")).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Forget" })).toBeInTheDocument();
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
