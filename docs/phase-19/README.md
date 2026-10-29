# Phase 19 — Goal planning and reflection

## Delivered

- Durable SQLite goals, versioned plans, ordered tasks, and evidence reviews.
- Explicit plan rationale and per-task selection reason.
- Autonomous read-only world-integrity task execution.
- Structured evidence plus SHA-256 evidence binding.
- Evidence-based `stop_success`, `pivot`, and `stop_blocked` verdicts.
- Version-preserving pivot from audit to diagnostic inventory.
- Owner stop control and a workspace showing reasons and evidence.

## Completion gate

`task_explains_choice_and_shows_evidence` creates a goal, runs its task, and verifies that the completed record contains a lowest-risk selection reason, structured world-consistency evidence, and a successful stop review. Contract and UI tests ensure the explanation and evidence are exposed to the owner.

## Security and limitations

- Planning capability is fixed to `internal_world_review_only`.
- Text cannot choose tools or grant authority.
- No shell, browser, files, network, identity, communications, or financial actions can be planned or executed.
- The planner currently uses a deterministic world-integrity template; model-generated decomposition is not implemented.
- Evidence proves what the internal reconstruction reported, not external-world truth.

## Verification

- `npm run verify`
- `npm run smoke:windows`
- Manual: create a coherence goal and confirm rationale, task-choice explanation, evidence, digest, and stop verdict appear.
