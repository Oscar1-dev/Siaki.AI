# Phase 02 — Overlay feasibility

Status: complete on the current Windows machine.

## Delivered

- Transparent, borderless, always-on-top visual overlay.
- Native full-surface click-through using Tauri `set_ignore_cursor_events(true)`.
- Separate compact controller as the only explicit hit region.
- Real monitor enumeration with physical position, size, and scale factor.
- Monitor switching that moves/resizes both windows, including negative desktop coordinates.
- Global `Ctrl+Shift+F12` pause/resume that hides/restores both windows.
- Typed IPC and validation for overlay status, monitor selection, and pause state.

## Verification

- `npm run verify`: passed; 5 Phase 00 tests, 4 IPC tests, 2 renderer tests, and 5 Rust tests.
- `npm run build:windows`: passed.
- `npm run smoke:windows`: native two-window process exited with code 0 after 1,729 ms.
- Windows reported three monitors: two 2560×1440 displays and one 1080×1920 portrait display, all at reported scale factor 1.00.
- Native visual inspection confirmed the transparent overlay over `DISPLAY1` and `DISPLAY2`; the controller followed and active selection updated.
- The overlay showed underlying applications while remaining non-interactive. Only the controller window exposed controls.
- Global hotkey hid both windows and restored them while Spotify—not SAIKI—was foreground.

## Limitations and blockers

- This is a feasibility implementation, not the character renderer.
- Only one visual overlay is active at a time; simultaneous per-monitor overlays are not implemented.
- DPI change events while dragging/reconfiguring displays are not yet subscribed; topology is refreshed on each status or move command.
- The controller is a rectangular hit region. Arbitrary character-shaped hit testing is deferred and must preserve the escape path.
- The Windows executable remains unsigned. WebView2 still emits error 1412 during automated shutdown without affecting exit status.
- Pause state is process-local and intentionally not persisted.
