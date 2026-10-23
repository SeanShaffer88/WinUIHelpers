# WinUI Helpers roadmap

## Next

- **Theme-change subscription helper**: a small message-only window
  that forwards WM_SETTINGCHANGE ("ImmersiveColorSet") to a caller
  callback, so apps can react when the user flips dark mode.
- **Snap-layout metrics / title-bar button sizing** for custom-drawn
  title bars (DWMWA_CAPTION_BUTTON_BOUNDS via GetWindowAttribute).
- **Toast notifications** — modern Windows toast XML over the COM
  notification manager, returning a tag the caller can dismiss by.

## Later

- Overlay icons and thumbnail toolbars (the rest of ITaskbarList3),
  jump-list construction (ICustomDestinationList).
- Drag-region helpers for borderless windows (hit-test math as a
  pure function so it's testable off-Windows).
- A `wh_apply_theme(hwnd, dark)` bundle call that sets title bar,
  caption, border, and text colors in one go with per-feature
  fallbacks reported as JSON.

## Non-goals

- Not a UI framework — no controls, no rendering. WinUI Helpers
  polishes windows other code owns.
