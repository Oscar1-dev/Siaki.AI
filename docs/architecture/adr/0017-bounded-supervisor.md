# ADR 0017: Autonomous supervision is deterministic and capability-bounded

Status: Accepted for Phase 18.

## Decision

The autonomous supervisor is a Rust-owned durable state machine, not an LLM loop. Goals have owner-defined priority, task limit, and optional sleep time. A condition-variable wake signal reacts to goal creation and resume; a bounded timeout also recovers missed signals. Work selection is deterministic by priority, creation time, and identifier.

Phase 18 exposes exactly one autonomous capability: an internal read-only world-integrity audit. It cannot invoke shell, browser, files, accounts, communication, policy mutation, or any external side effect. Model output cannot create authority or change this capability.

Each task is durably leased before work. Completion stores factual evidence from the Phase 17 reconstruction. Expired leases return to pending after restart or a later wake. Quotas are fixed in the native core at three tasks per wake, twenty completed tasks per hour, and one hundred tasks per goal. Pause is checked before claiming work and prevents execution.

## Consequences

- The agent can complete useful bounded internal tasks without an owner prompt.
- Pause and quotas cannot be overridden through task text.
- A crash cannot silently mark work complete; leased work is recoverable.
- Phase 18 does not perform model planning, reflection, or external action. Those require later phases and existing policy gates.
