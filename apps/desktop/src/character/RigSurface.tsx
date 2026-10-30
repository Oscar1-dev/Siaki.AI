import { useEffect, useRef, useState } from "react";
import "pixi.js/unsafe-eval";
import { Application, Container } from "pixi.js";
import { loadLayeredRigV2, loadReferenceMotionPreview } from "@saiki/character-rig";
import type { AnimationRequest, LayeredRigLoadResult, PresentationMode, ReferenceMotionPreview } from "@saiki/character-rig";
import { getWorkspaceState, writeFrontendLog } from "../ipc";
import { CHARACTER_DEBUG_STORAGE_KEY, CHARACTER_MANIFEST_URL, CHARACTER_REFERENCE_PREVIEW_URL } from "./asset-foundation";
import "./reference-preview.css";

type Status = { kind: "loading" } | { kind: "ready"; layers: number; mode: PresentationMode } | { kind: "reference_preview"; motion: string } | { kind: "unavailable"; message: string };
const PREVIEW_SEQUENCE: readonly AnimationRequest[] = [{ state: "idle" }, { state: "blink" }, { state: "gaze", gaze: { x: 0.45, y: -0.2 } }, { state: "walk", direction: 1 }, { state: "teleport" }];

export function RigSurface() {
  const host = useRef<HTMLDivElement>(null); const [status, setStatus] = useState<Status>({ kind: "loading" });
  useEffect(() => {
    const element = host.current; if (!element) return;
    let cancelled = false; let rig: LayeredRigLoadResult | undefined; let preview: ReferenceMotionPreview | undefined; let refreshTimer: ReturnType<typeof setInterval> | undefined; let sequenceTimer: ReturnType<typeof setInterval> | undefined; let observer: ResizeObserver | undefined; let sequenceIndex = 0;
    const app = new Application(); const viewport = new Container({ label: "character-viewport" });
    void (async () => {
      try {
        await app.init({ backgroundAlpha: 0, antialias: true, resizeTo: element, preference: "webgl" });
        if (cancelled) { app.destroy(true); return; }
        element.append(app.canvas); app.stage.addChild(viewport);
        let response: Response | undefined;
        try { response = await fetch(CHARACTER_MANIFEST_URL, { cache: "no-store" }); } catch { response = undefined; }
        const contentType = response?.headers.get("content-type")?.toLowerCase() ?? "";
        const manifestMissing = !response || response.status === 404 || response.status === 0 || (response.ok && contentType.includes("text/html"));
        if (response?.ok && !manifestMissing) {
          rig = await loadLayeredRigV2(await response.json() as unknown, new URL(CHARACTER_MANIFEST_URL, window.location.href).toString()); viewport.addChild(rig.root);
          const resize = () => viewport.position.set(element.clientWidth / 2, element.clientHeight / 2); resize(); observer = new ResizeObserver(resize); observer.observe(element);
          const refresh = async () => { if (!rig) return; const workspace = await getWorkspaceState(); const mode: PresentationMode = workspace.mode === "full" ? "full_body" : "floating"; rig.setMode(mode); rig.debug.setEnabled(localStorage.getItem(CHARACTER_DEBUG_STORAGE_KEY) === "true"); if (!cancelled) setStatus({ kind: "ready", layers: rig.layers.size, mode }); };
          await refresh(); refreshTimer = setInterval(() => { void refresh().catch(() => undefined); }, 750);
          void writeFrontendLog({ level: "info", event: "rig.asset_package_loaded", message: `layers=${rig.layers.size} runtime=pixi-layered-v2` });
        } else if (manifestMissing) {
          const previewResponse = await fetch(CHARACTER_REFERENCE_PREVIEW_URL, { cache: "no-store" });
          if (!previewResponse.ok) throw new Error(`Production artwork and reference preview are unavailable (${previewResponse.status}).`);
          preview = await loadReferenceMotionPreview(await previewResponse.json() as unknown, new URL(CHARACTER_REFERENCE_PREVIEW_URL, window.location.href).toString());
          viewport.addChild(preview.root);
          const resize = () => { viewport.position.set(element.clientWidth / 2, element.clientHeight * 0.88); preview?.resize(element.clientWidth, element.clientHeight); }; resize(); observer = new ResizeObserver(resize); observer.observe(element);
          app.ticker.add((ticker) => { if (!cancelled && preview) { const pose = preview.update(ticker.deltaMS); setStatus((current) => current.kind === "reference_preview" && current.motion === pose.state ? current : { kind: "reference_preview", motion: pose.state }); } });
          preview.request(PREVIEW_SEQUENCE[sequenceIndex]!);
          sequenceTimer = setInterval(() => { if (!preview) return; sequenceIndex = (sequenceIndex + 1) % PREVIEW_SEQUENCE.length; preview.request(PREVIEW_SEQUENCE[sequenceIndex]!); }, 2_400);
          void writeFrontendLog({ level: "info", event: "rig.reference_preview_loaded", message: "Checksum-verified non-production reference motion preview loaded; layered rig remains unavailable." });
        } else {
          throw new Error(`Production character manifest is unavailable (${response?.status ?? "fetch error"}); reference fallback was not used for an unexpected response.`);
        }
      } catch (error) { const reason = error instanceof Error ? error.message : String(error); const message = reason === "Failed to fetch" ? "Production character artwork is not installed. Supply the documented licensed layer package." : reason; if (!cancelled) setStatus({ kind: "unavailable", message }); void writeFrontendLog({ level: "warn", event: "rig.asset_package_unavailable", message: message.slice(0, 500) }); }
    })();
    return () => { cancelled = true; if (refreshTimer) clearInterval(refreshTimer); if (sequenceTimer) clearInterval(sequenceTimer); observer?.disconnect(); rig?.destroy(); preview?.destroy(); if (app.renderer) app.destroy(true); };
  }, []);
  return <div ref={host} className="rig-surface" data-rig-status={status.kind} aria-hidden="true">
    {status.kind === "loading" && <span>CHECKING LAYERED CHARACTER ASSETS</span>}
    {status.kind === "unavailable" && <span>CHARACTER ARTWORK UNAVAILABLE<br />{status.message}<br />No placeholder substituted.</span>}
    {status.kind === "reference_preview" && <div className="rig-telemetry preview"><b>REFERENCE MOTION PREVIEW</b><small>non-production · license unresolved · {status.motion}</small></div>}
    {status.kind === "ready" && <div className="rig-telemetry"><b>LAYERED RIG READY</b><small>{status.layers} validated layers · {status.mode}</small></div>}
  </div>;
}
