# Boundary contracts

`src/index.ts` is a runtime-free, dependency-free contract module. It freezes trust-boundary vocabulary; it is not an executor.

IDs are branded to avoid interchange. `ProposedAction` is untrusted and carries no approval. Trusted code normalizes `ActionRequest`. `PolicyDecision` is closed: deny, require owner approval, or permit with constraints. `ApprovalGrant` binds one digest and expiry. `ExecutionPermit` is opaque and policy-created. `ActionResult` preserves unknown, unavailable, denied, partial, failed, and successful outcomes. `MemoryRecord` carries provenance and never authority.

Required-field, semantic, or union changes are breaking and need an ADR, migration plan, and compatibility tests. Additive optional fields need compatibility tests.
