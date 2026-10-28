import { describe, expect, test } from "vitest";
import { CharacterStateMachine, FrameBudgetTracker, PlaceholderPixiRig, validateRigManifest } from "../src/index";

const manifest = {
  schemaVersion: 1,
  rigId: "saiki.placeholder",
  displayName: "SAIKI Placeholder",
  runtime: "pixi-placeholder",
  logicalSize: { width: 420, height: 720 },
  layers: [{ id: "body", role: "body", asset: "layers/body.png", zIndex: 0, anchor: [0.5, 1], required: true }],
  motions: [{ id: "idle", durationMs: 2400, loop: true }],
} as const;

describe("rig manifest", () => {
  test("accepts a versioned layered manifest", () => expect(validateRigManifest(manifest).rigId).toBe("saiki.placeholder"));
  test("rejects traversal in asset paths", () => expect(() => validateRigManifest({ ...manifest, layers: [{ ...manifest.layers[0], asset: "../secret" }] })).toThrow(/unsafe asset path/i));
  test("rejects duplicate layer ids", () => expect(() => validateRigManifest({ ...manifest, layers: [manifest.layers[0], manifest.layers[0]] })).toThrow(/duplicate/i));
  test("exposes a replaceable runtime contract", () => expect(new PlaceholderPixiRig().runtimeId).toBe(manifest.runtime));
  test("animates relative to its resized origin", async () => {
    const rig = new PlaceholderPixiRig();
    await rig.load(validateRigManifest(manifest));
    rig.resize(800, 600);
    rig.transition({ state: "idle" });
    rig.update(16);
    expect(rig.view.x).toBe(400);
    expect(rig.view.y).toBeGreaterThan(300);
    expect(rig.view.y).toBeLessThan(301);
    rig.destroy();
  });
  test("every required motion produces a distinct pose", () => {
    const machine = new CharacterStateMachine();
    expect(machine.update(16).state).toBe("idle");
    machine.request({ state: "gaze", gaze: { x: 2, y: -2 } });
    expect(machine.update(16)).toMatchObject({ state: "gaze", gazeX: 1, gazeY: -1 });
    machine.request({ state: "blink" });
    expect(machine.update(90).blink).toBeGreaterThan(0.9);
    machine.request({ state: "walk", direction: -1 });
    expect(machine.update(400).offsetX).toBeGreaterThan(0);
    machine.request({ state: "sit" });
    expect(machine.update(16).bodyScaleY).toBeLessThan(1);
    machine.request({ state: "teleport" });
    expect(machine.update(50).effectIntensity).toBeGreaterThan(0.7);
  });
  test("a newer transition immediately interrupts the current state", () => {
    const machine = new CharacterStateMachine();
    const walkId = machine.request({ state: "walk" });
    machine.update(300);
    const teleportId = machine.request({ state: "teleport" });
    expect(teleportId).toBeGreaterThan(walkId);
    expect(machine.snapshot()).toMatchObject({ state: "teleport", elapsedMs: 0, transitionId: teleportId });
  });
  test("timed states return to idle while sit remains stable", () => {
    const machine = new CharacterStateMachine();
    machine.request({ state: "blink" });
    machine.update(100);
    expect(machine.update(100).state).toBe("idle");
    machine.request({ state: "sit" });
    expect(machine.update(2_000).state).toBe("sit");
  });
  test("frame telemetry reports deterministic percentiles and overruns", () => {
    const tracker = new FrameBudgetTracker(16.67, 5);
    [[0.1, 16], [0.2, 17], [0.3, 15], [0.4, 20], [0.5, 16]].forEach(([update, frame]) => tracker.record(update!, frame!));
    expect(tracker.snapshot()).toEqual({ sampleCount: 5, updateMedianMs: 0.3, updateP95Ms: 0.5, frameP95Ms: 20, framesOverBudget: 2, budgetMs: 16.67 });
  });
});
