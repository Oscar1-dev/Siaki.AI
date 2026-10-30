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
const voice = { sttModel: { configured: false, present: false, path: null }, ttsVoice: { configured: false, present: false, path: null }, microphone: { available: false, reason: "Microphone capture is not implemented in this build." }, playback: { available: false, reason: "Audio playback is not implemented in this build." }, listeningAvailable: false, speechAudioAvailable: false, lipSyncAvailable: true, session: "idle" };
const policyState = { killSwitchEngaged: false, grants: [], pendingApprovals: [] };
const sysTools = { workspaceRoot: null, workspaceConfigured: false, workspacePresent: false, allowedApps: [], metrics: { cpuCount: 8, os: "windows", arch: "x86_64", pointerWidthBits: 64 } };
const devTools = { repoRoot: null, repoConfigured: false, isGitRepo: false, allowedCommands: [], vscodeConfigured: false, vscodePresent: false, codex: { available: false, reason: "No supported local Codex activity source is available in this build." } };

afterEach(cleanup);

beforeEach(() => {
  invoke.mockReset();
  invoke.mockImplementation((command: string) => command === "get_shell_status" ? Promise.resolve(shell) : command === "get_overlay_status" ? Promise.resolve(overlay) : command === "get_workspace_state" ? Promise.resolve(workspace) : command === "set_workspace_mode" ? Promise.resolve({ ...workspace, mode: "full" }) : command === "set_ui_preferences" ? Promise.resolve({ ...workspace, mode: "full", reducedMotion: true }) : command === "get_cloud_status" ? Promise.resolve(cloud) : command === "get_conversation" ? Promise.resolve(conversation) : command === "list_memory" ? Promise.resolve([memoryClaim]) : command === "list_documents" ? Promise.resolve([doc]) : command === "get_voice_status" ? Promise.resolve(voice) : command === "get_policy_state" ? Promise.resolve(policyState) : command === "list_policy_audit" ? Promise.resolve([]) : command === "get_systools_status" ? Promise.resolve(sysTools) : command === "get_devtools_status" ? Promise.resolve(devTools) : Promise.resolve(undefined));
});

test("devtools view bounds the shell and reports Codex unavailable", async () => {
  render(<App />);
  fireEvent.click(await screen.findByRole("button", { name: "Open workspace" }));
  fireEvent.click(await screen.findByRole("button", { name: /devtools/i }));
  expect(await screen.findByRole("heading", { name: "Terminal & dev tools" })).toBeInTheDocument();
  expect(screen.getByText(/No commands allowed/i)).toBeInTheDocument();
  expect(screen.getByText(/Codex unavailable/i)).toBeInTheDocument();
});

test("browser view reports the real blocker and denies page-authorized sensitive action", async () => {
  const previous = invoke.getMockImplementation();
  invoke.mockImplementation((command: string, args?: unknown) => {
    if (command === "get_browser_status") return Promise.resolve({ availability: "unavailable", reason: "Playwright sidecar is not provisioned.", sidecarPresent: false, enginePresent: false, profileIsolation: "dedicated_non_owner_profile", contentTrust: "untrusted_web_content" });
    if (command === "list_browser_tasks") return Promise.resolve([]);
    if (command === "evaluate_browser_step") return Promise.resolve({ outcome: "denied", reason: "Untrusted page content cannot authorize or initiate a sensitive browser action.", executed: false });
    return previous?.(command, args);
  });
  render(<App />);
  fireEvent.click(await screen.findByRole("button", { name: "Open workspace" }));
  fireEvent.click(await screen.findByRole("button", { name: "browser" }));
  expect(await screen.findByText("UNAVAILABLE")).toBeInTheDocument();
  fireEvent.click(screen.getByRole("button", { name: "Test untrusted sensitive proposal" }));
  expect(await screen.findByText(/Untrusted page content cannot authorize/i)).toBeInTheDocument();
});

test("scheduler view exposes durable wake and retry state", async () => {
  const previous = invoke.getMockImplementation();
  invoke.mockImplementation((command: string, args?: unknown) => command === "get_scheduler_snapshot" ? Promise.resolve({ tasks: [], notifications: [], wakeTrigger: "native_core_1s_tick", retryPolicy: "durable_exponential_backoff_max_8" }) : previous?.(command, args));
  render(<App />);
  fireEvent.click(await screen.findByRole("button", { name: "Open workspace" }));
  fireEvent.click(await screen.findByRole("button", { name: "scheduler" }));
  expect(await screen.findByRole("heading", { name: "Reminders & scheduler" })).toBeInTheDocument();
  expect(screen.getByText(/native_core_1s_tick/)).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Schedule" })).toBeInTheDocument();
});

test("world view reports reconstruction consistency and lifecycle version",async()=>{
 const previous=invoke.getMockImplementation();
 invoke.mockImplementation((command:string,args?:unknown)=>command==="get_world_snapshot"?Promise.resolve({projects:[{id:"p",title:"SAIKI",summary:"Build",lifecycle:"active",version:2,createdAtUnixMs:1,updatedAtUnixMs:2}],eventCount:2,consistent:true,inconsistencies:[]}):command==="list_world_events"?Promise.resolve([]):previous?.(command,args));
 render(<App/>);fireEvent.click(await screen.findByRole("button",{name:"Open workspace"}));fireEvent.click(await screen.findByRole("button",{name:"world"}));
 expect(await screen.findByText("RECONSTRUCTION CONSISTENT")).toBeInTheDocument();expect(screen.getByText("active · v2")).toBeInTheDocument();expect(screen.getByText("2 persisted events")).toBeInTheDocument();
});

test("supervisor exposes pause and fixed internal quotas",async()=>{
 const previous=invoke.getMockImplementation();const supervisor={paused:false,goals:[],tasks:[],tasksPerWake:3,tasksPerHour:20,capability:"internal_world_integrity_audit_only"};invoke.mockImplementation((command:string,args?:unknown)=>command==="get_supervisor_snapshot"?Promise.resolve(supervisor):previous?.(command,args));
 render(<App/>);fireEvent.click(await screen.findByRole("button",{name:"Open workspace"}));fireEvent.click(await screen.findByRole("button",{name:"supervisor"}));expect(await screen.findByRole("heading",{name:"Autonomous supervisor"})).toBeInTheDocument();expect(screen.getByRole("button",{name:"Pause supervisor"})).toBeInTheDocument();expect(screen.getByText(/internal_world_integrity_audit_only/)).toBeInTheDocument();
});

test("planning view explains task choice and shows evidence",async()=>{const previous=invoke.getMockImplementation();const planning={goals:[{id:"g",objective:"Coherence",successCriterion:"consistent",status:"completed",currentPlanVersion:1,createdAtUnixMs:1,updatedAtUnixMs:2}],plans:[{id:"p",goalId:"g",version:1,rationale:"Verify first",status:"completed",createdAtUnixMs:1}],tasks:[{id:"t",planId:"p",ordinal:1,taskType:"world_integrity_audit",status:"completed",selectionReason:"This read-only audit is the lowest-risk task.",evidence:{consistent:true},createdAtUnixMs:1,completedAtUnixMs:2}],reviews:[{id:"r",goalId:"g",planVersion:1,taskId:"t",verdict:"stop_success",reason:"criterion satisfied",evidenceDigest:"sha256:abc",createdAtUnixMs:2}],capability:"internal_world_review_only"};invoke.mockImplementation((command:string,args?:unknown)=>command==="get_planning_snapshot"?Promise.resolve(planning):previous?.(command,args));render(<App/>);fireEvent.click(await screen.findByRole("button",{name:"Open workspace"}));fireEvent.click(await screen.findByRole("button",{name:"planning"}));expect(await screen.findByText(/lowest-risk task/)).toBeInTheDocument();expect(screen.getByText(/"consistent": true/)).toBeInTheDocument();expect(screen.getByText("stop_success")).toBeInTheDocument();});

test("identity registry has no secret input and reports provisioning unavailable",async()=>{const previous=invoke.getMockImplementation();invoke.mockImplementation((command:string,args?:unknown)=>command==="get_identity_registry"?Promise.resolve({identities:[],secretInputSupported:false,vaultBackend:"windows_credential_manager_reference",credentialProvisioning:"unavailable"}):previous?.(command,args));render(<App/>);fireEvent.click(await screen.findByRole("button",{name:"Open workspace"}));fireEvent.click(await screen.findByRole("button",{name:"identities"}));expect(await screen.findByText("UNAVAILABLE CREDENTIAL PROVISIONING")).toBeInTheDocument();expect(screen.getByText(/No password, token, cookie, recovery code, or secret input exists/)).toBeInTheDocument();expect(screen.queryByLabelText(/password|token|secret/i)).not.toBeInTheDocument();});

test("onboarding view exposes manual checklist without automation",async()=>{const previous=invoke.getMockImplementation();const identity={id:"i",provider:"GitHub",accountLabel:"owner",loginHint:"owner",ownerProofMethod:"owner_attestation",ownerVerifiedAtUnixMs:1,vaultReference:"wincred:x",vaultBinding:"reference_only",credentialPresent:false,credentialReason:"unavailable",mfaEnabled:false,mfaMethods:[],recoveryControlConfirmed:true,status:"active",createdAtUnixMs:1,updatedAtUnixMs:1};invoke.mockImplementation((command:string,args?:unknown)=>command==="get_account_workflows"?Promise.resolve({workflows:[],audit:[],automation:"manual_owner_confirmed_only",secretInputSupported:false}):command==="get_identity_registry"?Promise.resolve({identities:[identity],secretInputSupported:false,vaultBackend:"windows_credential_manager_reference",credentialProvisioning:"unavailable"}):previous?.(command,args));render(<App/>);fireEvent.click(await screen.findByRole("button",{name:"Open workspace"}));fireEvent.click(await screen.findByRole("button",{name:"onboarding"}));expect(await screen.findByText("manual_owner_confirmed_only")).toBeInTheDocument();expect(screen.getByText(/No provider automation or credential input/)).toBeInTheDocument();expect(screen.getByRole("button",{name:"Create manual checklist"})).toBeEnabled();});

test("experiment view labels finances unavailable and never fabricates revenue", async () => {
  const previous = invoke.getMockImplementation();
  invoke.mockImplementation((command: string, args?: unknown) => command === "get_experiments" ? Promise.resolve({ experiments: [{ id: "e", idea: "Measured trial", hypothesis: "A causes B", stage: "validation", budgetCurrency: "GBP", budgetLimitMinor: 5000, actualSpendMinor: null, spendTracking: "unavailable", createdAtUnixMs: 1, updatedAtUnixMs: 1, notes: [], kpis: [], evaluations: [] }], financialExecution: "unavailable", revenuePolicy: "observed_evidence_only_never_inferred" }) : previous?.(command, args));
  render(<App />);
  fireEvent.click(await screen.findByRole("button", { name: "Open workspace" }));
  fireEvent.click(await screen.findByRole("button", { name: "experiments" }));
  expect(await screen.findByText(/Revenue is never inferred/i)).toBeInTheDocument();
  expect(screen.getByText(/Financial execution: unavailable/i)).toBeInTheDocument();
  expect(screen.getByText(/actual spend unknown\/unavailable/i)).toBeInTheDocument();
});

test("character foundation reports missing artwork without a placeholder", async () => {
  const originalFetch = globalThis.fetch;
  globalThis.fetch = vi.fn().mockResolvedValue({ ok: false, status: 404 }) as typeof fetch;
  try {
    render(<App />);
    fireEvent.click(await screen.findByRole("button", { name: "Open workspace" }));
    fireEvent.click(await screen.findByRole("button", { name: "character" }));
    expect(await screen.findByText(/Layered artwork is not installed/i)).toBeInTheDocument();
    expect(screen.getByText(/procedural capsule is not loaded/i)).toBeInTheDocument();
    expect(screen.getByRole("checkbox", { name: /Layer debug overlay/i })).toBeDisabled();
    expect(screen.getByText(/source\/saiki-kusuo-master.psd/i)).toBeInTheDocument();
    expect(screen.getByText(/REFERENCE CANDIDATE/i)).toBeInTheDocument();
    expect(screen.getByAltText(/full-body Saiki character reference/i)).toHaveAttribute("src", "/character/saiki-kusuo/reference/reference-clean-v1.webp");
  } finally { globalThis.fetch = originalFetch; }
});

test("system view confines file tools and launches only allowlisted apps", async () => {
  render(<App />);
  fireEvent.click(await screen.findByRole("button", { name: "Open workspace" }));
  fireEvent.click(await screen.findByRole("button", { name: /system/i }));
  expect(await screen.findByRole("heading", { name: "System & files" })).toBeInTheDocument();
  expect(screen.getByText(/confined to the configured workspace root/i)).toBeInTheDocument();
  expect(screen.getByText(/No applications allowed/i)).toBeInTheDocument();
});

test("policy view is deny-by-default with an owner kill switch", async () => {
  render(<App />);
  fireEvent.click(await screen.findByRole("button", { name: "Open workspace" }));
  fireEvent.click(await screen.findByRole("button", { name: /policy/i }));
  expect(await screen.findByRole("heading", { name: "Policy engine" })).toBeInTheDocument();
  expect(screen.getByText(/only this engine authorizes it/i)).toBeInTheDocument();
  expect(screen.getByText(/No active grants/i)).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "ENGAGE KILL SWITCH" })).toBeInTheDocument();
});

test("voice view shows honest unavailable capture and disabled push-to-talk", async () => {
  render(<App />);
  fireEvent.click(await screen.findByRole("button", { name: "Open workspace" }));
  fireEvent.click(await screen.findByRole("button", { name: /voice/i }));
  expect(await screen.findByRole("heading", { name: "Voice" })).toBeInTheDocument();
  expect(screen.getByText("MIC: UNAVAILABLE")).toBeInTheDocument();
  expect(screen.getByRole("button", { name: "Push to talk" })).toBeDisabled();
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
