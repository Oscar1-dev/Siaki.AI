# Phase 21 — Account onboarding workflows

## Delivered

- Rust-owned SQLite onboarding/recovery workflows.
- Provider-specific templates for GitHub, OpenAI, Google, and Microsoft.
- Sequential owner-confirmed terms, ownership, contact, MFA, and recovery steps.
- Recovery checklist covering official recovery, recovery-channel control, manual credential rotation, and MFA revalidation.
- Append-only owner audit events for creation, confirmation, completion, and cancellation.
- Owner workspace with no credential or evidence-note input.

## Completion gate

`onboarding_is_manual_and_terms_cannot_be_skipped` verifies every step is non-automated, the first step is terms review, and later steps cannot be confirmed first. `ownership_is_append_only_auditable` verifies owner-attributed durable confirmation events. Unsupported providers return unavailable and false confirmation is rejected.

## Security and limitations

- No automated signup, terms acceptance, CAPTCHA handling, browser automation, OAuth, recovery, or credential rotation occurs.
- Workflow completion records owner confirmation; it does not claim provider-side verification.
- Workflows require an active Phase 20 identity entry.
- Templates must be reviewed when provider requirements change.
- Provider APIs and cryptographic ownership verification remain unavailable.

## Verification

- `npm run verify`
- `npm run smoke:windows`
- Manual: create a GitHub checklist, verify only the first step is actionable, confirm it, and inspect the owner audit event.
