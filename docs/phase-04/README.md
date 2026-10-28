# Phase 04 — Character animation

Status: complete on the current Windows machine.

## Delivered

- Deterministic, renderer-neutral animation state machine.
- Interruptible `idle`, `gaze`, `blink`, `walk`, `sit`, and `teleport` states.
- Clamped gaze/direction inputs, bounded time advancement, timed idle fallback, and stable sitting.
- Pixi pose application for eye gaze, blink, movement, sitting, teleport opacity/scale, and psychic aura intensity.
- GPU-rendered Pixi blur effect with the existing strict CSP-safe renderer path.
- Bounded rolling frame telemetry with median/p95 accounting and over-budget counts.
- Automatic local demonstration sequence; no model or external input can inject code or shaders.

## Completion evidence

- `npm run verify`: passed; 5 Phase 00 tests, 4 IPC tests, 9 rig/animation tests, 2 renderer tests, and 5 Rust tests.
- `npm run build:windows`: passed.
- `npm run smoke:windows`: passed with clean exit code 0 after 1,939 ms.
- Release build, 2560×1440 overlay, Pixi WebGL path, psychic blur enabled, current Windows power/hardware state.
- Native bounded sample: 240 frames, update CPU p95 0.100 ms, observed frame-interval p95 6.200 ms, zero samples over the 16.67 ms budget.
- Manual native inspection confirmed movement between sampled states, visible gaze/blink changes, transparent GPU aura, and preserved click-through behavior.

The target was met on the first measured release workload, so no optimization loop or causal performance claim was necessary. These timings are session-specific adapter telemetry, not proof of GPU synchronization, universal hardware performance, or statistical significance.

## Security and limitations

- Animation requests select a closed typed state; they do not authorize side effects.
- The GPU effect uses fixed application shaders/filter configuration, not model-provided shader source.
- The procedural character remains a rig-validation asset, not final SAIKI artwork.
- The demonstration sequence is local scaffolding for Phase 04 evidence and is not an autonomous behavior planner.
- Final artwork, licensed production rig selection, signed release, and long-duration performance testing remain blocked.
