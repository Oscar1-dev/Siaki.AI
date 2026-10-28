import { Container, Graphics } from "pixi.js";
import type { RigManifest } from "./manifest";

export type RigMotionId = "idle" | "focus";

export interface CharacterRig {
  readonly runtimeId: string;
  readonly view: Container;
  load(manifest: RigManifest): Promise<void>;
  play(motion: RigMotionId): void;
  update(deltaMs: number): void;
  resize(width: number, height: number): void;
  destroy(): void;
}

export type RigFactory = () => CharacterRig;

export class PlaceholderPixiRig implements CharacterRig {
  readonly runtimeId = "pixi-placeholder";
  readonly view = new Container();
  #body = new Graphics();
  #elapsedMs = 0;
  #motion: RigMotionId = "idle";
  #loaded = false;
  #originX = 0;
  #originY = 0;

  async load(manifest: RigManifest): Promise<void> {
    if (manifest.runtime !== this.runtimeId) throw new Error(`Rig runtime ${manifest.runtime} is unavailable.`);
    this.#body.clear().circle(0, 0, 78).fill({ color: 0x7d5cff, alpha: 0.12 }).stroke({ color: 0xb7a7ff, alpha: 0.7, width: 2 });
    const core = new Graphics().roundRect(-32, -60, 64, 120, 28).fill({ color: 0x171329, alpha: 0.9 }).stroke({ color: 0xe8e1ff, alpha: 0.65, width: 1 });
    this.view.addChild(this.#body, core);
    this.#loaded = true;
  }

  play(motion: RigMotionId): void {
    this.#assertLoaded();
    this.#motion = motion;
    this.#elapsedMs = 0;
  }

  update(deltaMs: number): void {
    this.#assertLoaded();
    this.#elapsedMs += Math.max(0, Math.min(deltaMs, 100));
    const wave = Math.sin(this.#elapsedMs / (this.#motion === "focus" ? 260 : 600));
    this.view.position.set(this.#originX, this.#originY + wave * (this.#motion === "focus" ? 3 : 8));
    this.view.rotation = wave * (this.#motion === "focus" ? 0.006 : 0.015);
    this.#body.scale.set(1 + wave * 0.025);
  }

  resize(width: number, height: number): void {
    this.#originX = width / 2;
    this.#originY = height / 2;
    this.view.position.set(this.#originX, this.#originY);
    const scale = Math.max(0.55, Math.min(1.4, Math.min(width, height) / 420));
    this.view.scale.set(scale);
  }

  destroy(): void {
    this.view.destroy({ children: true });
    this.#loaded = false;
  }

  #assertLoaded(): void {
    if (!this.#loaded) throw new Error("Rig must be loaded before use.");
  }
}
