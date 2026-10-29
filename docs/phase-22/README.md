# Phase 22 — Business experiment engine

## Delivered

- Rust-owned SQLite experiments with ideas, hypotheses, budget caps, stages, and stop state.
- Provenance-labelled research notes, including an explicit untrusted-web-content classification.
- Typed KPI definitions and evidence-referenced observations.
- Deterministic `insufficient_evidence`, `hold`, or `advance` evaluations.
- Evidence-gated stage transitions and a workspace for real experiment operations.
- Explicit unavailable states for financial execution and actual-spend tracking.

## Completion gate

`evaluation_never_fabricates_revenue`, `observed_revenue_is_source_labelled`, `cannot_advance_without_a_passing_gate`, and `budget_is_not_actual_spend` verify the core gate. Revenue appears only when an explicit revenue KPI observation exists; a budget never becomes spend or revenue.

## Security and limitations

- Model output, research notes, and untrusted page content cannot authorize a stage or external action.
- Observation evidence references are provenance labels, not cryptographic proof.
- No payments, purchases, ads, provider mutation, automated research, bookkeeping, or tax logic exists.
- SQLite data is not encrypted at rest by this phase.
- Currency conversion and cross-currency aggregation are intentionally absent.

## Verification

- `npm run verify`
- `npm run smoke:windows`
- Manual: create an experiment, add a note and KPI, verify evaluation is insufficient before an observation, record an observation, evaluate again, and confirm the stage can advance only after an `advance` result.
