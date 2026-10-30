export const CHARACTER_MANIFEST_URL = "/character/saiki-kusuo/rig.v2.json";
export const CHARACTER_REFERENCE_PREVIEW_URL = "/character/saiki-kusuo/reference/reference-preview.v1.json";
export const CHARACTER_DEBUG_STORAGE_KEY = "saiki.character.debug.v1";
export const REQUIRED_CHARACTER_FILES = [
  "source/saiki-kusuo-master.psd", "layers/back/back-hair.png", "layers/body/torso.png", "layers/body/hips.png",
  "layers/body/upper-leg-left.png", "layers/body/lower-leg-left.png", "layers/body/foot-left.png", "layers/body/upper-leg-right.png", "layers/body/lower-leg-right.png", "layers/body/foot-right.png",
  "layers/body/upper-arm-left.png", "layers/body/forearm-left.png", "layers/body/hand-left-relaxed.png", "layers/body/hand-left-psychic.png", "layers/body/upper-arm-right.png", "layers/body/forearm-right.png", "layers/body/hand-right-relaxed.png", "layers/body/hand-right-psychic.png",
  "layers/clothing/jacket-body.png", "layers/clothing/jacket-sleeve-left.png", "layers/clothing/jacket-sleeve-right.png", "layers/clothing/trousers.png", "layers/clothing/shoes.png",
  "layers/head/neck.png", "layers/head/head-base.png", "layers/head/ear-left.png", "layers/head/ear-right.png", "layers/head/front-hair.png", "layers/head/side-hair-left.png", "layers/head/side-hair-right.png",
  "layers/face/eye-white-left.png", "layers/face/eye-white-right.png", "layers/face/iris-left.png", "layers/face/iris-right.png", "layers/face/eyelid-left-open.png", "layers/face/eyelid-left-closed.png", "layers/face/eyelid-right-open.png", "layers/face/eyelid-right-closed.png", "layers/face/eyebrow-left.png", "layers/face/eyebrow-right.png",
  "layers/face/mouth-neutral.png", "layers/face/mouth-a.png", "layers/face/mouth-e.png", "layers/face/mouth-i.png", "layers/face/mouth-o.png", "layers/face/mouth-u.png", "layers/face/mouth-mbp.png",
  "layers/accessories/glasses-frame.png", "layers/accessories/glasses-lenses.png", "layers/accessories/limiter-left.png", "layers/accessories/limiter-right.png",
  "layers/effects/aura-mask.png", "layers/effects/teleport-mask.png", "provenance.json", "rig.v2.json",
] as const;
