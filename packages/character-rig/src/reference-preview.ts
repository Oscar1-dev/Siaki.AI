import { Container, Graphics, Sprite, Texture } from "pixi.js";
import { CharacterStateMachine, type AnimationRequest, type CharacterPose } from "./animation";

export interface ReferencePreviewDescriptor {
  readonly schemaVersion: 1;
  readonly kind: "reference_motion_preview";
  readonly asset: {
    readonly path: string;
    readonly mediaType: "image/png" | "image/webp";
    readonly sha256: `sha256:${string}`;
    readonly width: number;
    readonly height: number;
  };
  readonly provenance: {
    readonly source: string;
    readonly releaseApproved: false;
    readonly license: "unresolved";
  };
}

export interface ReferenceMotionPreview {
  readonly root: Container;
  readonly stateMachine: CharacterStateMachine;
  request(request: AnimationRequest): number;
  update(deltaMs: number): CharacterPose;
  resize(width: number, height: number): void;
  destroy(): void;
}

const DIGEST = /^sha256:[a-f0-9]{64}$/;

export function validateReferencePreviewDescriptor(value: unknown): ReferencePreviewDescriptor {
  if (!record(value) || value.schemaVersion !== 1 || value.kind !== "reference_motion_preview") throw new Error("Invalid reference preview descriptor.");
  if (!record(value.asset) || !safeAssetPath(value.asset.path) || (value.asset.mediaType !== "image/png" && value.asset.mediaType !== "image/webp")) throw new Error("Reference preview asset is invalid.");
  if (typeof value.asset.sha256 !== "string" || !DIGEST.test(value.asset.sha256)) throw new Error("Reference preview digest is invalid.");
  if (!positiveInt(value.asset.width) || !positiveInt(value.asset.height)) throw new Error("Reference preview dimensions are invalid.");
  if (!record(value.provenance) || typeof value.provenance.source !== "string" || !value.provenance.source.trim()) throw new Error("Reference preview provenance is required.");
  if (value.provenance.releaseApproved !== false || value.provenance.license !== "unresolved") throw new Error("Reference preview must remain non-production with unresolved licensing.");
  return value as unknown as ReferencePreviewDescriptor;
}

export async function loadReferenceMotionPreview(raw: unknown, baseUrl: string): Promise<ReferenceMotionPreview> {
  const descriptor = validateReferencePreviewDescriptor(raw);
  const assetUrl = new URL(descriptor.asset.path, baseUrl).toString();
  const response = await fetch(assetUrl, { cache: "no-store" });
  if (!response.ok) throw new Error(`Reference preview artwork is unavailable (${response.status}).`);
  const bytes = await response.arrayBuffer();
  if (bytes.byteLength > 16 * 1024 * 1024) throw new Error("Reference preview artwork exceeds the 16 MiB limit.");
  const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", bytes));
  const actual = `sha256:${[...digest].map((item) => item.toString(16).padStart(2, "0")).join("")}`;
  if (actual !== descriptor.asset.sha256) throw new Error("Reference preview artwork failed SHA-256 verification.");
  const texture = await decodeTexture(assetUrl);
  if (!(texture instanceof Texture)) throw new Error("Reference preview artwork did not decode as a Pixi texture.");
  if (texture.width !== descriptor.asset.width || texture.height !== descriptor.asset.height) throw new Error("Reference preview artwork dimensions do not match its descriptor.");

  const root = new Container({ label: "saiki-reference-motion-preview" });
  const aura = new Graphics({ label: "preview-psychic-aura" });
  const character = new Sprite({ texture, label: "admitted-reference-artwork" });
  character.anchor.set(0.5, 1);
  root.addChild(aura, character);
  const stateMachine = new CharacterStateMachine();
  let fittedScale = 1;

  function redrawAura(intensity: number): void {
    aura.clear();
    if (intensity <= 0.01) return;
    const height = descriptor.asset.height * fittedScale;
    const width = descriptor.asset.width * fittedScale;
    aura.ellipse(0, -height * 0.48, width * 0.46, height * 0.49).fill({ color: 0xd642ff, alpha: 0.045 + intensity * 0.07 });
    aura.ellipse(0, -height * 0.48, width * 0.36, height * 0.43).stroke({ color: 0x78f7ff, alpha: 0.12 + intensity * 0.2, width: 2 });
  }

  function applyPose(pose: CharacterPose): void {
    root.position.set(pose.offsetX, pose.offsetY);
    root.rotation = pose.rotation;
    root.alpha = pose.opacity;
    character.scale.set(fittedScale * pose.bodyScaleX, fittedScale * pose.bodyScaleY);
    redrawAura(pose.effectIntensity);
  }

  return {
    root,
    stateMachine,
    request: (request) => stateMachine.request(request),
    update: (deltaMs) => { const pose = stateMachine.update(deltaMs); applyPose(pose); return pose; },
    resize: (width, height) => {
      fittedScale = Math.min((height * 0.66) / descriptor.asset.height, (width * 0.36) / descriptor.asset.width, 0.55);
      applyPose(stateMachine.update(0));
    },
    destroy: () => root.destroy({ children: true, texture: false, textureSource: false }),
  };
}

function record(value: unknown): value is Record<string, unknown> { return !!value && typeof value === "object" && !Array.isArray(value); }
function positiveInt(value: unknown): value is number { return Number.isInteger(value) && (value as number) > 0 && (value as number) <= 8192; }
function safeAssetPath(value: unknown): value is string { return typeof value === "string" && value.length <= 260 && !value.includes("\\") && !value.startsWith("/") && !value.includes(":") && !value.split("/").includes("..") && /\.(png|webp)$/i.test(value); }
async function decodeTexture(url: string): Promise<Texture> { const image = new Image(); image.src = url; await image.decode(); return Texture.from(image); }
