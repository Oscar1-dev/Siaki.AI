# Phase 01 — Monorepo and Windows shell

Status: implementation complete; verification evidence is recorded in `verification.md`.

## Delivered boundary

- npm workspaces for the desktop app and IPC contract package, plus a Rust workspace.
- Tauri 2 Windows shell with React 19, TypeScript, and Vite.
- Three typed IPC calls: read shell status, write bounded/redacted frontend logs, and request graceful owner exit.
- Strict renderer contract validation, React error boundary, native file logging, log-forging protection, and sensitive-context redaction.
- Deterministic test/build scripts, locked JavaScript/Rust dependencies, Windows CI, and a native launch/clean-exit smoke mode.

## Explicitly unavailable

No overlay, transparent/click-through window, global hotkey, monitor/DPI handling, PixiJS rig, memory database, model, browser, sidecar, autonomous planner, tool executor, accounts, credentials, or external network capability is implemented. Those belong to later approved phases.

## IPC security

The renderer has only the Tauri core defaults and the three registered commands. Native deserialization rejects unknown fields for mutation requests. Frontend logs are length-bounded, newline-normalized, context-count limited, and redact keys suggestive of secrets. The exit command changes only this application process and cannot execute arbitrary commands.

## Development

```powershell
npm ci
npm run verify
npm run tauri:dev --workspace @saiki/desktop
```

## Reproducible native build and smoke

```powershell
cargo test --workspace --locked
npm run build:windows
npm run smoke:windows
```

The native build uses `package-lock.json` and `Cargo.lock`. `build:windows` intentionally uses Tauri `--no-bundle` for the Phase 01 gate; installer packaging/signing is not claimed.
