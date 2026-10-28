import { BlurFilter, Container, Graphics } from "pixi.js";
import type { AnimationRequest, CharacterAnimationState, CharacterPose } from "./animation";
import { CharacterStateMachine } from "./animation";
import type { RigManifest } from "./manifest";

export interface CharacterRig {
  readonly runtimeId: string;
  readonly view: Container;
  load(manifest: RigManifest): Promise<void>;
  transition(request: AnimationRequest): number;
  update(deltaMs: number): CharacterPose;
  snapshot(): Readonly<{ state: CharacterAnimationState; elapsedMs: number; transitionId: number }>;
  resize(width: number, height: number): void;
  destroy(): void;
}

export type RigFactory = () => CharacterRig;

export class PlaceholderPixiRig implements CharacterRig {
  readonly runtimeId = "pixi-placeholder";
  readonly view = new Container();
  readonly #machine = new CharacterStateMachine();
  readonly #effects = new Container();
  readonly #aura = new Graphics();
  readonly #body = new Container();
  readonly #core = new Graphics();
  readonly #leftEye = new Graphics();
  readonly #rightEye = new Graphics();
  #loaded = false;
  #originX = 0;
  #originY = 0;
  #layoutScale = 1;

  async load(manifest: RigManifest): Promise<void> {
    if (manifest.runtime !== this.runtimeId) throw new Error(`Rig runtime ${manifest.runtime} is unavailable.`);
    this.#aura.circle(0, 0, 84).fill({ color: 0x7653ff, alpha: 0.16 }).stroke({ color: 0xb7a7ff, alpha: 0.75, width: 2 });
    const outer = new Graphics().circle(0, 0, 108).stroke({ color: 0x7b5cff, alpha: 0.28, width: 2 });
    this.#effects.addChild(outer, this.#aura);
    if (typeof document !== "undefined") this.#effects.filters = [new BlurFilter({ strength: 3, quality: 2 })];
    this.#core.roundRect(-34, -64, 68, 128, 29).fill({ color: 0x171329, alpha: 0.94 }).stroke({ color: 0xe8e1ff, alpha: 0.7, width: 1 });
    this.#leftEye.circle(0, 0, 5).fill(0xded7ff).position.set(-13, -18);
    this.#rightEye.circle(0, 0, 5).fill(0xded7ff).position.set(13, -18);
    this.#body.addChild(this.#core, this.#leftEye, this.#rightEye);
    this.view.addChild(this.#effects, this.#body);
    this.#loaded = true;
  }

  transition(request: AnimationRequest): number {
    this.#assertLoaded();
    return this.#machine.request(request);
  }

  update(deltaMs: number): CharacterPose {
    this.#assertLoaded();
    const pose = this.#machine.update(deltaMs);
    this.view.position.set(this.#originX + pose.offsetX * this.#layoutScale, this.#originY + pose.offsetY * this.#layoutScale);
    this.view.rotation = pose.rotation;
    this.view.alpha = pose.opacity;
    this.#body.scale.set(pose.bodyScaleX * this.#layoutScale, pose.bodyScaleY * this.#layoutScale);
    this.#effects.scale.set(this.#layoutScale * (0.96 + pose.effectIntensity * 0.13));
    this.#effects.rotation = -pose.rotation * 2;
    this.#aura.alpha = 0.25 + pose.effectIntensity * 0.65;
    const eyeX = pose.gazeX * 4;
    const eyeY = pose.gazeY * 3;
    const eyeScaleY = Math.max(0.08, 1 - pose.blink);
    this.#leftEye.position.set(-13 + eyeX, -18 + eyeY);
    this.#rightEye.position.set(13 + eyeX, -18 + eyeY);
    this.#leftEye.scale.set(1, eyeScaleY);
    this.#rightEye.scale.set(1, eyeScaleY);
    return pose;
  }

  snapshot() {
    return this.#machine.snapshot();
  }

  resize(width: number, height: number): void {
    this.#originX = width / 2;
    this.#originY = height / 2;
    this.#layoutScale = Math.max(0.55, Math.min(1.4, Math.min(width, height) / 420));
    this.view.position.set(this.#originX, this.#originY);
  }

  destroy(): void {
    this.view.destroy({ children: true });
    this.#loaded = false;
  }

  #assertLoaded(): void {
    if (!this.#loaded) throw new Error("Rig must be loaded before use.");
  }
}
