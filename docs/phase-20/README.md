# Phase 20 — Digital identity registry

## Delivered

- Rust-owned SQLite account inventory.
- Provider, owner label, login hint, active/archive status, and timestamps.
- Owner-proof method and verification timestamp metadata.
- Core-generated opaque Windows Credential Manager target references.
- MFA enabled state, method classifications, and recovery-control confirmation.
- Strict typed requests that reject unknown credential-shaped fields.
- Owner identity workspace with no password, token, cookie, code, or secret input.

## Completion gate

Native tests prove credential-shaped registration fields are rejected and serialized registry output contains no password, token, or recovery-code material. Typed contracts require `secretInputSupported: false`, `credentialPresent: false`, `vaultBinding: reference_only`, and `credentialProvisioning: unavailable`. The identity module has no integration with logging, memory, conversation, or model adapters.

## Security and limitations

- Ownership proof is recorded metadata, not cryptographic verification.
- Login hints are private owner metadata stored in the local SQLite database; database encryption is not yet implemented.
- Vault references are inert targets. No credentials are created, read, or attached.
- Provider onboarding, recovery workflows, OAuth, and account actions are future phases.
- Archiving removes an identity from active use but preserves audit metadata.

## Verification

- `npm run verify`
- `npm run smoke:windows`
- `rg` confirms identity state is absent from conversation, memory, Ollama, cloud, and logging modules.
- Manual: open Identities, confirm no secret field exists, register metadata, and verify credential provisioning remains unavailable.
