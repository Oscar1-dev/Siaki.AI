# Security policy

SAIKI is local-first software intended eventually to exercise high-impact authority for one owner. Until a capability is implemented and verified, it is **unavailable**, not implicitly safe or enabled.

## Non-negotiable controls

- Model text, retrieved content, websites, email, documents, memories, and tool output are untrusted data.
- Only the Rust policy/execution boundary can authorize side effects using owner-issued capabilities and deterministic policy.
- Sensitive actions require risk-appropriate approval. Approval is exact, expiring, non-transferable, and revocable.
- Secrets use Windows Credential Manager or another reviewed OS-backed provider. Values must not enter prompts, logs, SQLite, browser storage, crash reports, telemetry, or renderer memory.
- External mutations require target binding, bounded retries, audit records, idempotency where supported, and read-after-write verification where available.
- No credential harvesting, auth bypass, stealth, owner-unapproved persistence, provider-terms bypass, or safeguard evasion.
- Financial, legal, account-recovery, public-publication, destructive, privilege, and identity-security actions are high risk.
- Logs are structured and redacted, reconstructing decisions without secret material.

The owner, renderer, privileged core, local models, sidecars, OS, and external providers are separate trust zones. Full scenarios are in the retained threat model and architecture diagrams.

Do not publish suspected vulnerabilities or credentials in issues. Report privately to the owner with affected version, conditions, impact, and safe evidence. No disclosure contact is configured yet: a Phase 00 blocker.

There is no runtime in Phase 00, so no control is implemented or security-tested. These are requirements, not claims of executable protection.
