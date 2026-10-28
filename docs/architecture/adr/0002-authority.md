# ADR-0002: Separate cognition from authority

- Status: Accepted
- Date: 2026-10-28

## Context
Models can be mistaken or manipulated; planning cannot be authorization.

## Decision
Models emit typed proposals only. Deterministic policy evaluates normalized requests. Execution needs an opaque, expiring, one-use permit. Model-authored fields cannot represent approval. High-risk actions need owner approval bound to the exact request digest and immediate re-evaluation.

## Consequences
Prompt instructions cannot expand authority. Types distinguish proposals, requests, approvals, decisions, permits, and results.
