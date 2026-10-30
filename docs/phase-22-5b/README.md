# Phase 22.5B — character implementation progress

## Delivered in this checkpoint

- The admitted AI-generated Saiki reference is now visible in the transparent Windows overlay.
- The preview asset is bound to an exact SHA-256 digest, decoded dimensions, provenance and an explicit unresolved-license state.
- PixiJS drives breathing, whole-character gaze response, walking translation, teleport opacity/scale and a GPU-rendered aura through the existing interruptible state machine.
- The production `rig.v2.json` remains absent. A missing production manifest may fall back only to the explicit `reference_motion_preview` descriptor; malformed or unexpected production responses do not silently downgrade.

## Honest boundary

This is a reference motion preview, not the production layered rig. A single flattened image cannot produce correct eyelid, mouth, head, arm, leg, sitting or clothing deformation. Those capabilities remain unavailable until the semantic layers in the Phase 22.5A checklist are reconstructed and reviewed. The preview descriptor structurally requires `releaseApproved: false` and `license: unresolved` so it cannot be mistaken for an admitted release asset.

## Next artwork work

1. Create artist-reviewed masks and reconstruct hidden pixels for the 52-layer plan.
2. Draw closed eyelids, mouth visemes and alternate hands.
3. Validate pivots and deformation seams in Krita, Photoshop or Clip Studio Paint.
4. Run `npm run character:validate` and `npm run character:build` only when every layer is complete.
5. Replace the preview runtime with the validated layered package and connect per-layer animation channels.
