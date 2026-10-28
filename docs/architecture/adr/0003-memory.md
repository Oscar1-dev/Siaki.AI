# ADR-0003: SQLite and provenance-bearing memory

- Status: Accepted
- Date: 2026-10-28

## Context
Persistent learning requires durable state, while memories may be false, malicious, sensitive, stale, or revoked.

## Decision
Use SQLite behind Rust repositories and migrations. Memory carries provenance, timestamps, sensitivity, confidence, retention, and lifecycle. Separate memory categories logically. Retrieval grants no authority. Owner export, correction, and deletion precede long-term ingestion.

## Consequences
Renderer and sidecars never access the database directly. Schema changes require migrations and recovery consideration. Vector search is an adapter, not truth.
