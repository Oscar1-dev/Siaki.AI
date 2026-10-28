# Phase 05 interface system

The workspace extends the existing restrained dark overlay language. It intentionally avoids dashboard metrics, decorative charts, and placeholder cards.

## Tokens and patterns

- Backgrounds: near-black shell, one raised panel level, one interactive panel level.
- Accent: violet only for focus, current navigation, monitor selection, and the SAIKI signal.
- Status: muted green for verified live state; muted rose for unavailable or destructive state.
- Typography: Segoe UI Variable, small uppercase metadata, sentence-case actions.
- Navigation: Activity, Settings, and Approvals use native buttons with keyboard focus and `aria-current`.
- Settings: labelled native checkboxes and buttons; descriptions state persistence and effect.
- Unavailable capability: reason, scope boundary, and absence of action controls.

All controls include default, hover/focus through native browser behavior, active selection, and error containment through the existing boundary. The overlay remains `aria-hidden` because it is native click-through; owner interaction stays in the controller window.
