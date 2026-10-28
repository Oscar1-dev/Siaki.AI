# ADR 0016: World state is reconstructed from durable events

Status: Accepted for Phase 17.

## Decision

The Rust core owns a separate SQLite world database. Every entity mutation appends an immutable, globally ordered event and writes a versioned snapshot plus current projection in one immediate transaction. Project commands use optimistic concurrency through an exact `expectedVersion`; stale or invalid lifecycle transitions are rejected.

The durable event stream is the reconstruction source. On read, the core folds ordered events into project entities and compares the result with the materialized projection. Inconsistency is reported explicitly rather than silently trusting either representation. Successfully committed events are also emitted on the process-local typed event bus for live consumers.

Event actors are assigned by the core and cannot be supplied through request payloads. Phase 17 records owner project lifecycle facts only; events do not grant capabilities or authorize external effects.

## Consequences

- State can be reconstructed after restart without relying on renderer state.
- Entity versions prevent lost updates and stale autonomous plans.
- Snapshots preserve auditable version history while events remain append-only.
- Event payloads may contain private project text and must not be copied into prompts or logs without later sensitivity policy.
- Event compaction, redaction/export, generic entity schemas, and cross-database transactions remain future work.
