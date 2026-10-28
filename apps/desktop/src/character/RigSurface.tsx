import { useEffect, useRef, useState } from "react";
import "pixi.js/unsafe-eval";
import { Application } from "pixi.js";
import { PlaceholderPixiRig, validateRigManifest } from "@saiki/character-rig";
import { PLACEHOLDER_RIG_MANIFEST } from "./placeholder-manifest";

type Status = { kind: "loading" } | { kind: "ready" } | { kind: "unavailable"; message: string };

export function RigSurface() {
  const host = useRef<HTMLDivElement>(null);
  const [status, setStatus] = useState<Status>({ kind: "loading" });

  useEffect(() => {
    const element = host.current;
    if (!element) return;
    let cancelled = false;
    const app = new Application();
    const rig = new PlaceholderPixiRig();
    let observer: ResizeObserver | undefined;

    void (async () => {
      try {
        await app.init({ backgroundAlpha: 0, antialias: true, resizeTo: element, preference: "webgl" });
        if (cancelled) { app.destroy(true); return; }
        const manifest = validateRigManifest(PLACEHOLDER_RIG_MANIFEST);
        await rig.load(manifest);
        element.append(app.canvas);
        app.stage.addChild(rig.view);
        rig.resize(element.clientWidth, element.clientHeight);
        rig.play("idle");
        app.ticker.add((ticker) => rig.update(ticker.deltaMS));
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
      if (app.renderer) {
        rig.destroy();
        app.destroy(true);
      }
    };
  }, []);

  return <div ref={host} className="rig-surface" data-rig-status={status.kind} aria-hidden="true">
    {status.kind === "unavailable" && <span>RIG UNAVAILABLE<br />{status.message}</span>}
  </div>;
}
