# Phase 03 — Asset pipeline and rig spike

Status: complete on the current Windows machine.

## Delivered

- `@saiki/character-rig` package with a runtime-neutral adapter contract.
- Versioned TypeScript manifest, runtime validator, and JSON Schema for layered assets and motions.
- Relative, traversal-free asset locator enforcement and duplicate layer rejection.
- PixiJS procedural placeholder adapter with load, idle/focus animation, resize, and cleanup.
- Overlay integration that exposes an explicit unavailable state if initialization fails.
- ADR 0009 and a first-party-source license/runtime evaluation.

## Acceptance boundary

The placeholder is intentionally generated geometry. It is not final SAIKI artwork and makes no artwork, likeness, or distribution-right assumptions. A licensed rig runtime is not bundled because the owner has not selected an authoring format or provided the relevant license.

## Verification

- `npm run verify`: passed; 5 Phase 00 tests, 4 IPC tests, 5 rig tests, 2 renderer tests, and 5 Rust tests.
- `npm run build:windows`: passed and produced the native release executable.
- `npm run smoke:windows`: passed with clean exit code 0 after 1,998 ms.
- Manual native inspection confirmed the animated procedural rig rendered centrally on a 2560×1440 monitor.
- Moving to the 1080×1920 portrait monitor re-centered and rescaled the same rig while preserving transparency and click-through.
- Global pause hid the overlay and the hotkey restored it while Spotify was foreground.
- Native inspection initially exposed Pixi's CSP check and an animation-origin defect; both were fixed, the strict CSP remained intact, and the origin regression is covered by a test.

## Limitations and blockers

- Final layered artwork and documented rights are missing.
- Final rig technology and editor/runtime license are owner decisions.
- Phase 04 motion state machine, transitions, gaze, blink, walk, sit, teleport, effects, and frame-budget instrumentation are not implemented.
- The placeholder supports only `idle` and `focus` to exercise the adapter contract.
