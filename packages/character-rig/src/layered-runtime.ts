import { Assets, Container, Graphics, Sprite, Text, Texture } from "pixi.js";
import type { PresentationMode, RigAssetV2, RigLayerV2 } from "./manifest-v2";
import { validateRigManifestV2 } from "./manifest-v2";

export interface LoadedLayerV2 { readonly definition: RigLayerV2; readonly sprite: Sprite; readonly node: Container; }
export interface LayeredRigLoadResult { readonly root: Container; readonly layers: ReadonlyMap<string, LoadedLayerV2>; readonly debug: RigDebugOverlay; setMode(mode: PresentationMode): void; destroy(): void; }
export type TextureLoaderV2 = (asset: RigAssetV2, baseUrl: string) => Promise<Texture>;

export async function loadLayeredRigV2(rawManifest: unknown, baseUrl: string, textureLoader: TextureLoaderV2 = loadPixiTexture): Promise<LayeredRigLoadResult> {
  const manifest = validateRigManifestV2(rawManifest);
  const root = new Container({ label: manifest.rigId });
  const loaded = new Map<string, LoadedLayerV2>();
  const assets = new Map(manifest.assets.map((asset) => [asset.id, asset]));
  try {
    for (const layer of [...manifest.layers].sort((a, b) => a.zIndex - b.zIndex)) {
      const asset = assets.get(layer.assetId)!;
      const texture = await textureLoader(asset, baseUrl);
      if (texture.width !== asset.width || texture.height !== asset.height) throw new Error(`Asset ${asset.path} dimensions ${texture.width}x${texture.height} do not match manifest ${asset.width}x${asset.height}.`);
      const node = new Container({ label: layer.id });
      const sprite = new Sprite({ texture, label: layer.semantic });
      sprite.anchor.set(layer.pivot[0], layer.pivot[1]);
      node.position.set(layer.position[0], layer.position[1]);
      node.zIndex = layer.zIndex;
      node.addChild(sprite);
      loaded.set(layer.id, { definition: layer, sprite, node });
    }
    for (const layer of manifest.layers) {
      const item = loaded.get(layer.id)!;
      const parent = layer.parentId ? loaded.get(layer.parentId)?.node : root;
      if (!parent) throw new Error(`Layer ${layer.id} has unavailable parent ${layer.parentId}.`);
      parent.addChild(item.node);
    }
    root.sortableChildren = true;
    const debug = new RigDebugOverlay(root, loaded);
    const setMode = (mode: PresentationMode) => {
      const profile = manifest.profiles.find((item) => item.mode === mode)!;
      root.scale.set(profile.scale);
      root.pivot.set(manifest.logicalSize.width * profile.anchor[0], manifest.logicalSize.height * profile.anchor[1]);
      root.position.set(profile.offset[0], profile.offset[1]);
      for (const item of loaded.values()) item.node.visible = item.definition.modes.includes(mode);
      debug.refresh();
    };
    setMode("full_body");
    return { root, layers: loaded, debug, setMode, destroy: () => root.destroy({ children: true, texture: false, textureSource: false }) };
  } catch (error) { root.destroy({ children: true, texture: false, textureSource: false }); throw error; }
}

export class RigDebugOverlay {
  readonly view = new Container({ label: "rig-debug-overlay" });
  readonly #root: Container;
  readonly #layers: ReadonlyMap<string, LoadedLayerV2>;
  #enabled = false;
  constructor(root: Container, layers: ReadonlyMap<string, LoadedLayerV2>) { this.#root = root; this.#layers = layers; this.view.eventMode = "none"; this.view.zIndex = 20_000; this.#root.addChild(this.view); }
  get enabled(): boolean { return this.#enabled; }
  setEnabled(enabled: boolean): void { this.#enabled = enabled; this.view.visible = enabled; if (enabled) this.refresh(); }
  refresh(): void {
    this.view.removeChildren().forEach((child) => child.destroy());
    if (!this.#enabled) return;
    for (const [id, item] of this.#layers) {
      if (!item.node.visible) continue;
      const bounds = item.sprite.getBounds(); const local = this.#root.toLocal({ x: bounds.x, y: bounds.y });
      const box = new Graphics().rect(local.x, local.y, bounds.width, bounds.height).stroke({ color: 0x58f6ff, alpha: 0.75, width: 1 });
      const pivot = item.node.position;
      const cross = new Graphics().moveTo(pivot.x - 5, pivot.y).lineTo(pivot.x + 5, pivot.y).moveTo(pivot.x, pivot.y - 5).lineTo(pivot.x, pivot.y + 5).stroke({ color: 0xff5ebd, width: 1 });
      const label = new Text({ text: id, style: { fill: 0xffffff, fontFamily: "Segoe UI", fontSize: 10, stroke: { color: 0x000000, width: 3 } } }); label.position.set(local.x + 2, local.y + 2);
      this.view.addChild(box, cross, label);
    }
  }
}

async function loadPixiTexture(asset: RigAssetV2, baseUrl: string): Promise<Texture> {
  const response = await fetch(new URL(asset.path, baseUrl), { cache: "no-store" });
  if (!response.ok) throw new Error(`Required asset ${asset.path} is unavailable (${response.status}).`);
  const bytes = await response.arrayBuffer(); if (bytes.byteLength > 16 * 1024 * 1024) throw new Error(`Asset ${asset.path} exceeds the 16 MiB limit.`);
  await verifyAssetBytes(asset, bytes);
  const objectUrl = URL.createObjectURL(new Blob([bytes], { type: asset.mediaType }));
  try { const texture = await Assets.load<Texture>(objectUrl); if (!(texture instanceof Texture)) throw new Error(`Asset ${asset.path} did not decode as a Pixi texture.`); return texture; }
  finally { URL.revokeObjectURL(objectUrl); }
}

export async function verifyAssetBytes(asset: RigAssetV2, bytes: ArrayBuffer): Promise<void> {
  if (bytes.byteLength > 16 * 1024 * 1024) throw new Error(`Asset ${asset.path} exceeds the 16 MiB limit.`);
  const digest = new Uint8Array(await crypto.subtle.digest("SHA-256", bytes));
  const actual = `sha256:${[...digest].map((value) => value.toString(16).padStart(2, "0")).join("")}`;
  if (actual !== asset.sha256) throw new Error(`Asset ${asset.path} failed SHA-256 verification.`);
}
