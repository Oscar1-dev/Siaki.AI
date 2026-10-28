# SAIKI

SAIKI is a Windows-first, local-first autonomous digital operator and animated desktop companion, delivered in gated phases.

## Current status

**Phase 06 — Local model adapter.** The architecture freeze remains normative. SAIKI now discovers installed Ollama models and streams cancellable local responses while keeping model tool calls non-executable and untrusted.

The current shell provides typed status/log/exit IPC, native lifecycle logging, error containment, real monitor/DPI discovery, a click-through visual overlay, explicit controller hit region, global pause hotkey, layered asset contracts, measured procedural animation, owner workspace controls, and a loopback-only Ollama adapter. It does not contain final character artwork, cloud model integration, autonomous execution, memory storage, approval execution, or credential handling.

Start with [repository rules](AGENTS.md), the [architecture freeze](docs/architecture/README.md), [security policy](SECURITY.md), [acceptance matrix](docs/phase-00/acceptance-matrix.md), and [typed contracts](contracts/src/index.ts).

Run `npm ci` and `npm run verify` for cross-layer checks. See [Phase 06](docs/phase-06/README.md) for the current gate and [Phase 01](docs/phase-01/README.md) for native build and smoke commands. The repository name is `Siaki.AI`; the product name is **SAIKI**.
