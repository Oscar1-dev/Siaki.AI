# ADR 0021: Business experiments are evidence-bound records

Status: Accepted for Phase 22.

## Decision

Business experiments are Rust-owned, durable records containing an idea, falsifiable hypothesis, declared budget cap, research notes, KPI definitions, observations, evaluations, and explicit stage gates. A declared budget is a constraint, not proof of spend. Financial execution and spend tracking remain unavailable.

Research notes carry a trust classification. Untrusted web content can inform research but cannot authorize an action or satisfy an owner approval. KPI observations require a finite value, timestamp, source classification, and evidence reference. Revenue is reported only from an explicit `revenue_minor` observation and is always accompanied by its evidence source; it is never inferred from a budget, hypothesis, conversion, or model statement.

Evaluation is deterministic. Missing notes, KPI definitions, or observations yields `insufficient_evidence`; satisfied KPI targets yield `advance`; otherwise the recommendation is `hold`. Advancing a stage requires the most recent persisted evaluation to recommend advancement. Stopping is always explicit.

## Consequences

- Experiments can be compared without fabricated revenue or spend metrics.
- Stage changes have a durable evidence trail and cannot be authorized by model prose.
- Owner-entered and provider-reported observations are source-labelled but are not independently authenticated in this phase.
- Payment, advertising, purchasing, account mutation, and automated research remain unavailable.
