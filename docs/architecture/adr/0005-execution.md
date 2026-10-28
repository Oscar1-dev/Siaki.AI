# ADR-0005: Policy-gated execution and truthful outcomes

- Status: Accepted
- Date: 2026-10-28

## Context
Tools can mutate accounts, publish, spend, delete, or misreport success.

## Decision
Adapters implement one request/result contract. The executor validates permit, target/payload digests, constraints, deadline, idempotency, and cancellation. It audits redacted events and verifies writes where available. Results explicitly include partial, unknown, and unavailable states.

## Consequences
Models get no arbitrary command tool. Every adapter documents terms, verification, retry, budget, and failure classification before enablement.
