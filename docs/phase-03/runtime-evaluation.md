# Rig runtime evaluation

Reviewed 2026-10-28. This is an engineering evaluation, not legal advice.

| Candidate | Fit | License consequence | Phase 03 decision |
| --- | --- | --- | --- |
| PixiJS procedural/layered sprites | Native match for the frozen PixiJS renderer; sufficient for adapter and layered-art validation | MIT package metadata is retained through npm; no proprietary editor required | Selected for the placeholder only |
| Spine Pixi runtime | Mature skeletal animation and official PixiJS support | Runtime integration and distribution depend on a qualifying Spine Editor license; trial rights do not grant runtime distribution | Technically viable, blocked on owner license and final rig format |
| Live2D Cubism SDK | Strong 2D deformation and expression workflow | Proprietary terms; publication requirements depend on usage and SAIKI may need the Expandable Application review path | Technically viable, blocked on written licensing determination and final art |

Primary references:

- PixiJS repository/license: <https://github.com/pixijs/pixijs>
- Spine runtimes and license: <https://en.esotericsoftware.com/spine-runtimes>, <https://en.esotericsoftware.com/spine-runtimes-license>
- Live2D SDK release licensing: <https://www.live2d.com/en/sdk/license/>

No Spine or Live2D dependency, model, sample asset, or runtime code is included. Selection remains deliberately unresolved rather than implying distribution rights.
