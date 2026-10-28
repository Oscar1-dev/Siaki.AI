# Phase 16 — Reminders and scheduler

## Delivered

- Rust-owned SQLite schedules and notification history.
- One-shot and fixed-interval schedules with cancellation.
- Native-core one-second wake trigger while SAIKI is running.
- Transactional occurrence creation and schedule advancement.
- Unique occurrence keys preventing duplicate creation after restart.
- Durable pending delivery, renderer acknowledgement, and bounded exponential retries.
- Owner workspace for creating, inspecting, and cancelling schedules.
- In-app reminder notification visible from every owner workspace tab.

## Completion gate

`restart_does_not_duplicate_occurrence` creates and fires a schedule, closes the SQLite connection, reopens the same database, runs another wake, and verifies exactly one notification, one occurrence, and a completed task. Repeated-wake idempotency is independently tested.

## Limitations

- The native wake loop runs only while SAIKI is running. It does not register a Windows Task Scheduler job or wake a sleeping/powered-off computer.
- Notifications are in-app; Windows toast registration is not implemented.
- Fixed intervals are supported. Time-zone/calendar recurrence and daylight-saving policies are not yet modeled.
- Notification acknowledgement confirms renderer receipt, not that the owner read it.

## Verification

- `npm run verify`
- `npm run smoke:windows`
- Manual: schedule an immediate reminder, observe the in-app notification and acknowledged history, restart, and confirm it is not emitted again.
