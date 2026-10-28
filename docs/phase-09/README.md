# Phase 09 — Memory foundation

## Delivered

- SQLite-backed semantic memory, owned by the Rust core (`rusqlite`, bundled SQLite with FTS5). A dedicated `memory.sqlite3` in the app config directory, opened in WAL mode.
- Versioned migrations keyed on `PRAGMA user_version`; migration v1 creates the claim table, status index, the FTS5 index, and an append-only event log. Re-running migrations is idempotent.
- Memory claims with full provenance: `source`, `confidence` (0.0–1.0), `sensitivity` (`public` / `private` / `sensitive`), created/updated timestamps, `status` (`active` / `superseded` / `forgotten`), and a `supersedes` correction link.
- Three owner operations that persist across restart:
  - **teach** — record a new active claim, indexed for search.
  - **correct** — supersede an active claim with a new one; the prior row is retained as `superseded` (not deleted) and the new claim links back through `supersedes`, so history is never silently overwritten.
  - **forget** — hard-delete a claim's content, honoring the owner's deletion right; an append-only `forget` event records that a claim was removed without retaining what it said.
- FTS5 keyword retrieval with input sanitized into quoted terms so arbitrary owner text cannot raise an FTS query syntax error.
- Full export of claims and the audit event log as JSON.
- Owner workspace "Memory" tab: teach/correct/forget, search, provenance display, and export.
- Typed IPC contracts, runtime guards, and command-name stability for all six commands.

## Completion gate

The gate is "teach/correct/forget works across restart."

- **teach + restart**: a test teaches a claim, drops the state, reopens a fresh `MemoryState` over the same file, and retrieves the claim by FTS search — the way a restart would.
- **correct**: a test confirms the old claim is no longer retrievable, the new claim is, the `supersedes` link is set, and the superseded row plus a `correct` event remain in the export.
- **forget + restart**: a test forgets a claim, reopens the store, and confirms the content is gone from search and from the claim table and stays gone, while a `forget` audit event (without the content) remains.

All seven memory unit tests pass, alongside migration idempotency, FTS operator-character tolerance, and claim validation.

## Boundaries (not in this phase)

- **No embeddings / vector retrieval.** Retrieval here is FTS5 keyword only; local embeddings and document chunking are Phase 10.
- **Conversation storage is unchanged.** Phase 08's JSON conversation store is intentionally left in place; unifying it onto SQLite is a later consolidation, not required by this gate.
- **No policy executor.** Memory operations are direct owner actions in the workspace; autonomous memory writes belong to the supervisor (Phase 18) behind the policy engine (Phase 12).

## Limitations

- The database is unencrypted local SQLite; at-rest encryption and a retention/scheduled-deletion policy are not implemented yet.
- `sensitivity` is owner-declared metadata; it does not itself encrypt or gate a claim, and the UI warns against storing secrets. Credential/secret material must never be taught here.
- Export is returned to the renderer as JSON for inspection; writing it to a chosen file path (a save dialog) is not wired in this phase.
- FTS ranking uses the default FTS5 `rank`; relevance tuning (e.g. `bm25` weighting) is deferred.
