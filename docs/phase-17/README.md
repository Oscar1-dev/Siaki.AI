# Phase 17 — World database and event bus

## Delivered

- Rust-owned SQLite world database with versioned migrations.
- Durable projects as world entities with planned, active, paused, completed, and cancelled lifecycle states.
- Append-only globally sequenced events and per-version snapshots.
- Transactional event, snapshot, and current-projection writes.
- Optimistic concurrency using exact expected entity versions.
- Reconstruction by replaying durable events and verifying the materialized projection.
- Typed process-local event emission after successful commits.
- Owner world workspace with lifecycle controls and durable event history.

## Completion gate

`durable_world_reconstructs_after_restart` creates and transitions a project, closes the database, reopens it, replays the event stream, and verifies an identical consistent entity at version 2. Tests also cover stale-version rejection, invalid lifecycle transitions, and stable global event sequencing.

## Security and limitations

- Request payloads cannot set event actors; the Rust core records `owner`.
- World events describe state and grant no authority.
- Project text is persisted locally but does not yet carry field-level sensitivity or retention labels.
- Project deletion/redaction, event compaction, and generic non-project entity mutation are not implemented.
- The live bus is process-local; durable consumers resume using global sequence numbers.

## Verification

- `npm run verify`
- `npm run smoke:windows`
- Manual: create a planned project, transition it to active, restart, and confirm version, lifecycle, event count, and reconstruction consistency.
