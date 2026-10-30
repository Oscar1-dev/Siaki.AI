# Character asset creation workflow

This workflow starts from an AI-generated reference but does not confuse a flat reference, automatically extracted pixels, or estimated pivots with a production rig.

## Current admitted reference

- Source: `art/character/saiki-kusuo/source/reference-ai-v1.png`
- Registered clean canvas: `art/character/saiki-kusuo/source/reference-clean-v1.png`
- Status: reference candidate, not a rig
- Provenance: `art/character/saiki-kusuo/provenance.json`
- Runtime preview: visible only in the Character asset-foundation screen and never loaded by the overlay

The clean reference is 2400 × 4400 with alpha. Automated cleanup removed low-alpha glow pixels, but human review is still required around hair, glasses, fingers, shoes, and psychic limiters.

## Pipeline

1. Generate one neutral, front-facing reference with visible hands and joints.
2. Review likeness, anatomy, silhouette, clothing, and rights before using it as an identity anchor.
3. Normalize it to the canonical transparent canvas with `npm run character:prepare-reference -- --input <file>`.
4. Run `npm run character:scaffold` to create the 52-layer plan and mask/layer directories.
5. Author one full-canvas grayscale/alpha mask for each semantic layer under `art/character/saiki-kusuo/masks/`.
6. Run `npm run character:extract` to apply available masks to the admitted reference.
7. Inpaint or redraw pixels that were occluded in the flat reference. Correct seams, overlaps, hands, eyelids, mouth shapes, and alternate poses in a real raster editor.
8. Review and correct the estimated pivots in `layer-plan.json`.
9. Run `npm run character:validate` until all required 2400 × 4400 transparent PNG layers pass.
10. Run `npm run character:build`. It trims exports, calculates hashes and relative transforms, generates `rig.v2.json`, creates a 4096-wide texture atlas and atlas map, and publishes only runtime assets.
11. Load the package with debug bounds/pivots enabled and perform human registration review at 100% and 200%.
12. Only after that review may Phase 22.5B animate the package.

## Commands

```powershell
npm run character:inspect -- --input art/character/saiki-kusuo/source/reference-ai-v1.png
npm run character:prepare-reference -- --input art/character/saiki-kusuo/source/reference-ai-v1.png
npm run character:scaffold
npm run character:preview
npm run character:extract
npm run character:validate
npm run character:build
```

`extract`, `validate`, and `build` fail honestly while masks or required layers are missing. A reference preview never creates `rig.v2.json`.

## What is automated

- Source inspection, dimensions, alpha statistics, and SHA-256 provenance
- Canonical-canvas registration and low-alpha cleanup
- Directory and 52-layer plan generation
- Mask-based extraction once masks exist
- PNG/dimension/alpha validation
- Transparent-bound trimming
- Relative pivot and parent transform calculation
- Manifest generation and hashing
- Deterministic shelf-packed texture atlas and atlas map
- Reference-only WebP preview

## What genuinely requires an external art tool

- Creating accurate semantic masks for hair, face, glasses, clothing, limbs, hands, and effects
- Reconstructing hidden pixels behind hair, sleeves, torso, joints, and glasses
- Drawing closed eyelids, alternate hands, all mouth/viseme shapes, and teleport/aura masks
- Correcting AI anatomy, line continuity, color consistency, and anti-aliased seams
- Producing and preserving a layered PSD or Clip Studio document
- Human approval of likeness and animation deformation

Krita, Photoshop, or Clip Studio Paint can perform this work. AI segmentation such as SAM 2 may accelerate mask creation, but its masks require manual review. Spine or Live2D may assist later deformation, but they require separate licensing decisions; the baseline PixiJS hierarchy does not require them.

## Rights boundary

Saiki Kusuo is third-party intellectual property. This repository records how the reference was produced but does not establish commercial or redistribution rights. Rights review remains mandatory before public release.
