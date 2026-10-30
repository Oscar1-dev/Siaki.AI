# Phase 22.5A — Character asset foundation

## Delivered

- Backward-compatible manifest v2 TypeScript contract and JSON Schema.
- Bounded validation for provenance, paths, hashes, dimensions, hierarchy, semantic slots, and presentation profiles.
- PixiJS layer loader with SHA-256 byte verification, decoded-dimension checks, parent-child construction, and full-body/floating profiles.
- Debug overlay for sprite bounds, pivots, and layer identifiers.
- Native workspace asset-foundation view with the exact artwork handoff list.
- AI-generated full-body reference candidate with recorded prompt, hashes, alpha cleanup, and an explicitly non-rig preview.
- Deterministic source inspection, mask extraction, validation, pivot registration, manifest generation, and atlas-building commands.
- Production overlay now fails honestly when artwork is absent; it does not load the procedural capsule.

## Completion gate

Contract and loader tests must reject traversal, missing parents, hierarchy cycles, digest/dimension mismatch, and incomplete presentation profiles. The running Windows application must visibly report missing artwork without presenting a substitute character.

## Blocker

The actual layered character is not present. A real reference candidate now exists, but Phase 22.5B requires artist-reviewed masks, reconstructed occluded pixels, alternate facial/hand artwork, corrected pivots, and rights review. See [asset-creation-workflow.md](asset-creation-workflow.md) and [artwork-checklist.md](artwork-checklist.md). Phase 22.5A does not claim character animation or final visual quality.

## Verification

- `npm run verify`
- `npm run smoke:windows`
- Native screenshot of the asset-foundation screen and the transparent overlay unavailable state
