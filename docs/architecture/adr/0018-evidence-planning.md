# ADR 0018: Plans are bounded hypotheses reviewed against evidence

Status: Accepted for Phase 19.

## Decision

Goals, plans, tasks, and reviews are durable Rust-owned records. A plan states its rationale; every task states why it was selected. Completion requires structured evidence from a trusted internal source and stores a SHA-256 digest binding the review to that evidence.

Phase 19 planning is deterministic and limited to read-only world review. A consistent reconstruction produces `stop_success`. Contradictory evidence supersedes plan version 1 and creates a bounded diagnostic plan version 2. Continued contradiction produces `stop_blocked` and requires owner review. The owner may stop an active goal before execution.

Goal text, rationale, and task descriptions are data, not capability. They cannot select arbitrary tools or authorize effects. Model-generated planning and broader policy-gated actions remain outside this phase.

## Consequences

- SAIKI can answer why a task was chosen and show its evidence.
- Reviews can stop or pivot instead of looping indefinitely.
- Plan history is preserved rather than overwritten.
- The current planner has one safe template and is not a general-purpose autonomous planner.
