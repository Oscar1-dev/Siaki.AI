# Phase 07 — Optional cloud provider

## Delivered

- Disabled-by-default OpenAI Responses API adapter owned by Rust.
- Windows Credential Manager storage under the dedicated `SAIKI.OpenAI` service.
- Renderer-safe environment import: the secret never crosses IPC or enters React state.
- Persistent non-secret provider configuration stored separately from the credential.
- Explicit destination, data, storage, subscription, and billing disclosure.
- Owner-selected model identifier with no account-entitlement assumptions.
- SSE streaming, cancellation, bounded timeout, and truthful provider error states.
- Requests use `store: false`; model output remains non-authoritative.

## Completion gate

The cloud provider must be visibly disabled without a credential. Enablement must fail unless the credential, model, and current disclosure acceptance are all present. Disabling must immediately prevent new cloud requests. Deleting the credential must also disable the provider. Automated tests cover contracts, disabled defaults, no-storage requests, disclosure fields, and existing regressions.

Live OpenAI inference is account-dependent and is not required to prove the enable/disable gate. If the owner does not provision a credential with API access and billing, the UI must remain honestly unavailable rather than implying that a ChatGPT subscription provides API access.

## Credential setup

1. Set `OPENAI_API_KEY` only in the environment of a one-time SAIKI launch.
2. Open Models → OpenAI API and choose **Import OPENAI_API_KEY**.
3. Exit and relaunch SAIKI without the environment variable.
4. Enter a model identifier available to the owner’s API project, read and accept the disclosure, then enable the provider.

The key can be removed from the same view. SAIKI never displays it.

## Limitations

- No OpenAI credential is bundled or created by SAIKI.
- No model catalog or billing entitlement is inferred.
- Import cannot safely mutate the multithreaded process environment; restart without the variable after import.
- Phase 08 conversation persistence and persona behavior are not included.
