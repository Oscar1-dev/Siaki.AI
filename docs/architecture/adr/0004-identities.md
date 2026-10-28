# ADR-0004: Owner-controlled identities and secrets

- Status: Accepted
- Date: 2026-10-28

## Context
Dedicated email, GitHub, business, and provider identities create takeover and ownership risk.

## Decision
The owner creates or explicitly adopts identities and controls recovery/revocation. SAIKI stores opaque secret references only. Secret values remain in a reviewed OS store or provider OAuth flow and never reach models, prompts, memory, renderer, logs, or tool results.

## Consequences
Identity setup is an owner ceremony. Autonomous account creation is unavailable unless provider terms permit it and the owner approves terms and recovery. The concrete backend needs a later ADR.
