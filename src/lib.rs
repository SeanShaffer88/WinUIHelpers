//! WinUI Helpers — the Windows window polish apps keep re-implementing
//! (dark title bars, Mica backdrop, rounded corners, caption colors,
//! DPI helpers, taskbar progress) behind one clean C ABI, in pure
//! Rust with zero dependencies and hand-declared Win32/COM bindings.
//!
//! Every feature is capability-checked against the actual Windows
//! build at runtime: backdrop surfaces and caption colors are
//! Windows 11, the immersive dark title bar works from Windows 10
//! 2004 (with the older attribute as fallback), and anything the
//! running OS can't do is a precise ERR_NOT_SUPPORTED, not a silent
//! no-op.

pub mod ffi;
pub mod platform;

pub const VERSION: &str = "0.1.0";
pub const VERSION_MAJOR: u32 = 0;
pub const VERSION_MINOR: u32 = 1;
pub const VERSION_PATCH: u32 = 0;

// Error codes (mirror include/winuihelpers.h).
pub const WH_OK: i32 = 0;
pub const ERR_INVALID_ARG: i32 = 1;
pub const ERR_NOT_SUPPORTED: i32 = 2;
pub const ERR_NOT_FOUND: i32 = 3;
pub const ERR_ACCESS_DENIED: i32 = 4;
pub const ERR_PLATFORM: i32 = 5;
pub const ERR_INTERNAL: i32 = 6;

/// The crate-wide result: failures carry a stable code + message.
pub type CResult<T> = Result<T, (i32, String)>;

// DWM attribute ids (dwmapi.h).
pub const DWMWA_USE_IMMERSIVE_DARK_MODE: u32 = 20;
pub const DWMWA_USE_IMMERSIVE_DARK_MODE_LEGACY: u32 = 19;
pub const DWMWA_BORDER_COLOR: u32 = 34;
pub const DWMWA_CAPTION_COLOR: u32 = 35;
pub const DWMWA_TEXT_COLOR: u32 = 36;
pub const DWMWA_SYSTEMBACKDROP_TYPE: u32 = 38;
pub const DWMWA_WINDOW_CORNER_PREFERENCE: u32 = 33;

// Backdrop kinds (public API values → DWM_SYSTEMBACKDROP_TYPE).
pub const BACKDROP_AUTO: i32 = 0;
pub const BACKDROP_NONE: i32 = 1;
pub const BACKDROP_MICA: i32 = 2;
pub const BACKDROP_ACRYLIC: i32 = 3;
pub const BACKDROP_MICA_ALT: i32 = 4;

// Taskbar progress states (TBPFLAG values).
pub const TBPF_NOPROGRESS: i32 = 0;
pub const TBPF_INDETERMINATE: i32 = 1;
pub const TBPF_NORMAL: i32 = 2;
pub const TBPF_ERROR: i32 = 4;
pub const TBPF_PAUSED: i32 = 8;

/// Windows 11 starts at build 22000; the system-backdrop and corner
/// attributes arrived with it.
pub fn is_windows_11(build: u32) -> bool {
    build >= 22000
}

/// Scale a pixel value authored at 96 DPI for the given DPI.
pub fn scale_px(value: i32, dpi: u32) -> i32 {
    if dpi == 0 {
        return value;
    }
    ((value as f64 * dpi as f64) / 96.0).round() as i32
}

/// Format an ARGB color as "#RRGGBB".
pub fn color_hex(argb: u32) -> String {
    format!(
        "#{:02X}{:02X}{:02X}",
        (argb >> 16) & 0xFF,
        (argb >> 8) & 0xFF,
        argb & 0xFF
    )
}

/// Escape a string for inclusion inside JSON double quotes.
pub fn json_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 8);
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_math() {
        assert!(!is_windows_11(19045));
        assert!(is_windows_11(22000));
        assert!(is_windows_11(26100));
        assert_eq!(scale_px(100, 96), 100);
        assert_eq!(scale_px(100, 144), 150);
        assert_eq!(scale_px(100, 120), 125);
        assert_eq!(scale_px(7, 0), 7);
        assert_eq!(color_hex(0xFF0078D4), "#0078D4");
        assert_eq!(color_hex(0x00000000), "#000000");
    }
}
