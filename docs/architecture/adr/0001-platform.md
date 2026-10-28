# ADR-0001: Platform and process boundaries

- Status: Accepted
- Date: 2026-10-28

## Context
SAIKI needs a native Windows security boundary, rich animation, and replaceable integrations without ambient authority.

## Decision
Use Tauri 2 with a Rust privileged core, React/TypeScript UI, and PixiJS character scene. Keep renderer IPC allowlisted and typed. Run dependency-heavy integrations as isolated sidecars with job-scoped capabilities and structured IPC. Windows is primary.

## Consequences
Rust owns policy, persistence, process creation, and native integration. The renderer has no arbitrary shell or secrets. Sidecars cannot directly open the main database or secret store.
