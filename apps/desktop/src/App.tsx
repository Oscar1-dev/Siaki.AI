import { useEffect, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import type { OverlayStatus, ShellStatus } from "@saiki/ipc-contracts";
import { getOverlayStatus, getShellStatus, moveOverlayToMonitor, requestAppExit, setOverlayPaused, writeFrontendLog } from "./ipc";
import { RigSurface } from "./character/RigSurface";
import "./styles.css";

type ViewState = { kind: "loading" } | { kind: "ready"; shell: ShellStatus; overlay: OverlayStatus } | { kind: "error"; message: string };

function OverlaySurface() {
  return <main className="overlay-surface" aria-hidden="true"><RigSurface /></main>;
}

function Controller() {
  const [state, setState] = useState<ViewState>({ kind: "loading" });
  useEffect(() => {
    let active = true;
    Promise.all([getShellStatus(), getOverlayStatus()]).then(([shell, overlay]) => {
      if (active) setState({ kind: "ready", shell, overlay });
      return writeFrontendLog({ level: "info", event: "overlay.controller_ready", message: "Phase 02 overlay controller initialized" });
    }).catch((error: unknown) => active && setState({ kind: "error", message: error instanceof Error ? error.message : String(error) }));
    return () => { active = false; };
  }, []);
  async function update(action: () => Promise<OverlayStatus>): Promise<void> {
    try { const overlay = await action(); setState((current) => current.kind === "ready" ? { ...current, overlay } : current); }
    catch (error) { setState({ kind: "error", message: error instanceof Error ? error.message : String(error) }); }
  }
  return <main className="controller"><header><span className="signal" /><b>SAIKI</b><small>OVERLAY CONTROL</small></header>
    {state.kind === "loading" && <p>Reading monitor topology…</p>}
    {state.kind === "error" && <p role="alert">Unavailable: {state.message}</p>}
    {state.kind === "ready" && <><div className="mode"><strong>{state.overlay.paused ? "PAUSED" : "CLICK-THROUGH"}</strong><span>{state.overlay.monitors.length} monitors · {state.overlay.pauseHotkey}</span></div>
      <div className="monitor-list">{state.overlay.monitors.map((monitor) => <button key={monitor.index} className={monitor.index === state.overlay.activeMonitorIndex ? "active" : ""} onClick={() => void update(() => moveOverlayToMonitor(monitor.index))}><b>{monitor.name ?? `Monitor ${monitor.index + 1}`}</b><span>{monitor.size.width}×{monitor.size.height} · {monitor.scaleFactor.toFixed(2)}×</span></button>)}</div>
      <footer><button onClick={() => void update(() => setOverlayPaused(!state.overlay.paused))}>{state.overlay.paused ? "Resume overlay" : "Pause overlay"}</button><button onClick={() => void requestAppExit()}>Exit</button></footer></>}
  </main>;
}

export function App() { return getCurrentWindow().label === "overlay" ? <OverlaySurface /> : <Controller />; }
