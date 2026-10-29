import { useEffect, useRef, useState } from "react";
import "pixi.js/unsafe-eval";
import { Application, Container } from "pixi.js";
import { loadLayeredRigV2 } from "@saiki/character-rig";
import type { LayeredRigLoadResult, PresentationMode } from "@saiki/character-rig";
import { getWorkspaceState, writeFrontendLog } from "../ipc";
import { CHARACTER_DEBUG_STORAGE_KEY, CHARACTER_MANIFEST_URL } from "./asset-foundation";

type Status = { kind: "loading" } | { kind: "ready"; layers: number; mode: PresentationMode } | { kind: "unavailable"; message: string };
export function RigSurface() {
  const host = useRef<HTMLDivElement>(null); const [status, setStatus] = useState<Status>({ kind: "loading" });
  useEffect(() => {
    const element = host.current; if (!element) return;
    let cancelled = false; let rig: LayeredRigLoadResult | undefined; let timer: ReturnType<typeof setInterval> | undefined; let observer: ResizeObserver | undefined;
    const app = new Application(); const viewport = new Container({ label: "character-viewport" });
    void (async () => {
      try {
        await app.init({ backgroundAlpha: 0, antialias: true, resizeTo: element, preference: "webgl" });
        if (cancelled) { app.destroy(true); return; }
        element.append(app.canvas); app.stage.addChild(viewport);
        const response = await fetch(CHARACTER_MANIFEST_URL, { cache: "no-store" });
        if (!response.ok) throw new Error(`Production character artwork unavailable (${response.status}). Supply /character/saiki-kusuo/rig.v2.json and its licensed layers.`);
        rig = await loadLayeredRigV2(await response.json() as unknown, new URL(CHARACTER_MANIFEST_URL, window.location.href).toString()); viewport.addChild(rig.root);
        const resize = () => viewport.position.set(element.clientWidth / 2, element.clientHeight / 2); resize(); observer = new ResizeObserver(resize); observer.observe(element);
        const refresh = async () => { if (!rig) return; const workspace = await getWorkspaceState(); const mode: PresentationMode = workspace.mode === "full" ? "full_body" : "floating"; rig.setMode(mode); rig.debug.setEnabled(localStorage.getItem(CHARACTER_DEBUG_STORAGE_KEY) === "true"); if (!cancelled) setStatus({ kind: "ready", layers: rig.layers.size, mode }); };
        await refresh(); timer = setInterval(() => { void refresh().catch(() => undefined); }, 750);
        void writeFrontendLog({ level: "info", event: "rig.asset_package_loaded", message: `layers=${rig.layers.size} runtime=pixi-layered-v2` });
      } catch (error) { const reason = error instanceof Error ? error.message : String(error); const message = reason === "Failed to fetch" ? "Production character artwork is not installed. Supply the documented licensed layer package." : reason; if (!cancelled) setStatus({ kind: "unavailable", message }); void writeFrontendLog({ level: "warn", event: "rig.asset_package_unavailable", message: message.slice(0, 500) }); }
    })();
    return () => { cancelled = true; if (timer) clearInterval(timer); observer?.disconnect(); rig?.destroy(); if (app.renderer) app.destroy(true); };
  }, []);
  return <div ref={host} className="rig-surface" data-rig-status={status.kind} aria-hidden="true">
    {status.kind === "loading" && <span>CHECKING LAYERED CHARACTER ASSETS</span>}
    {status.kind === "unavailable" && <span>CHARACTER ARTWORK UNAVAILABLE<br />{status.message}<br />No placeholder substituted.</span>}
    {status.kind === "ready" && <div className="rig-telemetry"><b>LAYERED RIG READY</b><small>{status.layers} validated layers · {status.mode}</small></div>}
  </div>;
}
