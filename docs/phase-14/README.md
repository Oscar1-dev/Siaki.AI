# Phase 14 — Terminal and developer tools

## Delivered

- Rust-owned developer tools scoped to an owner-configured repository root (persisted in `devtools-config.json`).
- **Real repo analysis**: `git_status` (parsed porcelain-v1 with branch) and `git_diff` (optionally path-scoped, confined to the repo), by invoking the real `git`.
- **Bounded, policy-gated shell**: `run_command` runs only commands on the owner's allowlist, and — before spawning — calls the Phase 12 policy engine for `shell.exec`. Because `shell.exec` is deny-by-default (critical tier), a command is refused unless the owner has granted that capability. This is the first place the policy engine gates a real side effect.
- **Command cancellation and timeout**: each run registers a cancel token and spawns with `kill_on_drop`; a `select!` races the process against cancel and a bounded timeout, so cancelling drops the process future and kills the child. Outcomes are `succeeded` / `failed` / `denied` / `cancelled` / `unavailable` — never fabricated.
- **VS Code bridge**: owner-configured executable path; `open_in_vscode` opens a repo-confined path, and is unavailable until configured.
- **Codex activity adapter**: reported as an explicit `unavailable` capability — no supported local source is wired, and no activity is fabricated.
- Owner "Terminal & dev tools" workspace tab, typed IPC contracts, runtime guards, and command-name stability for all nine commands.

## Completion gate

The gate is "real repo analysis, command cancellation and approval checks."

- **Real repo analysis**: tested by the porcelain parser and by a real temporary git repository (init → commit → modify → status), which the module analyzes; the test skips cleanly only if `git` is absent.
- **Approval checks**: tested — with a fresh policy engine (no grants), `authorize_command` refuses `shell.exec` with "owner approval required", so the bounded shell will not run without an owner grant.
- **Command cancellation**: tested at the registry level — a registered request id is signalled, an unknown id is not; the runtime path kills the child via `kill_on_drop` when the cancel/timeout branch wins.

## Boundaries (not in this phase)

- Git status/diff are read-only local analysis and are not themselves policy-gated (consistent with Phase 13 file reads); only the execute-capable bounded shell goes through policy approval.
- No arbitrary shell string interpretation — commands are spawned directly (no shell), from an explicit allowlist, with separate arguments.
- The VS Code bridge opens a path; richer editor integration (LSP, live state) is out of scope.
- The Codex adapter is a typed boundary only; wiring a real activity source is deferred until a supported one exists.

## Limitations

- Cancellation tokens and the active-command registry are process-local.
- `git` must be on PATH for repo analysis; its absence is reported as an error, not faked.
- Command output is captured up to 64 KB and flagged truncated beyond that.
- The devtools config is unencrypted local JSON.
