# ADR 0022: Production character assets use a validated runtime-neutral package

Status: Accepted for Phase 22.5A.

## Decision

The production character entry point is `/character/saiki-kusuo/rig.v2.json`. Manifest v2 binds explicit provenance, a semantic layer hierarchy, presentation modes, declared image dimensions, and a SHA-256 digest for every PNG or WebP. The PixiJS loader fetches and hashes each bounded asset before decoding it, rejects dimension mismatches, constructs parent-child containers, and exposes an owner-only visual debug overlay for bounds, pivots, and layer names.

The existing manifest v1 and procedural runtime remain source-compatible for historical tests, but the Windows overlay no longer instantiates them. If the production package is absent or invalid, the runtime reports `unavailable` and renders no substitute character.

Full-body and floating presentation profiles are properties of one rig package. Phase 22.5A changes visibility, scale, anchor, and offset only; animation mixing and final character motion belong to Phase 22.5B.

## Consequences

- Missing or altered artwork cannot silently appear as a production character.
- The owner can inspect registration problems without changing the assets.
- Artwork provenance remains explicit, but the repository cannot independently establish that a supplied license is legally sufficient.
- The actual character remains blocked until the required layered source and exports are supplied.
