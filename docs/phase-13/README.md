# Phase 13 — Windows and file tools

## Delivered

- Rust-owned system and file tools confined to a single owner-configured workspace root, persisted in `systools-config.json`.
- **Workspace confinement** (the security boundary): every file path passes through one `confine(root, rel)` chokepoint that rejects absolute paths, drive/root prefixes, and any `..` component, then canonicalizes the deepest existing ancestor of the target and requires it to stay within the canonical root — so symlink escapes are caught too.
- **Scoped file operations**: list a workspace directory, read a UTF-8 text file (≤ 1 MB), and write a file (≤ 1 MB). Every write captures the prior bytes and returns a **rollback token**; `rollback_write` restores the previous content, or removes the file if it was newly created — a real, reversible undo.
- **System metrics**: truthful, dependency-free — logical CPU count, OS, architecture, and pointer width.
- **Allowlist-only app launching**: the owner registers applications (name + existing executable path); `launch_app` runs only an allowlisted entry, and anything else is refused.
- Owner "System & files" workspace tab: configure the root, browse/read/edit/save files with rollback, view metrics, and manage the app allowlist.
- Typed IPC contracts, runtime guards, and command-name stability for all nine commands.

## Completion gate

The gate is "no operation escapes configured workspace."

Tests prove it against a real temporary workspace:
- `confine` rejects `../outside`, `sub/../../outside`, a Windows absolute path (`C:/...`), and a POSIX absolute path (`/etc/...`), while accepting a normal relative path.
- A write targeting `../escape.txt` is refused.
- File operations fail cleanly when no workspace root is configured.
- Write-then-rollback restores prior content; rollback of a newly created file removes it.
- Configuration (root + allowlist) persists across restart; metrics are truthful.

## Boundaries (not in this phase)

- **Policy-executor wiring is deferred.** This module's own boundary is workspace confinement and the launch allowlist; routing these operations through the Phase 12 policy engine (so each write/launch is a policy-evaluated, audited action) is the executor phase's job.
- No arbitrary shell execution; launching is restricted to the explicit allowlist.
- Binary files are not read (UTF-8 text only); large-file streaming is out of scope.

## Limitations

- Rollback tokens are process-local (an in-session undo), not persisted across restart.
- System metrics are std-only; CPU load, memory pressure, GPU/VRAM, and the performance benchmarks the blueprint calls for are a later concern.
- The workspace config is unencrypted local JSON.
- `launch_app` spawns the process but does not track or supervise its lifetime.
