# ADR-0008: Split visual overlay from interactive hit region

- Status: Accepted
- Date: 2026-10-28

## Context
An overlay that changes click-through state using its own pointer events can trap input: once ignored, it cannot reliably observe the pointer to become interactive again.

## Decision
Use two native windows. The monitor-sized transparent `overlay` window is always click-through. The compact `main` controller is the explicit interactive hit region. Both move using physical monitor coordinates and DPI metadata. `Ctrl+Shift+F12` is a native global pause/resume escape path that hides or restores both windows without model involvement.

## Consequences
Visual content cannot receive clicks. Future character interaction must use separately bounded hit-region windows or a reviewed native hit-test implementation. The controller remains owner-operated and must never expose arbitrary commands.
