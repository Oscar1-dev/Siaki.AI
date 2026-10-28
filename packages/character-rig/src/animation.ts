export type CharacterAnimationState = "idle" | "gaze" | "blink" | "walk" | "sit" | "teleport";

export interface AnimationRequest {
  readonly state: CharacterAnimationState;
  readonly gaze?: { readonly x: number; readonly y: number };
  readonly direction?: -1 | 1;
}

export interface CharacterPose {
  readonly state: CharacterAnimationState;
  readonly elapsedMs: number;
  readonly gazeX: number;
  readonly gazeY: number;
  readonly blink: number;
  readonly offsetX: number;
  readonly offsetY: number;
  readonly rotation: number;
  readonly bodyScaleX: number;
  readonly bodyScaleY: number;
  readonly opacity: number;
  readonly effectIntensity: number;
}

const DURATION_MS: Readonly<Record<CharacterAnimationState, number>> = {
  idle: Number.POSITIVE_INFINITY,
  gaze: 900,
  blink: 180,
  walk: 1_600,
  sit: 1_200,
  teleport: 720,
};

export class CharacterStateMachine {
  #request: AnimationRequest = { state: "idle" };
  #elapsedMs = 0;
  #transitionId = 0;

  request(request: AnimationRequest): number {
    this.#request = normalizeRequest(request);
    this.#elapsedMs = 0;
    return ++this.#transitionId;
  }

  update(deltaMs: number): CharacterPose {
    this.#elapsedMs += clamp(deltaMs, 0, 100);
    if (this.#elapsedMs >= DURATION_MS[this.#request.state] && this.#request.state !== "sit") this.request({ state: "idle" });
    return poseFor(this.#request, this.#elapsedMs);
  }

  snapshot(): Readonly<{ state: CharacterAnimationState; elapsedMs: number; transitionId: number }> {
    return { state: this.#request.state, elapsedMs: this.#elapsedMs, transitionId: this.#transitionId };
  }
}

export interface FrameBudgetSnapshot {
  readonly sampleCount: number;
  readonly updateMedianMs: number;
  readonly updateP95Ms: number;
  readonly frameP95Ms: number;
  readonly framesOverBudget: number;
  readonly budgetMs: number;
}

export class FrameBudgetTracker {
  readonly #budgetMs: number;
  readonly #capacity: number;
  readonly #updateSamples: number[] = [];
  readonly #frameSamples: number[] = [];

  constructor(budgetMs = 16.67, capacity = 600) {
    if (!(budgetMs > 0) || !Number.isInteger(capacity) || capacity < 1) throw new Error("Invalid frame budget configuration.");
    this.#budgetMs = budgetMs;
    this.#capacity = capacity;
  }

  record(updateMs: number, frameMs: number): void {
    pushBounded(this.#updateSamples, Math.max(0, updateMs), this.#capacity);
    pushBounded(this.#frameSamples, Math.max(0, frameMs), this.#capacity);
  }

  snapshot(): FrameBudgetSnapshot {
    const updates = [...this.#updateSamples].sort((a, b) => a - b);
    const frames = [...this.#frameSamples].sort((a, b) => a - b);
    return {
      sampleCount: frames.length,
      updateMedianMs: percentile(updates, 0.5),
      updateP95Ms: percentile(updates, 0.95),
      frameP95Ms: percentile(frames, 0.95),
      framesOverBudget: frames.filter((sample) => sample > this.#budgetMs).length,
      budgetMs: this.#budgetMs,
    };
  }
}

function poseFor(request: AnimationRequest, elapsedMs: number): CharacterPose {
  const state = request.state;
  const duration = DURATION_MS[state];
  const progress = Number.isFinite(duration) ? clamp(elapsedMs / duration, 0, 1) : 0;
  const wave = Math.sin(elapsedMs / 600);
  let blink = state === "blink" ? Math.sin(progress * Math.PI) : 0;
  let offsetX = 0;
  let offsetY = wave * 6;
  let rotation = wave * 0.012;
  let bodyScaleX = 1;
  let bodyScaleY = 1;
  let opacity = 1;
  let effectIntensity = 0.15;
  if (state === "walk") {
    const direction = request.direction ?? 1;
    offsetX = direction * (progress * 180 - 90);
    offsetY = Math.abs(Math.sin(progress * Math.PI * 8)) * -7;
    rotation = Math.sin(progress * Math.PI * 8) * 0.04;
  } else if (state === "sit") {
    offsetY = 36;
    bodyScaleX = 1.08;
    bodyScaleY = 0.76;
    rotation = 0;
  } else if (state === "teleport") {
    const edge = Math.min(progress * 3, (1 - progress) * 3, 1);
    opacity = edge;
    bodyScaleX = 0.75 + edge * 0.25;
    bodyScaleY = 1.35 - edge * 0.35;
    effectIntensity = 1 - edge * 0.25;
    rotation = progress * Math.PI * 0.08;
  } else if (state === "gaze") {
    effectIntensity = 0.35;
  } else if (state === "blink") {
    blink = Math.max(0, blink);
  }
  return { state, elapsedMs, gazeX: clamp(request.gaze?.x ?? 0, -1, 1), gazeY: clamp(request.gaze?.y ?? 0, -1, 1), blink, offsetX, offsetY, rotation, bodyScaleX, bodyScaleY, opacity, effectIntensity };
}

function normalizeRequest(request: AnimationRequest): AnimationRequest {
  if (request.state === "gaze") return { state: "gaze", gaze: { x: clamp(request.gaze?.x ?? 0, -1, 1), y: clamp(request.gaze?.y ?? 0, -1, 1) } };
  if (request.state === "walk") return { state: "walk", direction: request.direction === -1 ? -1 : 1 };
  return { state: request.state };
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, Number.isFinite(value) ? value : 0));
}

function pushBounded(target: number[], value: number, capacity: number): void {
  if (target.length === capacity) target.shift();
  target.push(value);
}

function percentile(sorted: readonly number[], p: number): number {
  if (sorted.length === 0) return 0;
  return sorted[Math.min(sorted.length - 1, Math.ceil(sorted.length * p) - 1)] ?? 0;
}
