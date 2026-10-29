import { describe, expect, test } from "vitest";
import { Texture } from "pixi.js";
import { CharacterStateMachine, FrameBudgetTracker, PlaceholderPixiRig, inspectRigManifestV2, loadLayeredRigV2, validateRigManifest, validateRigManifestV2, verifyAssetBytes } from "../src/index";

const manifest = {
  schemaVersion: 1,
  rigId: "saiki.placeholder",
  displayName: "SAIKI Placeholder",
  runtime: "pixi-placeholder",
  logicalSize: { width: 420, height: 720 },
  layers: [{ id: "body", role: "body", asset: "layers/body.png", zIndex: 0, anchor: [0.5, 1], required: true }],
  motions: [{ id: "idle", durationMs: 2400, loop: true }],
} as const;

const manifestV2 = {
  schemaVersion: 2, rigId: "saiki.production", displayName: "SAIKI production layers", runtime: "pixi-layered-v2",
  package: { version: "1.0.0", author: "Owner supplied", license: "Owner documented", source: "Commissioned layered artwork" }, logicalSize: { width: 1200, height: 2200 },
  assets: [{ id: "torso", path: "layers/body/torso.png", mediaType: "image/png", sha256: `sha256:${"a".repeat(64)}`, width: 1, height: 1, scale: 2 }],
  layers: [{ id: "torso", role: "body", semantic: "body.torso", assetId: "torso", parentId: null, zIndex: 0, pivot: [0.5, 0.5], position: [600, 1100], modes: ["full_body", "floating"], required: true }],
  profiles: [{ mode: "full_body", scale: 1, anchor: [0.5, 1], offset: [0, 0] }, { mode: "floating", scale: 1.3, anchor: [0.5, 0.45], offset: [0, 40] }],
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

describe("production manifest v2", () => {
  test("requires provenance, hashes, hierarchy, and both presentation profiles", () => expect(validateRigManifestV2(manifestV2).runtime).toBe("pixi-layered-v2"));
  test("rejects unsafe paths and missing parent layers", () => {
    expect(inspectRigManifestV2({ ...manifestV2, assets: [{ ...manifestV2.assets[0], path: "../torso.png" }] }).valid).toBe(false);
    expect(inspectRigManifestV2({ ...manifestV2, layers: [{ ...manifestV2.layers[0], parentId: "missing" }] }).issues.some((issue) => issue.code === "layer.parent_missing")).toBe(true);
  });
  test("rejects parent cycles", () => {
    const second = { ...manifestV2.layers[0], id: "head", semantic: "face.head", parentId: "torso" };
    const first = { ...manifestV2.layers[0], parentId: "head" };
    expect(inspectRigManifestV2({ ...manifestV2, layers: [first, second] }).issues.some((issue) => issue.code === "layer.parent_cycle")).toBe(true);
  });
  test("loads real texture sources and switches presentation mode", async () => {
    const rig = await loadLayeredRigV2(manifestV2, "https://saiki.local/character/", async () => Texture.EMPTY);
    expect(rig.layers.size).toBe(1); rig.setMode("floating"); expect(rig.root.scale.x).toBeCloseTo(1.3); expect(rig.layers.get("torso")?.node.visible).toBe(true); rig.destroy();
  });
  test("rejects decoded dimensions that do not match the manifest", async () => {
    await expect(loadLayeredRigV2({ ...manifestV2, assets: [{ ...manifestV2.assets[0], width: 2 }] }, "https://saiki.local/", async () => Texture.EMPTY)).rejects.toThrow(/dimensions/i);
  });
  test("verifies the exact asset bytes against the manifest digest", async () => {
    const bytes = new TextEncoder().encode("saiki-layer").buffer;
    const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", bytes));
    const sha256 = `sha256:${[...digest].map((value) => value.toString(16).padStart(2, "0")).join("")}` as `sha256:${string}`;
    await expect(verifyAssetBytes({ ...manifestV2.assets[0], sha256 }, bytes)).resolves.toBeUndefined();
    await expect(verifyAssetBytes(manifestV2.assets[0], bytes)).rejects.toThrow(/SHA-256/i);
  });
});
