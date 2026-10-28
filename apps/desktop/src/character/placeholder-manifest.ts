import type { RigManifest } from "@saiki/character-rig";

export const PLACEHOLDER_RIG_MANIFEST = {
  schemaVersion: 1,
  rigId: "saiki.placeholder",
  displayName: "SAIKI Procedural Placeholder",
  runtime: "pixi-placeholder",
  logicalSize: { width: 420, height: 720 },
  layers: [
    { id: "aura", role: "effect", asset: "procedural/aura", zIndex: 0, anchor: [0.5, 0.5], required: true },
    { id: "body", role: "body", asset: "procedural/body", zIndex: 10, anchor: [0.5, 0.5], required: true },
  ],
  motions: [
    { id: "idle", durationMs: 2400, loop: true },
    { id: "gaze", durationMs: 900, loop: false },
    { id: "blink", durationMs: 180, loop: false },
    { id: "walk", durationMs: 1600, loop: false },
    { id: "sit", durationMs: 1200, loop: true },
    { id: "teleport", durationMs: 720, loop: false },
  ],
} as const satisfies RigManifest;
