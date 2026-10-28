import { useEffect, useState } from "react";
import type { ShellStatus } from "@saiki/ipc-contracts";
import { getShellStatus, requestAppExit, writeFrontendLog } from "./ipc";
import "./styles.css";

type ViewState = { kind: "loading" } | { kind: "ready"; status: ShellStatus } | { kind: "error"; message: string };

export function App() {
  const [state, setState] = useState<ViewState>({ kind: "loading" });
  const [exiting, setExiting] = useState(false);

  useEffect(() => {
    let active = true;
    getShellStatus().then((status) => {
      if (active) setState({ kind: "ready", status });
      return writeFrontendLog({ level: "info", event: "renderer.ready", message: "Phase 01 renderer initialized" });
    }).catch((error: unknown) => active && setState({ kind: "error", message: error instanceof Error ? error.message : "Native shell unavailable" }));
    return () => { active = false; };
  }, []);

  async function exit(): Promise<void> {
    setExiting(true);
    try { await requestAppExit(); }
    catch { setExiting(false); setState({ kind: "error", message: "The native shell did not accept the exit request." }); }
  }

  return <main className="shell">
    <header><span className="signal" aria-hidden="true" /><span>SAIKI</span><small>PHASE 01 · NATIVE SHELL</small></header>
    <section className="presence" aria-live="polite">
      <p className="eyebrow">SYSTEM PRESENCE</p>
      {state.kind === "loading" && <h1>Establishing local shell…</h1>}
      {state.kind === "error" && <><h1>Shell unavailable</h1><p className="detail">{state.message}</p></>}
      {state.kind === "ready" && <>
        <h1>Present.</h1>
        <p className="detail">Native core connected · session {state.status.sessionId.slice(0, 8)}</p>
        <dl><div><dt>CORE</dt><dd>{state.status.lifecycle.toUpperCase()}</dd></div><div><dt>PLATFORM</dt><dd>WINDOWS</dd></div><div><dt>VERSION</dt><dd>{state.status.version}</dd></div></dl>
      </>}
    </section>
    <footer><span>Local shell only. Autonomous capabilities remain unavailable.</span><button type="button" disabled={exiting} onClick={() => void exit()}>{exiting ? "Exiting…" : "Exit cleanly"}</button></footer>
  </main>;
}
