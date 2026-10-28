# ADR 0012: Loopback-only local model adapter

Status: Accepted for Phase 06

## Decision

The privileged Rust core owns Ollama discovery and chat transport. Phase 06 uses only `http://127.0.0.1:11434`; the renderer cannot select another host or attach credentials. Discovery uses `GET /api/tags`. Chat uses streamed `POST /api/chat` responses, bounded request data, an owner-visible timeout, and per-request cancellation.

The renderer creates a UUID before invoking chat so its event listener can be active before the first token. Rust rejects duplicate or malformed IDs. Stream events have explicit `succeeded`, `cancelled`, `failed`, or `unavailable` outcomes.

Ollama function calls cross IPC only as `untrusted_model_output` tool proposals. This adapter has no tool executor and cannot grant authority or cause side effects. There is no automatic cloud fallback; timeout and local-service failure remain truthful unavailable states.

## Security consequences

- Fixed loopback transport prevents model requests from becoming an arbitrary HTTP client or silently leaving the PC.
- Prompts and generated text are not written to native logs.
- Request size, tool count, tool schema shape, timeout, and model identifier are bounded.
- Cancellation drops the HTTP stream but cannot guarantee immediate compute reclamation inside Ollama.
- Any future remote host, authentication, conversation persistence, or tool execution requires a separate ADR and phase contract.
