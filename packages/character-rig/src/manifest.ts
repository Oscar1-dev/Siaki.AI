export const RIG_MANIFEST_VERSION = 1 as const;

export type RigLayerRole = "body" | "face" | "hair" | "clothing" | "accessory" | "effect";

export interface RigLayer {
  readonly id: string;
  readonly role: RigLayerRole;
  readonly asset: string;
  readonly zIndex: number;
  readonly anchor: readonly [number, number];
  readonly required: boolean;
}

export interface RigMotion {
  readonly id: string;
  readonly durationMs: number;
  readonly loop: boolean;
}

export interface RigManifest {
  readonly schemaVersion: typeof RIG_MANIFEST_VERSION;
  readonly rigId: string;
  readonly displayName: string;
  readonly runtime: string;
  readonly logicalSize: { readonly width: number; readonly height: number };
  readonly layers: readonly RigLayer[];
  readonly motions: readonly RigMotion[];
}

const ID = /^[a-z0-9][a-z0-9._-]{0,63}$/;
const ROLES = new Set<RigLayerRole>(["body", "face", "hair", "clothing", "accessory", "effect"]);

export function validateRigManifest(value: unknown): RigManifest {
  if (!value || typeof value !== "object") throw new Error("Rig manifest must be an object.");
  const manifest = value as Record<string, unknown>;
  if (manifest.schemaVersion !== RIG_MANIFEST_VERSION) throw new Error("Unsupported rig manifest version.");
  if (typeof manifest.rigId !== "string" || !ID.test(manifest.rigId)) throw new Error("Invalid rig id.");
  if (typeof manifest.displayName !== "string" || !manifest.displayName.trim()) throw new Error("Invalid rig display name.");
  if (typeof manifest.runtime !== "string" || !ID.test(manifest.runtime)) throw new Error("Invalid rig runtime id.");
  const size = manifest.logicalSize as Record<string, unknown> | undefined;
  if (!size || !positive(size.width) || !positive(size.height)) throw new Error("Invalid logical size.");
  if (!Array.isArray(manifest.layers) || manifest.layers.length === 0) throw new Error("At least one rig layer is required.");
  const layerIds = new Set<string>();
  for (const raw of manifest.layers) {
    if (!raw || typeof raw !== "object") throw new Error("Invalid rig layer.");
    const layer = raw as Record<string, unknown>;
    if (typeof layer.id !== "string" || !ID.test(layer.id) || layerIds.has(layer.id)) throw new Error("Invalid or duplicate layer id.");
    layerIds.add(layer.id);
    if (!ROLES.has(layer.role as RigLayerRole)) throw new Error(`Invalid role for layer ${layer.id}.`);
    if (typeof layer.asset !== "string" || !safeRelativeAsset(layer.asset)) throw new Error(`Unsafe asset path for layer ${layer.id}.`);
    if (!Number.isInteger(layer.zIndex) || typeof layer.required !== "boolean") throw new Error(`Invalid metadata for layer ${layer.id}.`);
    if (!Array.isArray(layer.anchor) || layer.anchor.length !== 2 || !layer.anchor.every(unitInterval)) throw new Error(`Invalid anchor for layer ${layer.id}.`);
  }
  if (!Array.isArray(manifest.motions)) throw new Error("Rig motions must be an array.");
  for (const raw of manifest.motions) {
    if (!raw || typeof raw !== "object") throw new Error("Invalid rig motion.");
    const motion = raw as Record<string, unknown>;
    if (typeof motion.id !== "string" || !ID.test(motion.id) || !positive(motion.durationMs) || typeof motion.loop !== "boolean") throw new Error("Invalid rig motion metadata.");
  }
  return value as RigManifest;
}

function positive(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value) && value > 0;
}

function unitInterval(value: unknown): value is number {
  return typeof value === "number" && Number.isFinite(value) && value >= 0 && value <= 1;
}

function safeRelativeAsset(value: string): boolean {
  return value.length <= 260 && !value.includes("\\") && !value.startsWith("/") && !value.includes(":") && !value.split("/").includes("..");
}
