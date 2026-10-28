# Phase 08 — Conversation and persona

## Delivered

- Rust-owned Saiki persona policy: fixed display name, declared `authority = none`, owner-facing tone descriptors, and an advisory system preamble that explicitly cannot change facts or authorize actions.
- Persisted conversation store (atomic JSON in the app config directory, the same durability pattern as the cloud provider config). Conversations survive restart.
- Three typed message kinds: `owner`, `saiki`, and `tool_result`. A `tool_result` is a factual record — stored verbatim, carrying no persona expression, and never rewritten by the persona.
- Deterministic context assembly with hard limits (≤24 messages and ≤24 KiB per model context window; ≤200 stored messages; ≤32 KiB per message). The persona preamble is always prepended; tool results are copied verbatim.
- Deterministic, advisory character expressions (`neutral`, `talking`, `focused`, `annoyed`, `surprised`, `tired`) derived only from Saiki's own reply text, emitted as a typed `saiki://expression` event.
- Owner workspace "Conversation" tab: persisted transcript, persona disclosure, context-limit note, and reply generation that reuses the Phase 06 local Ollama adapter. A separate control records a factual tool result verbatim to exercise the invariant.
- Typed IPC contracts, runtime guards, and command-name stability for all six new commands.

## Completion gate

The gate is "conversation persists and personality never overrides factual tool results."

- **Persistence** is covered by a round-trip test: a fresh `ConversationState` loaded from the same directory reconstructs prior messages, the way a restart would. The renderer restores and displays the stored transcript on load.
- **Persona cannot override facts** is enforced structurally and verified by tests: a `tool_result` is stored verbatim, appears byte-identical in assembled persona context, carries no persona expression, and the persona summary declares `authority = none`. The contract rejects any message role or persona authority value outside the allowed set.

## Boundaries (not in this phase)

- **No SQLite.** The SQLite migration/FTS foundation is Phase 09; conversation storage here is a deliberate JSON-backed interim.
- **No policy executor.** Tool results are recorded verbatim through an explicit owner action; the executor that will produce them automatically is Phase 12. Model tool calls remain untrusted, non-executable proposals (Phase 06).
- **No autonomy.** Replies are owner-initiated; the wake loop and supervisor are Phase 18.

## Limitations

- Reply generation requires a running local Ollama model (reused from Phase 06); without one the view reports an honest unavailable/failed state and stores nothing.
- The `saiki://expression` event is emitted for the overlay character to consume; live rig consumption of conversation-driven expressions is a later integration and is not wired in this phase.
- Expression classification is a small deterministic keyword mapping, not a model-driven sentiment analysis; it is advisory styling only.
- Persistence is unencrypted local JSON; at-rest encryption and retention/deletion policy arrive with the Phase 09 memory foundation.
