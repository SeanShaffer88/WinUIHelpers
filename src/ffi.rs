//! C ABI surface — everything declared in include/winuihelpers.h.
//! Conventions (house standard):
//! - Window handles cross the ABI as u64 (HWNDs are pointer-sized).
//! - `*mut c_char` results are caller-owned; free with `wh_free_string`.
//! - Failures return an ERR_* code + a thread-local last-error message.
//! - Nothing unwinds across the FFI boundary.

use std::cell::RefCell;
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int};
use std::panic::{catch_unwind, AssertUnwindSafe};

use crate::platform;
use crate::{
    color_hex, json_escape, scale_px, ERR_INTERNAL, ERR_INVALID_ARG, VERSION_MAJOR, VERSION_MINOR,
    VERSION_PATCH, WH_OK,
};

thread_local! {
    static LAST_ERROR: RefCell<(i32, CString)> =
        RefCell::new((0, CString::new("").unwrap()));
}

pub fn set_error(code: i32, msg: &str) {
    let cleaned: String = msg
        .chars()
        .map(|c| if c == '\0' { ' ' } else { c })
        .collect();
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = (
            code,
            CString::new(cleaned).unwrap_or_else(|_| CString::new("error").unwrap()),
        );
    });
}

fn clear_error() {
    set_error(WH_OK, "");
}

fn guard<T, F: FnOnce() -> T>(f: F) -> Result<T, (i32, String)> {
    match catch_unwind(AssertUnwindSafe(f)) {
        Ok(v) => Ok(v),
        Err(_) => Err((ERR_INTERNAL, "internal panic caught at FFI boundary".into())),
    }
}

fn finish(run: Result<Result<(), (i32, String)>, (i32, String)>) -> c_int {
    match run {
        Ok(Ok(())) => {
            clear_error();
            WH_OK
        }
        Ok(Err((code, msg))) | Err((code, msg)) => {
            set_error(code, &msg);
            code
        }
    }
}

fn finish_str(run: Result<Result<String, (i32, String)>, (i32, String)>) -> *mut c_char {
    match run {
        Ok(Ok(s)) => {
            clear_error();
            match CString::new(s) {
                Ok(c) => c.into_raw(),
                Err(_) => {
                    set_error(ERR_INTERNAL, "result contained a NUL byte");
                    std::ptr::null_mut()
                }
            }
        }
        Ok(Err((code, msg))) | Err((code, msg)) => {
            set_error(code, &msg);
            std::ptr::null_mut()
        }
    }
}

#[no_mangle]
pub extern "C" fn wh_version() -> *const c_char {
    static VERSION_C: &[u8] = b"0.1.0\0";
    VERSION_C.as_ptr() as *const c_char
}

#[no_mangle]
pub extern "C" fn wh_version_major() -> c_int {
    VERSION_MAJOR as c_int
}

#[no_mangle]
pub extern "C" fn wh_version_minor() -> c_int {
    VERSION_MINOR as c_int
}

#[no_mangle]
pub extern "C" fn wh_version_patch() -> c_int {
    VERSION_PATCH as c_int
}

#[no_mangle]
pub extern "C" fn wh_last_error() -> *const c_char {
    LAST_ERROR.with(|e| e.borrow().1.as_ptr())
}

#[no_mangle]
pub extern "C" fn wh_last_error_code() -> c_int {
    LAST_ERROR.with(|e| e.borrow().0) as c_int
}

/// # Safety (caller contract): `p` is null or a string this library
/// returned and has not been freed yet.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn wh_free_string(p: *mut c_char) {
    if !p.is_null() {
        unsafe {
            drop(CString::from_raw(p));
        }
    }
}

/// {"build":26100,"windows11":true,"dark_apps":true,"dark_system":false,
///  "accent":"#0078D4","accent_argb":4293912788}
#[no_mangle]
pub extern "C" fn wh_windows_info_json() -> *mut c_char {
    finish_str(guard(|| {
        platform::windows_info().map(|i| {
            format!(
                "{{\"build\":{},\"windows11\":{},\"dark_apps\":{},\"dark_system\":{},\"accent\":\"{}\",\"accent_argb\":{}}}",
                i.build,
                crate::is_windows_11(i.build),
                i.dark_apps,
                i.dark_system,
                color_hex(i.accent_argb),
                i.accent_argb
            )
        })
    }))
}

/// 1 when the app's theme is dark (AppsUseLightTheme == 0), 0 when
/// light, -1 on error.
#[no_mangle]
pub extern "C" fn wh_apps_dark_mode() -> c_int {
    match guard(platform::windows_info) {
        Ok(Ok(i)) => {
            clear_error();
            if i.dark_apps {
                1
            } else {
                0
            }
        }
        Ok(Err((code, msg))) | Err((code, msg)) => {
            set_error(code, &msg);
            -1
        }
    }
}

#[no_mangle]
pub extern "C" fn wh_accent_color() -> u32 {
    match guard(platform::windows_info) {
        Ok(Ok(i)) => i.accent_argb,
        Ok(Err((code, msg))) | Err((code, msg)) => {
            set_error(code, &msg);
            0
        }
    }
}

#[no_mangle]
pub extern "C" fn wh_dark_titlebar(hwnd: u64, enabled: c_int) -> c_int {
    finish(guard(|| platform::apply_dark_titlebar(hwnd, enabled != 0)))
}

/// backdrop: 0 auto, 1 none, 2 Mica, 3 Acrylic, 4 Mica Alt (Win11).
#[no_mangle]
pub extern "C" fn wh_set_backdrop(hwnd: u64, backdrop: c_int) -> c_int {
    finish(guard(|| platform::set_backdrop(hwnd, backdrop)))
}

/// preference: 0 default, 1 do-not-round, 2 round, 3 round-small.
#[no_mangle]
pub extern "C" fn wh_set_corners(hwnd: u64, preference: c_int) -> c_int {
    finish(guard(|| platform::set_corner_preference(hwnd, preference)))
}

#[no_mangle]
pub extern "C" fn wh_set_caption_color(hwnd: u64, rgb: u32) -> c_int {
    finish(guard(|| platform::set_caption_color(hwnd, rgb)))
}

#[no_mangle]
pub extern "C" fn wh_set_border_color(hwnd: u64, rgb: u32) -> c_int {
    finish(guard(|| platform::set_border_color(hwnd, rgb)))
}

#[no_mangle]
pub extern "C" fn wh_set_caption_text_color(hwnd: u64, rgb: u32) -> c_int {
    finish(guard(|| platform::set_text_color(hwnd, rgb)))
}

/// The window's DPI (96 = 100%), or -1 on error.
#[no_mangle]
pub extern "C" fn wh_get_dpi(hwnd: u64) -> c_int {
    match guard(|| platform::get_dpi(hwnd)) {
        Ok(Ok(d)) => {
            clear_error();
            d as c_int
        }
        Ok(Err((code, msg))) | Err((code, msg)) => {
            set_error(code, &msg);
            -1
        }
    }
}

/// Pure helper: scale pixels authored at 96 DPI. Never fails.
#[no_mangle]
pub extern "C" fn wh_scale_px(value: c_int, dpi: u32) -> c_int {
    scale_px(value, dpi)
}

/// Make the CALLING process per-monitor DPI aware (V2). Call once,
/// before creating windows.
#[no_mangle]
pub extern "C" fn wh_set_process_dpi_aware() -> c_int {
    finish(guard(platform::set_process_dpi_aware))
}

/// {"title":"…","width":800,"height":600,"dpi":96}
#[no_mangle]
pub extern "C" fn wh_window_info_json(hwnd: u64) -> *mut c_char {
    finish_str(guard(|| {
        platform::window_info(hwnd).map(|w| {
            format!(
                "{{\"title\":\"{}\",\"width\":{},\"height\":{},\"dpi\":{}}}",
                json_escape(&w.title),
                w.width,
                w.height,
                w.dpi
            )
        })
    }))
}

#[no_mangle]
pub extern "C" fn wh_center_on_screen(hwnd: u64) -> c_int {
    finish(guard(|| platform::center_on_screen(hwnd)))
}

#[no_mangle]
pub extern "C" fn wh_set_always_on_top(hwnd: u64, on: c_int) -> c_int {
    finish(guard(|| platform::set_always_on_top(hwnd, on != 0)))
}

/// # Safety (caller contract): title is null or a valid C string.
#[allow(clippy::not_unsafe_ptr_arg_deref)]
#[no_mangle]
pub extern "C" fn wh_set_title(hwnd: u64, title: *const c_char) -> c_int {
    finish(guard(|| {
        if title.is_null() {
            return Err((ERR_INVALID_ARG, "title must not be null".into()));
        }
        let s = unsafe { CStr::from_ptr(title) }
            .to_str()
            .map_err(|_| (ERR_INVALID_ARG, "title must be valid UTF-8".to_string()))?;
        platform::set_title(hwnd, s)
    }))
}

#[no_mangle]
pub extern "C" fn wh_taskbar_progress(hwnd: u64, completed: u64, total: u64) -> c_int {
    finish(guard(|| {
        platform::taskbar_set_progress(hwnd, completed, total)
    }))
}

/// state: 0 none, 1 indeterminate, 2 normal, 4 error, 8 paused.
#[no_mangle]
pub extern "C" fn wh_taskbar_state(hwnd: u64, state: c_int) -> c_int {
    finish(guard(|| platform::taskbar_set_state(hwnd, state)))
}

#[no_mangle]
pub extern "C" fn wh_taskbar_clear(hwnd: u64) -> c_int {
    finish(guard(|| platform::taskbar_clear(hwnd)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn non_windows_surface_behaves() {
        // On any host: pure helpers work; OS calls are precise errors.
        assert_eq!(wh_scale_px(100, 144), 150);
        if cfg!(windows) {
            // Exercised natively in CI (tests/roundtrip.rs).
        } else {
            assert_eq!(wh_dark_titlebar(0, 1), crate::ERR_NOT_SUPPORTED);
            assert_eq!(wh_get_dpi(0), -1);
            assert_eq!(wh_last_error_code(), crate::ERR_NOT_SUPPORTED);
            let p = wh_windows_info_json();
            assert!(p.is_null());
        }
    }
}
