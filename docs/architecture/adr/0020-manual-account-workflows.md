# ADR 0020: Account onboarding and recovery remain manual owner workflows

Status: Accepted for Phase 21.

## Decision

Provider onboarding and recovery are durable checklists bound to an active Phase 20 identity. GitHub, OpenAI, Google, and Microsoft have explicit templates. Every step is marked `automated: false` and `requiresOwnerConfirmation: true`; steps must be confirmed sequentially by the owner.

The first onboarding step requires manual review of current provider terms. SAIKI does not open signup forms, accept terms, solve CAPTCHAs, create accounts, handle recovery channels, rotate credentials, or simulate provider success. Unsupported providers return unavailable.

Workflow creation, each owner confirmation, completion, and cancellation append an audit event whose actor is assigned by the Rust core. No notes or proof-material fields exist, preventing credentials or recovery codes from being pasted into workflow storage.

## Consequences

- Account ownership actions are durably auditable.
- Terms review cannot be skipped by completing later steps first.
- Workflow completion means the owner confirmed checklist steps, not independent provider verification.
- Live provider APIs, OAuth, signup, recovery automation, and ToS acceptance remain unavailable.
