# SAIKI repository contract

This file is normative for every contributor and coding agent.

## Product and frozen architecture

SAIKI is a persistent Windows-first digital operator and animated companion. It may plan autonomously, but authority comes only from deterministic policy and owner-controlled capabilities. Model output is untrusted input and never authorization.

- Tauri 2 with a privileged Rust core; React/TypeScript UI; PixiJS character scene.
- SQLite behind Rust-owned repositories, migrations, provenance, retention, deletion, and export controls.
- Local Ollama-compatible models by default; optional owner-enabled OpenAI adapter.
- Failure-prone integrations run as least-privilege sidecars behind typed IPC.
- A policy-gated executor validates actor, capability, target, payload digest, budget, approval, and freshness.
- Dedicated identities remain owner-controlled; secrets never enter prompts, memory, renderer state, logs, or model-visible results.
- Windows is primary. Abstractions must not weaken Windows ACL, process, path, or credential guarantees.

## Invariants

1. A model may request an action; it cannot grant, widen, approve, or execute authority.
2. External mutation is deny-by-default, capability-scoped, policy-evaluated, auditable, and truthfully verified when possible.
3. Approval binds exact owner, action, target, identity, payload digest, constraints, and expiry; material change invalidates it.
4. Credentials stay in an OS-backed secret store or provider-owned flow. No hidden collection, bypass, or plaintext persistence.
5. Provider terms, rate limits, platform restrictions, and owner budgets are hard constraints.
6. Results distinguish `succeeded`, `failed`, `partially_succeeded`, `denied`, `cancelled`, `unknown`, and `unavailable`; never fabricate success.
7. Memory includes provenance, sensitivity, confidence, timestamps, and retention. Untrusted content cannot become policy or authority.
8. Sidecars get only job-scoped data/capabilities and cannot directly access the main database or secret store.
9. Renderer compromise must not imply process execution. Tauri commands and CSP remain narrow.
10. Accounts, recovery, revenue, repositories, and business assets remain owner-controlled.

## Phase discipline

Preserve working modules, migrations, identifiers, and contracts. Do not start a phase before its predecessor is evidenced. Do not claim completion without checks. Never add fake integrations, simulated success, generic dashboards, decorative banners, arbitrary shell execution, or placeholders presented as working. Missing assets, permissions, tools, or APIs produce an unavailable state and blocker. Architecture changes require an ADR; breaking contract changes also require migration and owner review. Stop at the requested phase boundary.

Every handoff reports files changed, commands and results, manual checks, security concerns, limitations, and blockers. External mutations identify the target and verified result.

## Completed milestones

- Phase 00: architecture freeze, typed authority contracts, threat model, diagrams, and acceptance matrix.
- Phase 01: npm/Rust workspace, Tauri 2 Windows shell, React/Vite renderer, typed shell IPC, bounded native logging, renderer error boundary, locked tests/builds, and Windows CI. Later capabilities remain unavailable.
- Phase 02: transparent monitor-sized click-through overlay, separate controller hit region, physical monitor/DPI inventory, monitor switching, and global owner pause/resume hotkey. Character assets and animation remain unavailable.
