# SAIKI

SAIKI is a Windows-first, local-first autonomous digital operator and animated desktop companion, delivered in gated phases.

## Current status

**Phase 01 — Monorepo and Windows shell.** The architecture freeze remains normative. The repository now contains a minimal native shell and no autonomous capabilities.

The current shell provides typed status/log/exit IPC, native lifecycle logging, error containment, tests, locked builds, and Windows CI. It does not contain autonomous execution, model integration, overlay behavior, memory storage, or credential handling.

Start with [repository rules](AGENTS.md), the [architecture freeze](docs/architecture/README.md), [security policy](SECURITY.md), [acceptance matrix](docs/phase-00/acceptance-matrix.md), and [typed contracts](contracts/src/index.ts).

Run `npm ci` and `npm run verify` for cross-layer checks. See [Phase 01](docs/phase-01/README.md) for native build and smoke commands. The repository name is `Siaki.AI`; the product name is **SAIKI**.
