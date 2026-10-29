# Phase 22.5A — Character asset foundation

## Delivered

- Backward-compatible manifest v2 TypeScript contract and JSON Schema.
- Bounded validation for provenance, paths, hashes, dimensions, hierarchy, semantic slots, and presentation profiles.
- PixiJS layer loader with SHA-256 byte verification, decoded-dimension checks, parent-child construction, and full-body/floating profiles.
- Debug overlay for sprite bounds, pivots, and layer identifiers.
- Native workspace asset-foundation view with the exact artwork handoff list.
- Production overlay now fails honestly when artwork is absent; it does not load the procedural capsule.

## Completion gate

Contract and loader tests must reject traversal, missing parents, hierarchy cycles, digest/dimension mismatch, and incomplete presentation profiles. The running Windows application must visibly report missing artwork without presenting a substitute character.

## Blocker

The actual character is not present. Phase 22.5B requires the licensed layered artwork described in [artwork-checklist.md](artwork-checklist.md). Phase 22.5A does not claim character animation or final visual quality.

## Verification

- `npm run verify`
- `npm run smoke:windows`
- Native screenshot of the asset-foundation screen and the transparent overlay unavailable state
