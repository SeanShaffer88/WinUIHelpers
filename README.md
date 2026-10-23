# WinUI Helpers

The window polish every Windows app re-implements — dark title
bars, Mica/Acrylic backdrops, rounded corners, caption colors, DPI
helpers, taskbar progress — behind one clean C ABI, in pure Rust
with zero dependencies and hand-declared Win32/COM bindings.
Library #7 in the Windows DLL Collection.

Every capability is checked against the running Windows build:
features the OS doesn't have return `WH_ERR_NOT_SUPPORTED` with an
explanation instead of silently doing nothing.

## ABI (`include/winuihelpers.h`, prefix `wh_`)

Window handles pass as `uint64_t` (HWNDs are pointer-sized).

| Function | Purpose |
|---|---|
| `wh_windows_info_json()` | `{"build":…,"windows11":…,"dark_apps":…,"dark_system":…,"accent":"#0078D4","accent_argb":…}` |
| `wh_apps_dark_mode()` / `wh_accent_color()` | Theme queries: app dark mode on/off; the DWM accent color as ARGB. |
| `wh_dark_titlebar(hwnd, on)` | Immersive dark title bar; attribute 20 with 19 as legacy fallback. |
| `wh_set_backdrop(hwnd, kind)` | 0 auto, 1 none, 2 **Mica**, 3 **Acrylic**, 4 Mica Alt (Windows 11). |
| `wh_set_corners(hwnd, pref)` | 0 default, 1 square, 2 round, 3 round-small (Windows 11). |
| `wh_set_caption_color` / `wh_set_border_color` / `wh_set_caption_text_color` | Frame colors by `0xRRGGBB` (Windows 11). |
| `wh_get_dpi(hwnd)` / `wh_scale_px(v, dpi)` / `wh_set_process_dpi_aware()` | Per-window DPI, 96-DPI scaling math, Per-Monitor-V2 opt-in for the calling process. |
| `wh_window_info_json(hwnd)` | `{"title":…,"width":…,"height":…,"dpi":…}` |
| `wh_center_on_screen(hwnd)`, `wh_set_always_on_top(hwnd, on)`, `wh_set_title(hwnd, s)` | Geometry and title basics. |
| `wh_taskbar_progress(hwnd, done, total)`, `wh_taskbar_state(hwnd, state)`, `wh_taskbar_clear(hwnd)` | `ITaskbarList3` progress: determinate values plus indeterminate/normal/error/paused states. |
| `wh_version*`, `wh_last_error*`, `wh_free_string` | Version info, thread-local errors, disposal. |

Invalid window handles are `WH_ERR_INVALID_ARG` (checked with
`IsWindow`), COM and DWM failures carry their HRESULTs in the error
message, and the shared 0–6 error codes apply throughout.

## How it's put together

- **DWM**: `DwmSetWindowAttribute` for the frame features, gated on
  the real build number from `RtlGetVersion` (never the manifest-
  dependent `GetVersion`).
- **Theme state**: read from the per-user Personalize registry keys
  plus `DwmGetColorizationColor` for the accent.
- **Taskbar**: `ITaskbarList3` through a hand-rolled COM vtable —
  the CLSID/IID and the method indices (HrInit 3, SetProgressValue
  9, SetProgressState 10) are transcribed from the shell headers;
  COM initialization is tolerant of a thread already initialized in
  another apartment model.

## Verification

- Unit tests cover the platform-free logic (capability math, DPI
  scaling, color formatting) and, off-Windows, that every OS call is
  a precise `NOT_SUPPORTED` rather than a stub success.
- **CI (`tests/roundtrip.rs`, Windows)**: creates a *real* window in
  the test process and runs the surface against it — window info
  reads its actual title and size, DPI is sane, the dark title-bar
  call succeeds (or reports NOT_SUPPORTED on a genuinely old build),
  centering and topmost toggling succeed, and the full taskbar
  state machine (normal → progress value → paused → cleared)
  executes through the hand-rolled COM vtable.
- Gate (WinDevKit `tools/gate.sh`): fmt, clippy on host **and**
  `x86_64-pc-windows-msvc` with `-D warnings`, full tests — green.
  Every extern block carries its `#[link(name = "...")]`; CI's real
  Windows link is the final check, as with all Win32-backed
  libraries in the collection.

Visual results (actual Mica rendering, caption colors) can only be
judged on a desktop — the CI proof establishes that every call is
accepted by the real APIs; what it looks like is for the caller to
see on Windows 11.

## License

MIT © 2026 Sean Shaffer.
