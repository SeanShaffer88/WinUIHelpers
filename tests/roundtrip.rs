//! Windows-only CI proof: create a REAL window in this test process
//! and run the polish calls against it — dark title bar, window info,
//! DPI, centering, and the ITaskbarList3 COM path — then destroy it.
//! On non-Windows this file is empty (the FFI unit tests cover the
//! contract there).
#![cfg(windows)]

use std::ffi::c_void;

use winuihelpers::ffi::*;
use winuihelpers::{ERR_NOT_SUPPORTED, WH_OK};

#[link(name = "user32")]
extern "system" {
    fn CreateWindowExW(
        ex_style: u32,
        class: *const u16,
        title: *const u16,
        style: u32,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        parent: *mut c_void,
        menu: *mut c_void,
        instance: *mut c_void,
        param: *mut c_void,
    ) -> *mut c_void;
    fn DestroyWindow(hwnd: *mut c_void) -> i32;
}

fn wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

#[test]
fn polish_a_real_window() {
    let class = wide("STATIC");
    let title = wide("WinUIHelpers CI window");
    let hwnd = unsafe {
        CreateWindowExW(
            0,
            class.as_ptr(),
            title.as_ptr(),
            0x10CF0000, // WS_OVERLAPPEDWINDOW | WS_VISIBLE family
            100,
            100,
            640,
            480,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
        )
    };
    assert!(!hwnd.is_null(), "CreateWindowExW failed");
    let h = hwnd as u64;

    // Window info sees the real window and its title/size.
    let info_ptr = wh_window_info_json(h);
    assert!(!info_ptr.is_null());
    let info = unsafe { std::ffi::CStr::from_ptr(info_ptr) }
        .to_string_lossy()
        .into_owned();
    wh_free_string(info_ptr);
    assert!(info.contains("WinUIHelpers CI window"), "{info}");
    assert!(info.contains("\"width\":640"), "{info}");
    let dpi = wh_get_dpi(h);
    assert!(dpi >= 96, "dpi {dpi}");

    // Dark title bar works on Windows 10 2004+ and 11 (CI is 11-class
    // or late-10); anything older must say NOT_SUPPORTED precisely.
    let rc = wh_dark_titlebar(h, 1);
    assert!(
        rc == WH_OK || rc == ERR_NOT_SUPPORTED,
        "dark titlebar rc {rc}: {}",
        unsafe { std::ffi::CStr::from_ptr(wh_last_error()) }.to_string_lossy()
    );
    assert_eq!(wh_dark_titlebar(0, 1), winuihelpers::ERR_INVALID_ARG);

    // Geometry helpers.
    assert_eq!(wh_center_on_screen(h), WH_OK);
    assert_eq!(wh_set_always_on_top(h, 1), WH_OK);
    assert_eq!(wh_set_always_on_top(h, 0), WH_OK);

    // Taskbar progress through the hand-rolled COM vtable: these are
    // valid even without a visible taskbar button.
    assert_eq!(wh_taskbar_state(h, 2), WH_OK); // NORMAL
    assert_eq!(wh_taskbar_progress(h, 50, 100), WH_OK);
    assert_eq!(wh_taskbar_state(h, 8), WH_OK); // PAUSED
    assert_eq!(wh_taskbar_clear(h), WH_OK);
    assert_eq!(wh_taskbar_state(h, 99), winuihelpers::ERR_INVALID_ARG);

    // System info JSON is well-formed and internally consistent.
    let sys_ptr = wh_windows_info_json();
    assert!(!sys_ptr.is_null());
    let sys = unsafe { std::ffi::CStr::from_ptr(sys_ptr) }
        .to_string_lossy()
        .into_owned();
    wh_free_string(sys_ptr);
    assert!(sys.contains("\"build\":"), "{sys}");
    assert!(
        wh_accent_color() != 0 || sys.contains("\"accent_argb\":0"),
        "{sys}"
    );

    unsafe { DestroyWindow(hwnd) };
}
