# SAIKI

SAIKI is a Windows-first, local-first autonomous digital operator and animated desktop companion, delivered in gated phases.

## Current status

**Phase 07 — Optional cloud provider.** The architecture freeze remains normative. SAIKI now offers a disabled-by-default OpenAI Responses adapter with Windows Credential Manager isolation and explicit owner disclosure, while local Ollama remains the default.

The current shell provides typed status/log/exit IPC, native lifecycle logging, error containment, real monitor/DPI discovery, a click-through visual overlay, explicit controller hit region, global pause hotkey, layered asset contracts, measured procedural animation, owner workspace controls, a loopback-only Ollama adapter, and an optional owner-enabled OpenAI adapter. It does not contain final character artwork, autonomous execution, memory storage, approval execution, conversation persistence, or persona policy.

Start with [repository rules](AGENTS.md), the [architecture freeze](docs/architecture/README.md), [security policy](SECURITY.md), [acceptance matrix](docs/phase-00/acceptance-matrix.md), and [typed contracts](contracts/src/index.ts).

Run `npm ci` and `npm run verify` for cross-layer checks. See [Phase 07](docs/phase-07/README.md) for the current gate and credential boundary. The repository name is `Siaki.AI`; the product name is **SAIKI**.
