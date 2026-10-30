import assert from "node:assert/strict";
import { access, readFile } from "node:fs/promises";
import { test } from "node:test";
import { fileURLToPath } from "node:url";
import sharp from "sharp";

const root = new URL("../", import.meta.url);
const planUrl = new URL("art/character/saiki-kusuo/layer-plan.json", root);
const referenceUrl = new URL("art/character/saiki-kusuo/source/reference-clean-v1.png", root);
const previewUrl = new URL("apps/desktop/public/character/saiki-kusuo/reference/reference-clean-v1.webp", root);
const manifestUrl = new URL("apps/desktop/public/character/saiki-kusuo/rig.v2.json", root);

test("the source workflow has a complete non-production layer plan", async () => {
  const plan = JSON.parse(await readFile(planUrl, "utf8"));
  assert.equal(plan.status, "registration_estimates_require_artist_review");
  assert.equal(plan.layers.length, 52);
  assert.equal(new Set(plan.layers.map((layer) => layer.id)).size, plan.layers.length);
  assert.ok(plan.layers.every((layer) => layer.registration === "estimated_requires_artist_review"));
});

test("the admitted reference has the canonical transparent canvas", async () => {
  const metadata = await sharp(fileURLToPath(referenceUrl)).metadata();
  assert.equal(metadata.width, 2400); assert.equal(metadata.height, 4400); assert.equal(metadata.format, "png"); assert.equal(metadata.hasAlpha, true);
});

test("the visible preview is labelled separately from production rig output", async () => {
  const metadata = await sharp(fileURLToPath(previewUrl)).metadata(); assert.equal(metadata.format, "webp"); assert.equal(metadata.hasAlpha, true);
  await assert.rejects(access(fileURLToPath(manifestUrl)), /ENOENT/);
});
