# SAIKI

SAIKI is a Windows-first, local-first autonomous digital operator and animated desktop companion, delivered in gated phases.

## Current status

**Phase 17 — World database and event bus.** SAIKI now has durable event-sourced project entities, versioned lifecycle transitions and snapshots, a live typed event bus, and restart reconstruction that verifies current projections against immutable history.

The current shell provides typed status/log/exit IPC, native lifecycle logging, error containment, real monitor/DPI discovery, a click-through visual overlay, explicit controller hit region, global pause hotkey, layered asset contracts, measured procedural animation, owner workspace controls, a loopback-only Ollama adapter, and an optional owner-enabled OpenAI adapter. It does not contain final character artwork, autonomous execution, memory storage, approval execution, conversation persistence, or persona policy.

Start with [repository rules](AGENTS.md), the [architecture freeze](docs/architecture/README.md), [security policy](SECURITY.md), [acceptance matrix](docs/phase-00/acceptance-matrix.md), and [typed contracts](contracts/src/index.ts).

Run `npm ci` and `npm run verify` for cross-layer checks. See [Phase 17](docs/phase-17/README.md) for the current gate and limitations. The repository name is `Siaki.AI`; the product name is **SAIKI**.
