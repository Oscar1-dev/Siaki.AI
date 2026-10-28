# Phase 00 acceptance matrix

Date: 2026-10-28. Gate: no runtime implementation begins until required rows are evidenced and applicable blockers are resolved.

| ID | Requirement | Evidence | Check | Status |
| --- | --- | --- | --- | --- |
| P00-01 | Scope and phase boundary | `README.md`, `AGENTS.md` | No runtime scaffold | Pass |
| P00-02 | Stack and dependency direction | ADR-0001, architecture index | ADR test/manual review | Pass |
| P00-03 | Models cannot authorize effects | ADR-0002, typed contracts | Contract assertions | Pass |
| P00-04 | Persistence/memory semantics | ADR-0003, `MemoryRecord` | Strict compile | Pass |
| P00-05 | Identity/secret boundaries | ADR-0004, `SECURITY.md` | Manual backend selection | Pass with blocker |
| P00-06 | Execution and honest outcomes | ADR-0005, `ActionResult` | Union assertions | Pass |
| P00-07 | Local/cloud model boundary | ADR-0006 | Owner enables cloud | Pass |
| P00-08 | Audit, budgets, revocation | ADR-0007 | Owner approves UX | Pass with blocker |
| P00-09 | Trust/action diagrams | `diagrams.md` | Mermaid fence/manual render | Pass |
| P00-10 | Repository threat model | Retained `threatmodel.md` (SHA-256 `439f844e38f03fb7c336a6459d3e5726841d69d349d90f6277c8b316d5af9d84`) | Managed artifact receipt/manual review | Pass |
| P00-11 | Typed contracts compile | `contracts/src/index.ts` | `tsc` strict | Pass |
| P00-12 | Reproducible gate | tests and `package.json` | `npm test` | Pass |
| P00-13 | Existing work preserved | Initial commit retained | Git review | Pass |
| P00-14 | Limitations/blockers honest | architecture index/handoff | Section check | Pass |

## Manual checks before Phase 01

1. Owner reviews the seven ADRs and requests any superseding ADRs.
2. Owner reviews the threat model and confirms single-owner/local-first deployment.
3. Owner selects the exact Phase 01 scope; this freeze does not authorize scaffolding.
4. Any next-phase secrets, mutation, accounts, spending, or publication stay blocked until their decisions are resolved.

Phase 00 completes only when `npm test` passes, the retained threat model exists, Git diff is reviewed, diagrams are readable, and limitations are reported. Completion documents architecture; it does not claim runtime enforcement.
