# Phase 06 — Local model adapter

## Delivered

- Rust-owned, loopback-only Ollama model discovery.
- NDJSON chat streaming through typed Tauri events.
- Per-request cancellation and bounded 1–600 second timeout handling.
- Structured function calls surfaced only as untrusted, non-executable proposals.
- Owner workspace model view with honest idle, streaming, completed, failed, cancelled, and unavailable states.
- Input bounds and runtime contract validation on both sides of IPC.

## Verification gate

Verified on Windows on 2026-10-28 with Ollama 0.40.12 and installed `qwen3:4b`:

- Native discovery reported one local model and selected `qwen3:4b` (2.5 GB).
- The owner workspace streamed a real response and reached `SUCCEEDED` with `SAIKI_STREAM_OK`.
- A second long-running request was cancelled and reached `CANCELLED` before its first token.
- Automated Rust coverage verifies timeout maps to `unavailable` and explicitly states that no cloud fallback was attempted.
- `npm run verify` and `npm run build:windows` passed; the latter produced `target/release/saiki.exe`.

## Limitations

- Phase 06 does not persist conversations or persona state; those begin in Phase 08.
- No cloud provider is present. Phase 07 must introduce its own disclosure and credential contract.
- Tool proposals are view-only model output. The policy executor belongs to a later phase.
- Ollama cancellation is client-side stream cancellation; Ollama may take a short time to release model work.
