# Phase 18 — Autonomous supervisor

## Delivered

- Rust-owned SQLite goals and task ledger.
- Event-driven condition-variable wake on goal creation and resume, with bounded fallback wake.
- Deterministic priority ordering and optional sleep-until times.
- Fixed native quotas: 3 tasks per wake, 20 per hour, 100 per goal.
- Durable task leases, retry attempts, and expired-lease recovery at restart and wake.
- Persistent global pause/resume that prevents task claims.
- Autonomous read-only world-integrity audits with factual reconstruction evidence.
- Owner supervisor workspace showing goals, tasks, quotas, capability, and pause state.

## Completion gate

`completes_bounded_task_without_prompt` creates a one-task goal and invokes the native wake loop without further owner input; the task and goal complete with world-integrity evidence. `pause_prevents_execution` verifies that a due goal remains untouched while paused. Restart recovery and deterministic priority are separately tested.

## Security and limitations

- The only autonomous capability is `internal_world_integrity_audit_only`.
- Goal text is descriptive data and grants no tool or policy authority.
- The supervisor cannot call shell, browser, files, network, accounts, communications, or policy mutation.
- Phase 19 planning/reflection and broader policy-gated task execution are not implemented.
- The fallback wake is process-local; powered-off execution is unavailable.

## Verification

- `npm run verify`
- `npm run smoke:windows`
- Manual: create a one-task audit goal, observe autonomous completion, pause, create another goal, verify it remains at zero, then resume.
