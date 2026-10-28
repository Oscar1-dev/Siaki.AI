# ADR 0011: Owner workspace state and unavailable approvals

- Status: Accepted for Phase 05
- Date: 2026-10-28

## Context

SAIKI needs an owner-facing workspace before persistence, policy execution, or approval processing exists. Visible controls must not imply capabilities that have not been implemented, and renderer state must not become authorization.

## Decision

Rust owns a small process-local workspace state containing display mode, reduced-motion preference, speech-bubble preference, and approval availability. Typed Tauri commands expose reads and bounded updates. Floating/full mode resizes the native owner window to fixed reviewed dimensions. Character preferences are polled by the overlay and affect only rendering.

The activity view derives entries only from current native shell, overlay, and Windows monitor state. It does not invent historical activity. The approval inbox is typed as `unavailable`, carries a reason, and must contain an empty pending list until a policy-gated executor and exact-payload approval implementation exist. No approve or deny controls are rendered while unavailable.

## Consequences

- Renderer compromise can change UI preferences or workspace size but gains no execution authority.
- Every Phase 05 control performs a current local action; absent capabilities have explicit unavailable states.
- Preferences reset on process restart because SQLite-backed settings are outside this phase.
- Future approval availability is a breaking security boundary and requires policy/executor contracts, tests, and an ADR before enabling controls.
