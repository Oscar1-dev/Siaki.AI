import { createHash } from "node:crypto";
import { mkdir, readFile, readdir, stat, writeFile } from "node:fs/promises";
import path from "node:path";
import process from "node:process";
import sharp from "sharp";

const ROOT = process.cwd();
const ART_ROOT = path.join(ROOT, "art", "character", "saiki-kusuo");
const SOURCE_ROOT = path.join(ART_ROOT, "source");
const MASK_ROOT = path.join(ART_ROOT, "masks");
const LAYER_ROOT = path.join(ART_ROOT, "layers");
const PUBLIC_ROOT = path.join(ROOT, "apps", "desktop", "public", "character", "saiki-kusuo");
const PLAN_PATH = path.join(ART_ROOT, "layer-plan.json");
const CANVAS = { width: 2400, height: 4400 };
const COMMAND = process.argv[2];
const args = parseArgs(process.argv.slice(3));

const LAYERS = [
  entry("back/back-hair", "hair.back", "hair", null, -30, [1200, 650]),
  entry("body/torso", "body.torso", "body", null, 0, [1200, 1850]),
  entry("body/hips", "body.hips", "body", "body/torso", 2, [1200, 2500]),
  entry("body/upper-leg-left", "body.leg.left.upper", "body", "body/hips", 1, [1020, 2660]),
  entry("body/lower-leg-left", "body.leg.left.lower", "body", "body/upper-leg-left", 1, [1000, 3350]),
  entry("body/foot-left", "body.foot.left", "body", "body/lower-leg-left", 1, [990, 4050]),
  entry("body/upper-leg-right", "body.leg.right.upper", "body", "body/hips", 1, [1380, 2660]),
  entry("body/lower-leg-right", "body.leg.right.lower", "body", "body/upper-leg-right", 1, [1400, 3350]),
  entry("body/foot-right", "body.foot.right", "body", "body/lower-leg-right", 1, [1410, 4050]),
  entry("body/upper-arm-left", "body.arm.left.upper", "body", "body/torso", 3, [780, 1500]),
  entry("body/forearm-left", "body.arm.left.lower", "body", "body/upper-arm-left", 3, [670, 2150]),
  entry("body/hand-left-relaxed", "body.hand.left.relaxed", "body", "body/forearm-left", 4, [620, 2780]),
  entry("body/hand-left-psychic", "body.hand.left.psychic", "body", "body/forearm-left", 4, [620, 2780], false),
  entry("body/upper-arm-right", "body.arm.right.upper", "body", "body/torso", 3, [1620, 1500]),
  entry("body/forearm-right", "body.arm.right.lower", "body", "body/upper-arm-right", 3, [1730, 2150]),
  entry("body/hand-right-relaxed", "body.hand.right.relaxed", "body", "body/forearm-right", 4, [1780, 2780]),
  entry("body/hand-right-psychic", "body.hand.right.psychic", "body", "body/forearm-right", 4, [1780, 2780], false),
  entry("clothing/jacket-body", "clothing.jacket.body", "clothing", "body/torso", 10, [1200, 1850]),
  entry("clothing/jacket-sleeve-left", "clothing.jacket.sleeve.left", "clothing", "body/upper-arm-left", 10, [780, 1500]),
  entry("clothing/jacket-sleeve-right", "clothing.jacket.sleeve.right", "clothing", "body/upper-arm-right", 10, [1620, 1500]),
  entry("clothing/trousers", "clothing.trousers", "clothing", "body/hips", 10, [1200, 2500]),
  entry("clothing/shoes", "clothing.shoes", "clothing", "body/hips", 10, [1200, 4050]),
  entry("head/neck", "head.neck", "body", "body/torso", 15, [1200, 1280]),
  entry("head/head-base", "head.base", "face", "head/neck", 16, [1200, 1040]),
  entry("head/ear-left", "head.ear.left", "face", "head/head-base", 17, [940, 1010]),
  entry("head/ear-right", "head.ear.right", "face", "head/head-base", 17, [1460, 1010]),
  entry("head/front-hair", "hair.front", "hair", "head/head-base", 30, [1200, 720]),
  entry("head/side-hair-left", "hair.side.left", "hair", "head/head-base", 29, [930, 850]),
  entry("head/side-hair-right", "hair.side.right", "hair", "head/head-base", 29, [1470, 850]),
  entry("face/eye-white-left", "face.eye.left.white", "face", "head/head-base", 20, [1090, 1010]),
  entry("face/eye-white-right", "face.eye.right.white", "face", "head/head-base", 20, [1310, 1010]),
  entry("face/iris-left", "face.eye.left.iris", "face", "face/eye-white-left", 21, [1090, 1010]),
  entry("face/iris-right", "face.eye.right.iris", "face", "face/eye-white-right", 21, [1310, 1010]),
  entry("face/eyelid-left-open", "face.eyelid.left.open", "face", "head/head-base", 22, [1090, 1010]),
  entry("face/eyelid-left-closed", "face.eyelid.left.closed", "face", "head/head-base", 22, [1090, 1010], false),
  entry("face/eyelid-right-open", "face.eyelid.right.open", "face", "head/head-base", 22, [1310, 1010]),
  entry("face/eyelid-right-closed", "face.eyelid.right.closed", "face", "head/head-base", 22, [1310, 1010], false),
  entry("face/eyebrow-left", "face.eyebrow.left", "face", "head/head-base", 23, [1090, 920]),
  entry("face/eyebrow-right", "face.eyebrow.right", "face", "head/head-base", 23, [1310, 920]),
  ...["neutral", "a", "e", "i", "o", "u", "mbp"].map((shape, index) => entry(`face/mouth-${shape}`, `face.mouth.${shape}`, "face", "head/head-base", 24, [1200, 1160], index === 0)),
  entry("accessories/glasses-frame", "accessory.glasses.frame", "accessory", "head/head-base", 40, [1200, 1010]),
  entry("accessories/glasses-lenses", "accessory.glasses.lenses", "accessory", "head/head-base", 39, [1200, 1010]),
  entry("accessories/limiter-left", "accessory.limiter.left", "accessory", "head/head-base", 42, [930, 650]),
  entry("accessories/limiter-right", "accessory.limiter.right", "accessory", "head/head-base", 42, [1470, 650]),
  entry("effects/aura-mask", "effect.aura.mask", "effect", null, -50, [1200, 2200], false),
  entry("effects/teleport-mask", "effect.teleport.mask", "effect", null, 50, [1200, 2200], false),
];

try {
  if (COMMAND === "inspect") await inspect(requiredArg("input"));
  else if (COMMAND === "prepare-reference") await prepareReference(requiredArg("input"), args.output ?? path.join(SOURCE_ROOT, "reference-clean-v1.png"), Number(args["alpha-cutoff"] ?? 32));
  else if (COMMAND === "scaffold") await scaffold();
  else if (COMMAND === "preview") await publishReferencePreview(args.input ?? path.join(SOURCE_ROOT, "reference-clean-v1.png"));
  else if (COMMAND === "extract") await extractLayers(args.reference ?? path.join(SOURCE_ROOT, "reference-clean-v1.png"));
  else if (COMMAND === "validate") await validateLayers();
  else if (COMMAND === "build") await buildPackage();
  else throw new Error("Usage: character-assets <inspect|prepare-reference|scaffold|preview|extract|validate|build> [--input path] [--output path] [--alpha-cutoff 32]");
} catch (error) { console.error(error instanceof Error ? error.message : String(error)); process.exitCode = 1; }

function entry(file, semantic, role, parent, zIndex, pivot, defaultVisible = true) { return { id: file.replaceAll("/", "."), file: `${file}.png`, mask: `${file}.mask.png`, semantic, role, parent: parent?.replaceAll("/", ".") ?? null, zIndex, pivot, modes: ["full_body", "floating"], required: true, defaultVisible, registration: "estimated_requires_artist_review" }; }
function parseArgs(values) { const result = {}; for (let index = 0; index < values.length; index += 2) { const key = values[index]; if (!key?.startsWith("--")) throw new Error(`Unexpected argument ${key}.`); result[key.slice(2)] = values[index + 1]; } return result; }
function requiredArg(name) { const value = args[name]; if (!value) throw new Error(`Missing --${name}.`); return path.resolve(ROOT, value); }
function sha256(bytes) { return `sha256:${createHash("sha256").update(bytes).digest("hex")}`; }
async function exists(target) { try { await stat(target); return true; } catch { return false; } }

async function inspect(input) {
  const bytes = await readFile(input); const image = sharp(bytes, { limitInputPixels: 8192 * 8192 }); const metadata = await image.metadata();
  const stats = await image.ensureAlpha().stats(); const alpha = stats.channels[3];
  console.log(JSON.stringify({ path: input, bytes: bytes.length, sha256: sha256(bytes), width: metadata.width, height: metadata.height, format: metadata.format, colourSpace: metadata.space, hasAlpha: metadata.hasAlpha, alpha: alpha ? { min: alpha.min, max: alpha.max, mean: alpha.mean } : null }, null, 2));
}

async function prepareReference(input, output, cutoff) {
  if (!Number.isInteger(cutoff) || cutoff < 0 || cutoff > 254) throw new Error("Alpha cutoff must be an integer from 0 to 254.");
  const { data, info } = await sharp(input, { limitInputPixels: 8192 * 8192 }).ensureAlpha().resize(CANVAS.width, CANVAS.height, { fit: "contain", background: { r: 0, g: 0, b: 0, alpha: 0 }, kernel: "lanczos3" }).raw().toBuffer({ resolveWithObject: true });
  let removed = 0; for (let offset = 3; offset < data.length; offset += 4) { const alpha = data[offset]; if (alpha <= cutoff) { if (alpha > 0) removed += 1; data[offset] = 0; } else data[offset] = Math.round(((alpha - cutoff) / (255 - cutoff)) * 255); }
  await mkdir(path.dirname(output), { recursive: true }); await sharp(data, { raw: info }).png({ compressionLevel: 9, adaptiveFiltering: true }).withMetadata({ density: 144 }).toFile(output);
  console.log(JSON.stringify({ output, canvas: CANVAS, alphaCutoff: cutoff, removedLowAlphaPixels: removed }, null, 2));
}

async function scaffold() {
  await Promise.all([SOURCE_ROOT, MASK_ROOT, LAYER_ROOT].map((target) => mkdir(target, { recursive: true })));
  for (const layer of LAYERS) { await mkdir(path.dirname(path.join(MASK_ROOT, layer.mask)), { recursive: true }); await mkdir(path.dirname(path.join(LAYER_ROOT, layer.file)), { recursive: true }); }
  const plan = { schemaVersion: 1, canvas: CANVAS, status: "registration_estimates_require_artist_review", layers: LAYERS };
  await writeFile(PLAN_PATH, `${JSON.stringify(plan, null, 2)}\n`, { flag: "wx" }).catch(async (error) => { if (error?.code !== "EEXIST") throw error; });
  console.log(JSON.stringify({ plan: PLAN_PATH, masks: MASK_ROOT, layers: LAYER_ROOT, layerCount: LAYERS.length }, null, 2));
}

async function publishReferencePreview(input) {
  const destination = path.join(PUBLIC_ROOT, "reference", "reference-clean-v1.webp");
  await mkdir(path.dirname(destination), { recursive: true });
  const info = await sharp(input).resize({ width: 900, height: 1650, fit: "inside", withoutEnlargement: true }).webp({ quality: 92, alphaQuality: 100, effort: 6 }).toFile(destination);
  console.log(JSON.stringify({ status: "reference_only_not_a_rig", destination, width: info.width, height: info.height, bytes: info.size }, null, 2));
}

async function extractLayers(reference) {
  const referenceImage = sharp(reference).ensureAlpha(); const referenceMetadata = await referenceImage.metadata();
  if (referenceMetadata.width !== CANVAS.width || referenceMetadata.height !== CANVAS.height) throw new Error("Clean reference must match the 2400x4400 canvas.");
  const referenceRaw = await referenceImage.raw().toBuffer(); let extracted = 0; const missingMasks = [];
  for (const layer of LAYERS) {
    const maskPath = path.join(MASK_ROOT, layer.mask); if (!await exists(maskPath)) { missingMasks.push(layer.mask); continue; }
    const mask = await sharp(maskPath).ensureAlpha().resize(CANVAS.width, CANVAS.height, { fit: "fill" }).raw().toBuffer(); const output = Buffer.from(referenceRaw);
    for (let offset = 0; offset < output.length; offset += 4) output[offset + 3] = Math.round((output[offset + 3] * mask[offset + 3]) / 255);
    const destination = path.join(LAYER_ROOT, layer.file); await mkdir(path.dirname(destination), { recursive: true }); await sharp(output, { raw: { width: CANVAS.width, height: CANVAS.height, channels: 4 } }).png({ compressionLevel: 9 }).toFile(destination); extracted += 1;
  }
  console.log(JSON.stringify({ extracted, missingMasks }, null, 2)); if (missingMasks.length) process.exitCode = 2;
}

async function validateLayers() {
  const missing = [], invalid = [];
  for (const layer of LAYERS) {
    const target = path.join(LAYER_ROOT, layer.file); if (!await exists(target)) { missing.push(layer.file); continue; }
    const metadata = await sharp(target, { limitInputPixels: 8192 * 8192 }).metadata();
    if (metadata.width !== CANVAS.width || metadata.height !== CANVAS.height || !metadata.hasAlpha || metadata.format !== "png") invalid.push({ file: layer.file, width: metadata.width, height: metadata.height, hasAlpha: metadata.hasAlpha, format: metadata.format });
  }
  const result = { valid: missing.length === 0 && invalid.length === 0, expected: LAYERS.length, missing, invalid };
  console.log(JSON.stringify(result, null, 2)); if (!result.valid) throw new Error(`Layer validation failed: ${missing.length} missing, ${invalid.length} invalid.`); return result;
}

async function buildPackage() {
  await validateLayers(); await mkdir(PUBLIC_ROOT, { recursive: true });
  const assets = [], layers = [], packed = [];
  const pivotById = new Map(LAYERS.map((layer) => [layer.id, layer.pivot]));
  for (const layer of LAYERS) {
    const source = path.join(LAYER_ROOT, layer.file); const trimmed = await sharp(source).trim({ background: { r: 0, g: 0, b: 0, alpha: 0 } }).png().toBuffer({ resolveWithObject: true });
    if (!trimmed.info.width || !trimmed.info.height) throw new Error(`Layer ${layer.file} is fully transparent.`);
    const left = -(trimmed.info.trimOffsetLeft ?? 0), top = -(trimmed.info.trimOffsetTop ?? 0); const pivot = [(layer.pivot[0] - left) / trimmed.info.width, (layer.pivot[1] - top) / trimmed.info.height];
    if (pivot.some((value) => value < 0 || value > 1)) throw new Error(`Pivot for ${layer.file} falls outside its visible pixels; review layer-plan.json.`);
    const relative = layer.parent ? [layer.pivot[0] - pivotById.get(layer.parent)[0], layer.pivot[1] - pivotById.get(layer.parent)[1]] : layer.pivot;
    const relativePath = `layers/${layer.file}`; const destination = path.join(PUBLIC_ROOT, relativePath); await mkdir(path.dirname(destination), { recursive: true }); await writeFile(destination, trimmed.data);
    assets.push({ id: layer.id, path: relativePath.replaceAll("\\", "/"), mediaType: "image/png", sha256: sha256(trimmed.data), width: trimmed.info.width, height: trimmed.info.height, scale: 2 });
    layers.push({ id: layer.id, role: layer.role, semantic: layer.semantic, assetId: layer.id, parentId: layer.parent, zIndex: layer.zIndex, pivot, position: relative, modes: layer.modes, required: layer.required });
    packed.push({ id: layer.id, data: trimmed.data, width: trimmed.info.width, height: trimmed.info.height });
  }
  const atlas = await buildAtlas(packed); await writeFile(path.join(PUBLIC_ROOT, "character.atlas.png"), atlas.png); await writeFile(path.join(PUBLIC_ROOT, "character.atlas.json"), `${JSON.stringify(atlas.map, null, 2)}\n`);
  const manifest = { schemaVersion: 2, rigId: "saiki.kusuo.production", displayName: "Saiki Kusuo layered production rig", runtime: "pixi-layered-v2", package: { version: "1.0.0", author: "AI reference plus owner-reviewed layer production", license: "Rights review required before distribution", source: "art/character/saiki-kusuo" }, logicalSize: CANVAS, assets, layers, profiles: [{ mode: "full_body", scale: 0.25, anchor: [0.5, 1], offset: [0, 0] }, { mode: "floating", scale: 0.38, anchor: [0.5, 0.45], offset: [0, 40] }] };
  await writeFile(path.join(PUBLIC_ROOT, "rig.v2.json"), `${JSON.stringify(manifest, null, 2)}\n`); console.log(JSON.stringify({ output: PUBLIC_ROOT, assets: assets.length, atlas: atlas.map.meta }, null, 2));
}

async function buildAtlas(images) {
  const width = 4096, padding = 2; let x = padding, y = padding, rowHeight = 0; const composites = [], frames = {};
  for (const image of [...images].sort((a, b) => b.height - a.height)) { if (image.width + padding * 2 > width) throw new Error(`Layer ${image.id} is wider than the atlas.`); if (x + image.width + padding > width) { x = padding; y += rowHeight + padding; rowHeight = 0; } frames[image.id] = { x, y, width: image.width, height: image.height }; composites.push({ input: image.data, left: x, top: y }); x += image.width + padding; rowHeight = Math.max(rowHeight, image.height); }
  const height = nextPowerOfTwo(y + rowHeight + padding); if (height > 8192) throw new Error("Layer atlas exceeds 4096x8192; split the package or reduce source dimensions.");
  const png = await sharp({ create: { width, height, channels: 4, background: { r: 0, g: 0, b: 0, alpha: 0 } } }).composite(composites).png({ compressionLevel: 9 }).toBuffer();
  return { png, map: { meta: { width, height, format: "png", sha256: sha256(png) }, frames } };
}
function nextPowerOfTwo(value) { let result = 1; while (result < value) result *= 2; return result; }
