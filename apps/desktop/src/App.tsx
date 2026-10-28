import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { OverlayStatus, ShellStatus, WorkspaceMode, WorkspaceState } from "@saiki/ipc-contracts";
import { getOverlayStatus, getShellStatus, getWorkspaceState, moveOverlayToMonitor, requestAppExit, setOverlayPaused, setUiPreferences, setWorkspaceMode, writeFrontendLog } from "./ipc";
import { RigSurface } from "./character/RigSurface";
import { ModelView } from "./ModelView";
import { ConversationView } from "./ConversationView";
import { MemoryView } from "./MemoryView";
import { KnowledgeView } from "./KnowledgeView";
import { VoiceView } from "./VoiceView";
import "./styles.css";

type ViewState = { kind: "loading" } | { kind: "ready"; shell: ShellStatus; overlay: OverlayStatus; workspace: WorkspaceState } | { kind: "error"; message: string };
type WorkspaceTab = "activity" | "conversation" | "memory" | "knowledge" | "voice" | "models" | "settings" | "approvals";

function OverlaySurface() { return <main className="overlay-surface" aria-hidden="true"><RigSurface /></main>; }

function Controller() {
  const [state, setState] = useState<ViewState>({ kind: "loading" });
  const [tab, setTab] = useState<WorkspaceTab>("activity");
  useEffect(() => {
    let active = true;
    Promise.all([getReadyShellStatus(), getOverlayStatus(), getWorkspaceState()]).then(([shell, overlay, workspace]) => {
      if (active) setState({ kind: "ready", shell, overlay, workspace });
      return writeFrontendLog({ level: "info", event: "workspace.ready", message: "Phase 05 workspace initialized" });
    }).catch((error: unknown) => active && setState({ kind: "error", message: error instanceof Error ? error.message : String(error) }));
    return () => { active = false; };
  }, []);
  function setError(error: unknown): void { setState({ kind: "error", message: error instanceof Error ? error.message : String(error) }); }
  function updateOverlay(action: () => Promise<OverlayStatus>): void { void action().then((overlay) => setState((current) => current.kind === "ready" ? { ...current, overlay } : current)).catch(setError); }
  function updateWorkspace(action: () => Promise<WorkspaceState>): void { void action().then((workspace) => setState((current) => current.kind === "ready" ? { ...current, workspace } : current)).catch(setError); }
  if (state.kind === "loading") return <main className="controller status-panel"><p>Reading native workspace state…</p></main>;
  if (state.kind === "error") return <main className="controller status-panel"><p role="alert">Unavailable: {state.message}</p><button onClick={() => void requestAppExit()}>Exit</button></main>;
  const { shell, overlay, workspace } = state;
  if (workspace.mode === "floating") return <FloatingController overlay={overlay} onMode={(mode) => updateWorkspace(() => setWorkspaceMode(mode))} onPause={() => updateOverlay(() => setOverlayPaused(!overlay.paused))} />;
  return <main className="workspace-shell">
    <header className="workspace-header"><div className="brand"><span className="signal" /><b>SAIKI</b><small>OWNER WORKSPACE</small></div><div className="workspace-status"><span>{shell.lifecycle}</span><span>{overlay.paused ? "overlay paused" : "overlay active"}</span></div><button className="quiet-button" onClick={() => updateWorkspace(() => setWorkspaceMode("floating"))}>Minimize to floating</button></header>
    <nav className="workspace-nav" aria-label="Workspace sections">{(["activity", "conversation", "memory", "knowledge", "voice", "models", "settings", "approvals"] as const).map((item) => <button key={item} className={tab === item ? "active" : ""} aria-current={tab === item ? "page" : undefined} onClick={() => setTab(item)}>{item}{item === "approvals" && <span className="unavailable-dot" title="Unavailable" />}</button>)}</nav>
    <section className="workspace-content">{tab === "activity" && <ActivityView shell={shell} overlay={overlay} />}{tab === "conversation" && <ConversationView />}{tab === "memory" && <MemoryView />}{tab === "knowledge" && <KnowledgeView />}{tab === "voice" && <VoiceView />}{tab === "models" && <ModelView />}{tab === "settings" && <SettingsView overlay={overlay} workspace={workspace} onOverlay={updateOverlay} onWorkspace={updateWorkspace} />}{tab === "approvals" && <ApprovalInbox workspace={workspace} />}</section>
    <footer className="workspace-footer"><span>Local session {shell.sessionId.slice(0, 8)}</span><button className="danger-button" onClick={() => void requestAppExit()}>Exit SAIKI</button></footer>
  </main>;
}

async function getReadyShellStatus(): Promise<ShellStatus> {
  let status = await getShellStatus();
  for (let attempt = 0; status.lifecycle === "starting" && attempt < 8; attempt += 1) {
    await new Promise((resolve) => setTimeout(resolve, 25));
    status = await getShellStatus();
  }
  return status;
}

function FloatingController({ overlay, onMode, onPause }: { overlay: OverlayStatus; onMode: (mode: WorkspaceMode) => void; onPause: () => void }) {
  const monitor = overlay.monitors[overlay.activeMonitorIndex];
  return <main className="floating-controller"><header><span className="signal" /><b>SAIKI</b><small>FLOATING</small></header><div className="floating-state"><strong>{overlay.paused ? "PAUSED" : "CLICK-THROUGH"}</strong><span>{monitor?.name ?? "Monitor unavailable"} · {overlay.pauseHotkey}</span></div><div className="floating-actions"><button onClick={() => onMode("full")}>Open workspace</button><button onClick={onPause}>{overlay.paused ? "Resume overlay" : "Pause overlay"}</button><button onClick={() => void requestAppExit()}>Exit</button></div></main>;
}

function ActivityView({ shell, overlay }: { shell: ShellStatus; overlay: OverlayStatus }) {
  const active = overlay.monitors[overlay.activeMonitorIndex];
  const entries = [{ label: "Native shell", value: `${shell.lifecycle} · started ${new Date(shell.startedAtUnixMs).toLocaleTimeString()}` }, { label: "Overlay", value: overlay.paused ? "Paused and hidden" : "Visible and native click-through" }, { label: "Active display", value: active ? `${active.name ?? `Monitor ${active.index + 1}`} · ${active.size.width}×${active.size.height} · ${active.scaleFactor.toFixed(2)}×` : "Unavailable" }, { label: "Display topology", value: `${overlay.monitors.length} monitor${overlay.monitors.length === 1 ? "" : "s"} reported by Windows` }];
  return <div className="view-stack"><div className="section-heading"><div><p className="eyebrow">CURRENT SESSION</p><h1>Activity</h1></div><span className="live-badge">LIVE</span></div><div className="activity-list">{entries.map((entry) => <article key={entry.label}><span className="activity-marker" /><div><b>{entry.label}</b><p>{entry.value}</p></div></article>)}</div><p className="scope-note">Only verified in-process and native shell events appear here. Historical persistence begins in a later phase.</p></div>;
}

function SettingsView({ overlay, workspace, onOverlay, onWorkspace }: { overlay: OverlayStatus; workspace: WorkspaceState; onOverlay: (action: () => Promise<OverlayStatus>) => void; onWorkspace: (action: () => Promise<WorkspaceState>) => void }) {
  return <div className="view-stack"><div className="section-heading"><div><p className="eyebrow">OWNER CONTROLS</p><h1>Settings</h1></div></div><div className="settings-grid"><section><h2>Character</h2><label className="switch-row"><span><b>Reduced motion</b><small>Stops the demonstration sequence at idle.</small></span><input aria-label="Reduced motion" type="checkbox" checked={workspace.reducedMotion} onChange={(event) => onWorkspace(() => setUiPreferences(event.target.checked, workspace.speechBubbles))} /></label><label className="switch-row"><span><b>Speech bubbles</b><small>Shows truthful current animation state.</small></span><input aria-label="Speech bubbles" type="checkbox" checked={workspace.speechBubbles} onChange={(event) => onWorkspace(() => setUiPreferences(workspace.reducedMotion, event.target.checked))} /></label></section><section><h2>Overlay</h2><button className="setting-action" onClick={() => onOverlay(() => setOverlayPaused(!overlay.paused))}>{overlay.paused ? "Resume overlay" : "Pause overlay"}<span>{overlay.pauseHotkey}</span></button><div className="monitor-settings">{overlay.monitors.map((monitor) => <button key={monitor.index} className={monitor.index === overlay.activeMonitorIndex ? "active" : ""} onClick={() => onOverlay(() => moveOverlayToMonitor(monitor.index))}><b>{monitor.name ?? `Monitor ${monitor.index + 1}`}</b><span>{monitor.size.width}×{monitor.size.height}</span></button>)}</div></section></div><p className="scope-note">Preferences are process-local in Phase 05 and reset on restart.</p></div>;
}

function ApprovalInbox({ workspace }: { workspace: WorkspaceState }) {
  return <div className="view-stack"><div className="section-heading"><div><p className="eyebrow">OWNER AUTHORITY</p><h1>Approval inbox</h1></div><span className="unavailable-badge">UNAVAILABLE</span></div><div className="unavailable-panel"><div className="lock-mark">×</div><h2>No approval executor yet</h2><p>{workspace.approvals.reason}</p><small>No requests are fabricated or accepted. This view will only expose policy-bound requests after the executor and exact-payload approval contracts are implemented.</small></div></div>;
}

export function App() { return getCurrentWindow().label === "overlay" ? <OverlaySurface /> : <Controller />; }
