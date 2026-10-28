import { describe, expect, test } from "vitest";
import { PlaceholderPixiRig, validateRigManifest } from "../src/index";

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
    rig.play("idle");
    rig.update(16);
    expect(rig.view.x).toBe(400);
    expect(rig.view.y).toBeGreaterThan(300);
    expect(rig.view.y).toBeLessThan(301);
    rig.destroy();
  });
});
