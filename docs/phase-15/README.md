# Phase 15 — Browser automation

## Delivered

- Rust-owned SQLite task/checkpoint repository with restart-safe ordered checkpoints.
- Typed task, checkpoint, status, and step-decision IPC contracts.
- HTTP/HTTPS-only task URLs and bounded task/page inputs.
- SHA-256 checkpoint digests; raw untrusted page text is not persisted.
- Deterministic provenance gate: page-derived sensitive click/fill proposals are denied before the policy engine.
- Phase 12 policy evaluation for every owner/planner browser proposal.
- Owner workspace view that reports runtime availability and exposes a real prompt-injection security probe.

## Gate evidence

The native test `page_injection_cannot_authorize_sensitive_action` sends page content containing an instruction to bypass policy and fill a sensitive target. The result is `denied` with `executed: false`. Contract tests reject any result that claims page-authorized execution.

## Honest blocker

No Node/Playwright sidecar or browser engine is bundled. Navigation, click, fill, extraction, downloads, and session execution remain `unavailable`. Merely placing runtime files on disk does not change this: authenticated job-scoped IPC is also required. Future provisioning must use a dedicated non-owner profile and may not import owner cookies, credentials, or browser state.

## Verification

- `cargo test --workspace --locked`
- `npm run check`
- `npm test`
- `npm run build:web`
- Manual native check: open Browser, confirm `UNAVAILABLE`, create a task record, run the injection probe, and observe `denied`.
