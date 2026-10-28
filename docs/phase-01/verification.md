# Phase 01 verification

This document records the expected gate and is updated only with observed command results.

## Completion gate

- [x] JavaScript dependencies install from lockfile.
- [x] TypeScript contracts and renderer compile strictly.
- [x] Contract, renderer, and Rust tests pass.
- [x] Vite production build succeeds.
- [x] Tauri release executable builds from locked dependencies.
- [x] Release executable opens a native window and exits with code 0 under smoke mode.
- [x] Normal interactive launch and owner-triggered exit are manually observed.
- [x] Persistent lifecycle log is located and inspected.

## Observed results

- `npm run verify`: passed; 5 Phase 00, 3 IPC, 2 renderer, and 4 Rust tests passed; strict TypeScript and production Vite build passed.
- `npm audit`: 0 vulnerabilities after upgrading Vite to 7.3.7 and Vitest to 4.1.12.
- `npm run build:windows`: passed; release executable created at `target/release/saiki.exe` using the committed Cargo lockfile.
- `npm run smoke:windows`: passed; native process exited with code 0 after 1,705 ms. WebView2 printed Windows error 1412 while unregistering a Chromium window class during smoke shutdown; it did not alter the exit code or omit lifecycle completion.
- Interactive Windows check: the native window visibly reported `Present`, `CORE READY`, `PLATFORM WINDOWS`, and version `0.1.0`. Activating `Exit cleanly` removed the window/process.
- Persistent log: `%LOCALAPPDATA%/ai.saiki.desktop/logs/saiki.log` recorded `shell.ready`, `renderer.ready`, `shell.exit_requested`, and `shell.exited_cleanly` for the interactive run. No credential values were present.
- `cargo fmt --all --check` and locked Clippy with warnings denied passed. The current Tauri CLI still emits a `STATIC_VCRUNTIME` deprecation warning even when the documented `WindowsAttributes::static_vc_runtime(true)` API is used; the build succeeds and the explicit setting is retained in `build.rs`.
- Final `saiki.exe`: 9,141,248 bytes; SHA-256 `E6690A55EA350C3D102F17AC88305D90980B2ACC4944D03627BFBDF6821710CA`; Authenticode status `NotSigned`. The checksum identifies this local build but does not establish publisher authenticity.

## Limitations

CI status is not inferred from a local run. Code signing, an installer, accessibility audit, multi-monitor behavior, overlay behavior, crash reporting, log rotation/retention, and automated visual regression are outside this phase.
