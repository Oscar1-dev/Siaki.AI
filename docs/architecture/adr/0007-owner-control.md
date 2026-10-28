# ADR-0007: Auditability and owner control

- Status: Accepted
- Date: 2026-10-28

## Context
Long-running autonomy needs accountability, bounded resources, interruption, and recovery outside model self-policing.

## Decision
Record structured, redacted decisions and executions with correlation IDs. Enforce time, rate, cost, and scope budgets outside models. Provide privileged pause, revoke, inspect, export, and emergency stop. Assets and earnings remain owner-controlled.

## Consequences
Audit retention/tamper evidence need later design. Emergency stop revokes pending permits and blocks new execution, not just UI state.
