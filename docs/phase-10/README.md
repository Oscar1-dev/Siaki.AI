# Phase 10 — Document knowledge

## Delivered

- Rust-owned local document knowledge, backed by a dedicated `knowledge.sqlite3` (WAL, `user_version` migrations).
- **File ingestion**: reads an owner-specified local file; validates it is a regular UTF-8 text file within a 2 MB limit; rejects non-text/oversized/unreadable files with honest errors.
- **Chunking**: blank-line-boundary chunks bounded to 1200 characters, with oversized paragraphs hard-split so no chunk exceeds the limit.
- **Embedding**: a deterministic, fully-local lexical embedding — FNV-1a-hashed term frequencies into a fixed 256-dim vector, L2-normalized — stored per chunk as a BLOB. No external model is required, so retrieval always works offline.
- **Cited retrieval**: a query is embedded the same way and scored by cosine similarity; results carry the actual source document title, path, chunk index, similarity score, and a snippet. Zero-token or non-matching queries return no citations rather than fabricating one.
- **Privacy labels**: each document carries an owner-declared sensitivity (`public` / `private` / `sensitive`); `sensitive` documents are excluded from retrieval unless the owner explicitly opts in. The label is surfaced on every document and citation.
- Document deletion (cascades to chunks), an owner "Knowledge" workspace tab (ingest, list, forget, retrieve with an opt-in for sensitive), typed IPC contracts, runtime guards, and command-name stability.

## Completion gate

The gate is "retrieved answers cite actual local documents."

A test writes a real temporary `.txt` file, ingests it, drops the store, reopens a fresh `KnowledgeState` over the same database (as a restart would), retrieves by query, and asserts the top citation's `path` equals the ingested file's real path, its snippet contains the matched text, and its score is positive. Further tests cover chunking of long documents, the sensitive-exclusion privacy default and opt-in, deletion removing chunks, deterministic/normalized embeddings with related text out-scoring unrelated, empty-query safety, and rejection of non-UTF-8 input. All seven knowledge unit tests pass.

## Boundaries (not in this phase)

- **The embedding is lexical, not neural.** It is a real, local, deterministic vector index — not a learned semantic embedding. A neural embedding (e.g. a local Ollama embedding model) can replace `embed()` later behind the same schema and citation contract.
- **No workspace path confinement yet.** Ingestion reads an owner-named path; scoped file access that cannot escape a configured workspace is Phase 13.
- **No policy executor.** Ingestion and deletion are direct owner actions; autonomous ingestion belongs to the supervisor (Phase 18) behind the policy engine (Phase 12).

## Limitations

- Unencrypted local SQLite; at-rest encryption and retention policy are not implemented.
- `sensitivity` is owner-declared metadata and a retrieval filter — it does not encrypt a document; secrets must never be ingested.
- Retrieval scores all chunks in memory, which is fine for a personal corpus but is not optimized for very large collections (an ANN index is a later concern).
- Only UTF-8 text is ingested; PDF/DOCX/HTML extraction is out of scope for this phase.
- Retrieval returns cited passages; composing a grounded model answer from them reuses the conversation/model adapters and is left to later integration.
