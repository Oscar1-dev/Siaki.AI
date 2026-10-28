# ADR 0013: Owner-enabled OpenAI provider and credential boundary

Status: Accepted for Phase 07

## Decision

OpenAI is an optional provider owned by the privileged Rust core. It is disabled by default and can be enabled only when all three conditions hold: an API credential exists in Windows Credential Manager, the owner supplies a model identifier available to their API project, and the owner accepts the current data and billing disclosure.

SAIKI calls only `https://api.openai.com/v1/responses` using bearer authentication and streamed server-sent events. Requests set `store: false`. A ChatGPT subscription is never treated as API access, credit, or authorization. The API model is owner-selected because availability and cost vary by account and project.

The renderer cannot submit or retrieve the API key. Rust may import `OPENAI_API_KEY` directly from its process environment into Windows Credential Manager; only credential-presence status crosses IPC. The owner must restart SAIKI without that environment variable after import. Deleting the credential also disables the provider.

## Consequences

- Prompts leave the PC only after explicit provider enablement and a visible per-send action.
- The disclosure names the destination, transmitted fields, storage request, and possible API billing.
- Provider configuration contains only enabled state and model identifier; it is stored separately from the credential.
- API errors distinguish authentication/access, rate or account budget limits, timeout, cancellation, and other failures without exposing provider response bodies.
- Model output remains untrusted text and has no side-effect authority.
- Future OAuth, project selection, model discovery, or tool calling requires a new contract and review.
