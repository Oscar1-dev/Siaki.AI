export const RIG_MANIFEST_VERSION_V2 = 2 as const;

export type AssetMediaType = "image/png" | "image/webp";
export type PresentationMode = "full_body" | "floating";
export type LayerRoleV2 = "body" | "face" | "hair" | "clothing" | "accessory" | "effect";

export interface RigAssetV2 { readonly id: string; readonly path: string; readonly mediaType: AssetMediaType; readonly sha256: `sha256:${string}`; readonly width: number; readonly height: number; readonly scale: 1 | 2; }
export interface RigLayerV2 { readonly id: string; readonly role: LayerRoleV2; readonly semantic: string; readonly assetId: string; readonly parentId: string | null; readonly zIndex: number; readonly pivot: readonly [number, number]; readonly position: readonly [number, number]; readonly modes: readonly PresentationMode[]; readonly required: boolean; }
export interface PresentationProfileV2 { readonly mode: PresentationMode; readonly scale: number; readonly anchor: readonly [number, number]; readonly offset: readonly [number, number]; }
export interface RigManifestV2 {
  readonly schemaVersion: typeof RIG_MANIFEST_VERSION_V2;
  readonly rigId: string;
  readonly displayName: string;
  readonly runtime: "pixi-layered-v2";
  readonly package: { readonly version: string; readonly author: string; readonly license: string; readonly source: string };
  readonly logicalSize: { readonly width: number; readonly height: number };
  readonly assets: readonly RigAssetV2[];
  readonly layers: readonly RigLayerV2[];
  readonly profiles: readonly PresentationProfileV2[];
}
export interface ManifestValidationIssue { readonly code: string; readonly path: string; readonly message: string; }
export interface ManifestValidationResult { readonly valid: boolean; readonly issues: readonly ManifestValidationIssue[]; readonly manifest?: RigManifestV2; }

const ID = /^[a-z0-9][a-z0-9._-]{0,63}$/;
const SEMANTIC = /^[a-z][a-z0-9._-]{0,95}$/;
const DIGEST = /^sha256:[a-f0-9]{64}$/;
const VERSION = /^\d+\.\d+\.\d+$/;
const ROLES = new Set<LayerRoleV2>(["body", "face", "hair", "clothing", "accessory", "effect"]);
const MEDIA = new Set<AssetMediaType>(["image/png", "image/webp"]);
const MODES = new Set<PresentationMode>(["full_body", "floating"]);

export function validateRigManifestV2(value: unknown): RigManifestV2 {
  const result = inspectRigManifestV2(value);
  if (!result.valid || !result.manifest) throw new Error(`Invalid rig manifest v2: ${result.issues.map((issue) => `${issue.path}: ${issue.message}`).join("; ")}`);
  return result.manifest;
}

export function inspectRigManifestV2(value: unknown): ManifestValidationResult {
  const issues: ManifestValidationIssue[] = [];
  const add = (code: string, path: string, message: string) => issues.push({ code, path, message });
  if (!record(value)) return { valid: false, issues: [{ code: "manifest.type", path: "$", message: "Manifest must be an object." }] };
  if (value.schemaVersion !== RIG_MANIFEST_VERSION_V2) add("manifest.version", "schemaVersion", "Expected schema version 2.");
  if (!id(value.rigId)) add("manifest.rig_id", "rigId", "Expected a stable lowercase identifier.");
  if (!nonEmpty(value.displayName, 128)) add("manifest.display_name", "displayName", "Expected a non-empty name up to 128 characters.");
  if (value.runtime !== "pixi-layered-v2") add("manifest.runtime", "runtime", "Expected pixi-layered-v2.");
  if (!record(value.package)) add("package.type", "package", "Package provenance is required.");
  else {
    if (typeof value.package.version !== "string" || !VERSION.test(value.package.version)) add("package.version", "package.version", "Expected semantic version x.y.z.");
    for (const key of ["author", "license", "source"] as const) if (!nonEmpty(value.package[key], 512)) add(`package.${key}`, `package.${key}`, "A real provenance value is required.");
  }
  if (!record(value.logicalSize) || !positiveInt(value.logicalSize.width, 8192) || !positiveInt(value.logicalSize.height, 8192)) add("logical_size", "logicalSize", "Width and height must be integers from 1 to 8192.");

  const assetIds = new Set<string>();
  if (!Array.isArray(value.assets) || value.assets.length === 0 || value.assets.length > 256) add("assets.count", "assets", "Provide 1 to 256 assets.");
  else value.assets.forEach((asset, index) => {
    const path = `assets[${index}]`;
    if (!record(asset)) { add("asset.type", path, "Asset must be an object."); return; }
    if (!id(asset.id) || assetIds.has(asset.id as string)) add("asset.id", `${path}.id`, "Asset id is invalid or duplicated."); else assetIds.add(asset.id as string);
    if (typeof asset.path !== "string" || !safeAssetPath(asset.path)) add("asset.path", `${path}.path`, "Use a safe relative PNG or WebP path.");
    if (!MEDIA.has(asset.mediaType as AssetMediaType)) add("asset.media_type", `${path}.mediaType`, "Expected image/png or image/webp.");
    if (typeof asset.sha256 !== "string" || !DIGEST.test(asset.sha256)) add("asset.digest", `${path}.sha256`, "Expected a lowercase SHA-256 digest.");
    if (!positiveInt(asset.width, 8192) || !positiveInt(asset.height, 8192)) add("asset.dimensions", path, "Asset dimensions must be integers from 1 to 8192.");
    if (asset.scale !== 1 && asset.scale !== 2) add("asset.scale", `${path}.scale`, "Scale must be 1 or 2.");
  });

  const layers = new Map<string, Record<string, unknown>>();
  if (!Array.isArray(value.layers) || value.layers.length === 0 || value.layers.length > 256) add("layers.count", "layers", "Provide 1 to 256 layers.");
  else value.layers.forEach((layer, index) => {
    const path = `layers[${index}]`;
    if (!record(layer)) { add("layer.type", path, "Layer must be an object."); return; }
    if (!id(layer.id) || layers.has(layer.id as string)) add("layer.id", `${path}.id`, "Layer id is invalid or duplicated."); else layers.set(layer.id as string, layer);
    if (!ROLES.has(layer.role as LayerRoleV2)) add("layer.role", `${path}.role`, "Unknown layer role.");
    if (typeof layer.semantic !== "string" || !SEMANTIC.test(layer.semantic)) add("layer.semantic", `${path}.semantic`, "Expected a stable semantic slot.");
    if (typeof layer.assetId !== "string" || !assetIds.has(layer.assetId)) add("layer.asset", `${path}.assetId`, "Layer references an unknown asset.");
    if (layer.parentId !== null && !id(layer.parentId)) add("layer.parent", `${path}.parentId`, "Parent must be null or a valid layer id.");
    if (!Number.isInteger(layer.zIndex) || (layer.zIndex as number) < -10_000 || (layer.zIndex as number) > 10_000) add("layer.z_index", `${path}.zIndex`, "Z index is out of bounds.");
    if (!point(layer.pivot, true)) add("layer.pivot", `${path}.pivot`, "Pivot values must be in the 0..1 interval.");
    if (!point(layer.position, false)) add("layer.position", `${path}.position`, "Position must contain two finite values.");
    if (!Array.isArray(layer.modes) || layer.modes.length === 0 || layer.modes.some((mode) => !MODES.has(mode as PresentationMode))) add("layer.modes", `${path}.modes`, "At least one valid presentation mode is required.");
    if (typeof layer.required !== "boolean") add("layer.required", `${path}.required`, "Required must be boolean.");
  });
  for (const [layerId, layer] of layers) {
    if (typeof layer.parentId === "string" && !layers.has(layer.parentId)) add("layer.parent_missing", `layers.${layerId}.parentId`, "Parent layer does not exist.");
    if (hasParentCycle(layerId, layers)) add("layer.parent_cycle", `layers.${layerId}.parentId`, "Layer hierarchy contains a cycle.");
  }

  const profiles = new Set<PresentationMode>();
  if (!Array.isArray(value.profiles)) add("profiles.type", "profiles", "Presentation profiles are required.");
  else value.profiles.forEach((profile, index) => {
    const path = `profiles[${index}]`;
    if (!record(profile) || !MODES.has(profile.mode as PresentationMode) || profiles.has(profile.mode as PresentationMode)) { add("profile.mode", `${path}.mode`, "Profile mode is invalid or duplicated."); return; }
    profiles.add(profile.mode as PresentationMode);
    if (typeof profile.scale !== "number" || !Number.isFinite(profile.scale) || profile.scale <= 0 || profile.scale > 4) add("profile.scale", `${path}.scale`, "Scale must be greater than zero and no more than four.");
    if (!point(profile.anchor, true)) add("profile.anchor", `${path}.anchor`, "Anchor values must be in the 0..1 interval.");
    if (!point(profile.offset, false)) add("profile.offset", `${path}.offset`, "Offset must contain two finite values.");
  });
  for (const mode of MODES) if (!profiles.has(mode)) add("profile.missing", "profiles", `Missing ${mode} profile.`);
  return issues.length === 0 ? { valid: true, issues, manifest: value as unknown as RigManifestV2 } : { valid: false, issues };
}

function record(value: unknown): value is Record<string, unknown> { return !!value && typeof value === "object" && !Array.isArray(value); }
function id(value: unknown): value is string { return typeof value === "string" && ID.test(value); }
function nonEmpty(value: unknown, max: number): value is string { return typeof value === "string" && value.trim().length > 0 && value.length <= max; }
function positiveInt(value: unknown, max: number): value is number { return Number.isInteger(value) && (value as number) > 0 && (value as number) <= max; }
function point(value: unknown, unit: boolean): value is readonly [number, number] { return Array.isArray(value) && value.length === 2 && value.every((item) => typeof item === "number" && Number.isFinite(item) && (!unit || (item >= 0 && item <= 1))); }
function safeAssetPath(value: string): boolean { return value.length <= 260 && !value.includes("\\") && !value.startsWith("/") && !value.includes(":") && !value.split("/").includes("..") && /\.(png|webp)$/i.test(value); }
function hasParentCycle(start: string, layers: ReadonlyMap<string, Record<string, unknown>>): boolean { const seen = new Set<string>(); let current: string | null = start; while (current) { if (seen.has(current)) return true; seen.add(current); const layer = layers.get(current); current = typeof layer?.parentId === "string" ? layer.parentId : null; } return false; }
