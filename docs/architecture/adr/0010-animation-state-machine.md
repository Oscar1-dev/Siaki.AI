# ADR 0010: Interruptible character animation state machine

- Status: Accepted for Phase 04
- Date: 2026-10-28

## Context

Character motion must remain deterministic, testable, and replaceable across rig runtimes. Rendering callbacks cannot become an authority or accumulate an unbounded queue of stale model-requested gestures.

## Decision

Animation requests use the closed states `idle`, `gaze`, `blink`, `walk`, `sit`, and `teleport`. A request atomically replaces the current state, clears elapsed time, and increments a transition identifier. There is no queued transition backlog. Inputs are clamped, each update advances by at most 100 ms, timed states return to idle, and sitting persists until interrupted.

The state machine returns a renderer-neutral pose. The Pixi adapter maps that pose onto procedural geometry, eye movement, blink scale, body transforms, opacity, aura intensity, and a GPU blur filter. Application or future model output may request a typed animation but cannot inject shader code, asset paths, or executable callbacks.

Performance telemetry maintains bounded rolling samples for update CPU duration and observed ticker frame interval. The Phase 04 local target is update p95 below 2 ms and frame-interval p95 below 16.67 ms after at least 240 release-build samples. Measurements describe only the named machine, workload, renderer, and session.

## Consequences

- Any state can be interrupted immediately without waiting for an animation to finish.
- Runtime adapters receive poses rather than owning behavioral decisions.
- A 100 ms update clamp prevents large resume gaps from skipping entire transitions.
- Future user or policy controls can issue the same typed requests without widening the renderer boundary.
- Telemetry arithmetic is evidence for this adapter run, not universal GPU or hardware proof.
