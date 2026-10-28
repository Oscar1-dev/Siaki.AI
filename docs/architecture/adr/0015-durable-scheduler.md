# ADR 0015: Durable scheduler occurrences and delivery

Status: Accepted for Phase 16.

## Decision

The Rust core owns schedule time, persistence, occurrence identity, wake evaluation, and notification delivery state. SQLite records each occurrence with a unique `(task_id, occurrence_at)` key. Creating a notification and advancing or completing its schedule happen in one immediate transaction.

Notifications are durable inbox records. The native one-second wake loop emits pending records to the owner renderer. Delivery remains pending until explicitly acknowledged; missed delivery is retried using bounded exponential backoff for at most eight attempts. Restart reopens the same database and resumes pending work without recreating an occurrence.

Repeating schedules have a minimum 60-second interval and skip missed bursts rather than flooding the owner. At most 100 active schedules are allowed. Phase 16 notifications are in-app owner notifications; Windows toast registration and waking a suspended or powered-off machine are not claimed.

## Consequences

- Restart cannot duplicate an already-created occurrence.
- A renderer outage does not lose a notification.
- Clock changes can alter wall-clock scheduling; monotonic/zone-aware calendar recurrence remains future work.
- Schedule records carry owner-authored text only and confer no tool authority.
