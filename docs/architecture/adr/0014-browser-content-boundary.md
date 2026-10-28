# ADR 0014: Browser content is observation, never authority

Status: Accepted for Phase 15.

## Decision

Browser automation runs only in a dedicated non-owner profile owned by a least-privilege sidecar. DOM text, accessibility text, links, attributes, downloads, and extracted data enter the core with `untrusted_web_content` provenance. They may inform planning but may not become an actor, capability grant, approval, credential request, or policy decision.

Every step carries a typed source and action. Page-originated sensitive click/fill proposals are denied before policy evaluation. Owner/planner proposals still pass through the Phase 12 policy engine. An allow decision is necessary but not sufficient: the executor must also have an authenticated, job-scoped sidecar and installed browser engine. Runtime absence produces `unavailable`, never simulated success.

Checkpoints store URL, bounded title, sequence, timestamp, and SHA-256 content digest. Raw page text is deliberately not persisted. Schemes other than HTTP/HTTPS are rejected at the task boundary.

## Consequences

- Prompt injection cannot authorize a sensitive browser action.
- Browser sessions cannot reuse the owner's everyday browser profile or cookies.
- A future sidecar must authenticate its protocol, receive only job-scoped data, enforce download/path confinement, and return verifiable evidence.
- Phase 15 does not bundle Node, Playwright, Chromium, or a browser executor, so navigation/click/fill/extract remain unavailable.
