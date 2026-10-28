import { useEffect, useRef, useState } from "react";
import "pixi.js/unsafe-eval";
import { Application } from "pixi.js";
import { FrameBudgetTracker, PlaceholderPixiRig, validateRigManifest } from "@saiki/character-rig";
import type { AnimationRequest, CharacterAnimationState, FrameBudgetSnapshot } from "@saiki/character-rig";
import { getWorkspaceState, writeFrontendLog } from "../ipc";
import { PLACEHOLDER_RIG_MANIFEST } from "./placeholder-manifest";

type Status = { kind: "loading" } | { kind: "ready" } | { kind: "unavailable"; message: string };
type Telemetry = { state: CharacterAnimationState; budget: FrameBudgetSnapshot };
type Preferences = { reducedMotion: boolean; speechBubbles: boolean };

const DEMO_SEQUENCE: readonly AnimationRequest[] = [
  { state: "idle" },
  { state: "gaze", gaze: { x: 1, y: -0.5 } },
  { state: "blink" },
  { state: "walk", direction: 1 },
  { state: "sit" },
  { state: "teleport" },
];

export function RigSurface() {
  const host = useRef<HTMLDivElement>(null);
  const [status, setStatus] = useState<Status>({ kind: "loading" });
  const [telemetry, setTelemetry] = useState<Telemetry | null>(null);
  const [preferences, setPreferences] = useState<Preferences>({ reducedMotion: false, speechBubbles: true });

  useEffect(() => {
    const element = host.current;
    if (!element) return;
    let cancelled = false;
    const app = new Application();
    const rig = new PlaceholderPixiRig();
    const budget = new FrameBudgetTracker();
    let observer: ResizeObserver | undefined;
    let preferenceTimer: ReturnType<typeof setInterval> | undefined;
    let currentPreferences: Preferences = { reducedMotion: false, speechBubbles: true };
    let frameCount = 0;
    let sequenceIndex = 0;
    let sequenceElapsedMs = 0;
    let loggedBudget = false;

    void (async () => {
      try {
        await app.init({ backgroundAlpha: 0, antialias: true, resizeTo: element, preference: "webgl" });
        if (cancelled) { app.destroy(true); return; }
        const manifest = validateRigManifest(PLACEHOLDER_RIG_MANIFEST);
        await rig.load(manifest);
        element.append(app.canvas);
        app.stage.addChild(rig.view);
        rig.resize(element.clientWidth, element.clientHeight);
        rig.transition(DEMO_SEQUENCE[0]!);
        const refreshPreferences = async () => {
          const workspace = await getWorkspaceState();
          currentPreferences = { reducedMotion: workspace.reducedMotion, speechBubbles: workspace.speechBubbles };
          if (!cancelled) setPreferences(currentPreferences);
        };
        await refreshPreferences();
        preferenceTimer = setInterval(() => { void refreshPreferences().catch(() => undefined); }, 750);
        app.ticker.add((ticker) => {
          sequenceElapsedMs += ticker.deltaMS;
          if (currentPreferences.reducedMotion && rig.snapshot().state !== "idle") {
            sequenceElapsedMs = 0;
            sequenceIndex = 0;
            rig.transition({ state: "idle" });
          } else if (!currentPreferences.reducedMotion && sequenceElapsedMs >= 1_800) {
            sequenceElapsedMs = 0;
            sequenceIndex = (sequenceIndex + 1) % DEMO_SEQUENCE.length;
            rig.transition(DEMO_SEQUENCE[sequenceIndex]!);
          }
          const started = performance.now();
          const pose = rig.update(ticker.deltaMS);
          budget.record(performance.now() - started, ticker.deltaMS);
          frameCount += 1;
          if (frameCount % 120 === 0 && !cancelled) {
            const snapshot = budget.snapshot();
            setTelemetry({ state: pose.state, budget: snapshot });
            if (!loggedBudget && snapshot.sampleCount >= 240) {
              loggedBudget = true;
              void writeFrontendLog({ level: "info", event: "rig.frame_budget", message: `samples=${snapshot.sampleCount} update_p95_ms=${snapshot.updateP95Ms.toFixed(3)} frame_p95_ms=${snapshot.frameP95Ms.toFixed(3)} overruns=${snapshot.framesOverBudget}` });
            }
          }
        });
        observer = new ResizeObserver(() => rig.resize(element.clientWidth, element.clientHeight));
        observer.observe(element);
        setStatus({ kind: "ready" });
      } catch (error) {
        if (!cancelled) setStatus({ kind: "unavailable", message: error instanceof Error ? error.message : String(error) });
      }
    })();

    return () => {
      cancelled = true;
      observer?.disconnect();
      if (preferenceTimer) clearInterval(preferenceTimer);
      if (app.renderer) {
        rig.destroy();
        app.destroy(true);
      }
    };
  }, []);

  return <div ref={host} className="rig-surface" data-rig-status={status.kind} aria-hidden="true">
    {status.kind === "unavailable" && <span>RIG UNAVAILABLE<br />{status.message}</span>}
    {status.kind === "ready" && <div className="rig-telemetry"><b>{telemetry?.state ?? "idle"}</b><small>{telemetry ? `UPDATE P95 ${telemetry.budget.updateP95Ms.toFixed(2)}MS · FRAME P95 ${telemetry.budget.frameP95Ms.toFixed(2)}MS` : "MEASURING FRAME BUDGET"}</small></div>}
    {status.kind === "ready" && preferences.speechBubbles && <div className="speech-bubble">Current state: {telemetry?.state ?? "idle"}</div>}
  </div>;
}
