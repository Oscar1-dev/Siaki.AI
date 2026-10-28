# ADR 0009: Character rig boundary

- Status: Accepted for Phase 03
- Date: 2026-10-28

## Context

SAIKI needs a replaceable animated character without coupling application state to one authoring tool, proprietary runtime, or copyrighted asset set. Final artwork and its distribution rights are not yet available.

## Decision

The renderer consumes a versioned layered manifest and a narrow `CharacterRig` lifecycle: `load`, `play`, `update`, `resize`, and `destroy`. Runtime implementations own their Pixi display container. Application code does not inspect bones, textures, proprietary model data, or editor-specific state.

Phase 03 ships a procedural `pixi-placeholder` adapter. It draws geometry at runtime and therefore proves loading, animation, resize, cleanup, and replacement without representing final artwork. Unknown runtime IDs fail as unavailable. Manifest asset locators must be relative and traversal-free.

Tauri retains its strict CSP without `unsafe-eval`. Pixi's `pixi.js/unsafe-eval` compatibility entry is imported because, despite its name, it replaces generated renderer synchronization functions with non-eval polyfills when CSP blocks dynamic evaluation.

No proprietary rig SDK is selected or bundled until the owner has final asset requirements and confirms the applicable editor, runtime, publication, and artwork rights. A future runtime must implement the same adapter and receive a separate dependency/license review.

## Consequences

- Character state can migrate between rig technologies without changing shell IPC.
- Missing or incompatible rigs fail visibly instead of silently falling back to purported final art.
- Runtime-specific advanced features remain behind adapters.
- The manifest is metadata, not authority, executable code, or a permitted network locator.
