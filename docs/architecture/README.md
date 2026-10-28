# Architecture freeze

Status: **Frozen for Phase 00**. Scope: boundaries and contracts, not runtime implementation.

## System shape

SAIKI separates cognition from authority. Models produce proposals. A deterministic Rust policy engine evaluates a typed request against owner grants, approval evidence, risk rules, budgets, and current state. Only a scoped executor invokes an adapter. Results are independently classified and verified where possible.

The React/PixiJS renderer presents state and captures owner intent but holds no credentials or general process/filesystem authority. SQLite stores versioned records through Rust. Sidecars isolate browser automation, model hosting, and dependency-heavy capabilities. Dedicated identities remain owner-controlled and use opaque OS-backed secret references.

Dependency direction: `UI -> typed Tauri IPC -> application services -> policy -> executor -> adapter/sidecar/provider`. Core types never import UI, Tauri, provider SDK, or sidecar implementation types.

## Normative artifacts

| Artifact | Purpose |
| --- | --- |
| `AGENTS.md` | Engineering and safety contract |
| `SECURITY.md` | Security objectives and limitations |
| `contracts/src/index.ts` | Typed cross-boundary vocabulary |
| `docs/architecture/adr/` | Frozen decisions and consequences |
| `docs/architecture/diagrams.md` | Components, trust zones, action flow |
| `docs/phase-00/acceptance-matrix.md` | Requirement-to-evidence gate |
| Retained `threatmodel.md` | Threat scenarios and severity calibration |

## Ownership and blockers

The owner controls identities, recovery, repositories, funds, exports, retention, and revocation. SAIKI has no independent legal identity or authorization; autonomy is bounded initiative within owner policy. Memory assertions never become permissions.

Implementation remains blocked on: exact Windows secret-store/recovery design; risk tiers, approval UX, spending limits, and emergency-stop semantics; database encryption/backup assumptions; browser isolation choice; character/voice assets and rights; optional hosted-model disclosure/budget policy; release signing, updates, telemetry, and vulnerability contact. Each affected capability needs a later ADR and owner decision.
