# Character artwork handoff checklist

## Rights and source

- Confirm in writing that the supplied artwork may be used and distributed with SAIKI.
- Supply `source/saiki-kusuo-master.psd` with layers intact; Clip Studio source is acceptable in addition, not instead of the interchange PSD.
- Supply `provenance.json` naming the artist, source, creation date, license/permission reference, and any redistribution restrictions. Do not place private credentials or contract text in the manifest.

## Canvas and export rules

- Master canvas: 2400 × 4400 px, transparent, character facing front in a neutral standing pose.
- Exported layers: transparent PNG, sRGB, straight alpha, lossless, maximum 8192 px per dimension and 16 MiB per file.
- Every exported layer retains the same 2400 × 4400 canvas and registration; do not trim individual layers.
- Artwork must extend behind occluding hair, clothing, glasses, joints, and facial components so deformation does not expose holes.
- Keep left/right names from the character's perspective.
- Separate line, fill, and shadow groups in the master PSD; the runtime handoff may composite each semantic part into one PNG.
- Put no passwords, signatures, personal addresses, hidden notes, or unrelated material in image metadata.

## Required exports

The runtime screen contains the authoritative 55-path checklist. It covers the full segmented body, clothing, head/hair, independently movable eyes and eyelids, eyebrows, seven mouth/viseme shapes, glasses, limiters, hand variants, and aura/teleport masks.

## Registration reference

Alongside the images, provide a simple table giving each layer's parent, pivot in normalized image coordinates, and resting X/Y position in master-canvas pixels. Required joint pivots include neck, shoulders, elbows, wrists, hips, knees, ankles, eyes, jaw/mouth, glasses, and both limiters.

## Acceptance

- No missing required path.
- Transparent pixels and dimensions decode correctly.
- SHA-256 values in `rig.v2.json` match the exact files.
- Layer hierarchy is acyclic and every reference resolves.
- Full-body and floating profiles both render from the same package.
- Human visual review confirms registration at 100% and 200% scale.
