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
- Phase 03: versioned layered rig manifest, runtime-neutral adapter contract, licensed-runtime evaluation, and animated procedural PixiJS rig spike. Final artwork and proprietary runtime selection remain unavailable.
- Phase 04: interruptible renderer-neutral state machine for gaze, blink, walk, sit, and teleport; procedural Pixi pose application, fixed GPU psychic effects, and bounded frame telemetry. Final artwork remains unavailable.
- Phase 05: native floating/full owner workspace, truthful current-session activity, process-local character/overlay settings, animation-state speech bubbles, and an explicitly unavailable approval inbox with no fake requests or actions.
- Phase 06: loopback-only Ollama discovery and streaming, cancellation, bounded timeout behavior, and non-executable untrusted tool proposals.
- Phase 07: optional disabled-by-default OpenAI Responses adapter, Windows Credential Manager isolation, explicit data/billing disclosure, owner-selected model, streaming, cancellation, and truthful unavailable states.
- Phase 08: Rust-owned Saiki persona policy with declared zero authority, persisted JSON conversation store surviving restart, verbatim-immutable `tool_result` records the persona cannot override, bounded persona-framed context assembly, deterministic advisory character expressions, and an owner conversation view reusing the local model adapter. SQLite memory foundation remains deferred to Phase 09.
- Phase 09: Rust-owned SQLite memory foundation (bundled FTS5) with versioned migrations, provenance-bearing claims (source, confidence, sensitivity, timestamps), teach/correct/forget persisting across restart, corrections that supersede without destroying audit history, forget that deletes content while logging the deletion, sanitized FTS keyword retrieval, JSON export, and an owner memory view. Embeddings/vector retrieval and at-rest encryption remain deferred (Phase 10+).
- Phase 10: Rust-owned local document knowledge (SQLite) — UTF-8 file ingestion with size/type validation, blank-line chunking, a deterministic fully-local lexical embedding (FNV-hashed term-frequency vectors, cosine retrieval), citations carrying the real source path and chunk, owner-declared privacy labels that exclude sensitive documents from retrieval unless opted in, document deletion, and an owner knowledge view. Neural embeddings, workspace path confinement, and at-rest encryption remain deferred.
- Phase 11: voice and lip-sync, partially delivered with honest blockers. Real and tested: deterministic text→viseme lip-sync timeline, a push-to-talk/interruption/cancellation session state machine, persisted owner engine-asset configuration (Whisper/Piper paths), truthful capability reporting, and an owner voice view with visible mic status. Microphone capture, STT transcription, and TTS audio playback are reported as explicit `unavailable` capabilities with blockers (native-audio engines/models/mic not wired); end-to-end spoken voice remains unavailable until those are provided.
- Phase 12: Rust-owned SQLite policy engine — the deterministic authority boundary. Deny-by-default evaluation (allow/deny/approval_required) over risk tiers, typed revocable capability grants (resource/action/spend/expiry scoped), an approval queue binding an exact SHA-256 payload digest, an emergency kill switch overriding all grants, and an append-only audit log with no edit/delete path. The model's only entry is evaluate_action; all grant/approval/kill commands are owner-only, so model output cannot authorize side effects. The policy-gated executor (acting on allow with idempotency/rollback) remains a later phase.
- Phase 13: Rust-owned Windows and file tools, workspace-confined. A single confine() chokepoint rejects absolute paths and ".." and canonicalizes to keep every file operation inside the owner-configured workspace root (symlink escapes caught); scoped list/read/write with reversible writes via rollback tokens; truthful std-only system metrics; and allowlist-only application launching. Owner system/files view. Routing these through the policy executor remains a later phase.
