# WinUI Helpers — agent notes

- Every extern block MUST carry #[link(name = "...")] (dwmapi /
  user32 / advapi32 / ole32 / ntdll). Clippy and check cannot catch
  a missing one; CI's Windows link can.
- Capability gating uses RtlGetVersion's build number, never
  GetVersionEx (manifest lies). Backdrop/corners/colors at >= 22000;
  dark title bar tries attribute 20 then the legacy 19.
- COLORREF byte order (0x00BBGGRR) is handled in exactly one place —
  rgb_to_colorref. Public API colors are 0xRRGGBB.
- COM discipline: with_taskbar owns init/uninit; RPC_E_CHANGED_MODE
  means "already initialized differently — proceed, don't uninit."
  ITaskbarList3 method indices are transcribed from the shell
  headers (Release=2, HrInit=3, SetProgressValue=9,
  SetProgressState=10); re-verify against shobjidl_core.h if the
  surface grows.
- tests/roundtrip.rs creates a REAL window on Windows CI and is the
  runtime proof. Off-Windows, the FFI unit test asserts precise
  NOT_SUPPORTED answers — never weaken either into stub successes.
- Gate: WinDevKit tools/gate.sh (fmt + clippy -D warnings BOTH
  targets + FULL tests).
