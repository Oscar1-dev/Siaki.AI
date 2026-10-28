# Phase 05 — Psychic UI and workspace

Status: complete on the current Windows machine.

## Delivered

- Native 560×300 floating controller and 1040×720 full owner workspace modes.
- Truthful current-session activity view sourced from shell, overlay, and monitor contracts.
- Settings for reduced motion, speech bubbles, overlay pause/resume, and monitor selection.
- Click-through speech bubble reporting the current animation state.
- Typed process-local Rust workspace state and renderer validation.
- Approval inbox that is explicitly unavailable and exposes no approval actions.
- Design tokens and component/state documentation.

## Verification

- `npm run verify`: passed; 5 Phase 00 tests, 5 IPC tests, 9 rig/animation tests, 3 renderer tests, and 7 Rust tests.
- `npm run build:windows`: passed.
- `npm run smoke:windows`: run during final gate; result recorded in the handoff.
- Native inspection confirmed floating-to-full resize and full-to-floating restore.
- The activity view refreshed the startup lifecycle to `ready` and displayed the real three-monitor topology.
- Reduced motion reached the overlay and held the rig at `idle`; disabling speech bubbles removed the bubble.
- Approval navigation displayed `UNAVAILABLE`, the native reason, an empty pending list, and no approve/deny buttons.

## Security and limitations

- Workspace commands can change only native window size and renderer preferences.
- Approval state is deny-by-absence: unavailable with no pending entries or action controls.
- Activity and settings are process-local; SQLite persistence and historical activity are later phases.
- Speech bubbles currently report animation state. Conversation content is not implemented.
- Final artwork, production rig licensing, and release signing remain blocked.
