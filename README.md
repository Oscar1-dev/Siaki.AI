# SAIKI

SAIKI is a Windows-first, local-first autonomous digital operator and animated desktop companion, delivered in gated phases.

## Current status

**Phase 04 — Character animation.** The architecture freeze remains normative. The replaceable PixiJS rig now has an interruptible, measured animation state machine and fixed GPU psychic effects.

The current shell provides typed status/log/exit IPC, native lifecycle logging, error containment, real monitor/DPI discovery, a click-through visual overlay, explicit controller hit region, global pause hotkey, layered asset contracts, and measured procedural character animation. It does not contain final character artwork, autonomous execution, model integration, memory storage, or credential handling.

Start with [repository rules](AGENTS.md), the [architecture freeze](docs/architecture/README.md), [security policy](SECURITY.md), [acceptance matrix](docs/phase-00/acceptance-matrix.md), and [typed contracts](contracts/src/index.ts).

Run `npm ci` and `npm run verify` for cross-layer checks. See [Phase 01](docs/phase-01/README.md) for native build and smoke commands. The repository name is `Siaki.AI`; the product name is **SAIKI**.
