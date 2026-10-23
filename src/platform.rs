//! Platform layer: hand-declared Win32 + COM bindings.
//! Every extern block carries its #[link(name = "...")] — the build
//! fails at LINK time without it; check/clippy cannot catch that.
//!
//! On non-Windows everything returns ERR_NOT_SUPPORTED so the FFI
//! contract stays testable anywhere.

use crate::{CResult, ERR_NOT_SUPPORTED};

pub fn not_supported<T>(what: &str) -> CResult<T> {
    Err((
        ERR_NOT_SUPPORTED,
        format!("{what} is only available on Windows"),
    ))
}

#[cfg(windows)]
pub use win::{
    apply_dark_titlebar, center_on_screen, get_dpi, set_always_on_top, set_backdrop,
    set_border_color, set_caption_color, set_corner_preference, set_process_dpi_aware,
    set_text_color, set_title, taskbar_clear, taskbar_set_progress, taskbar_set_state, window_info,
    windows_info, WindowsInfo,
};

#[cfg(windows)]
mod win {
    use std::ffi::c_void;

    use crate::{
        CResult, DWMWA_BORDER_COLOR, DWMWA_CAPTION_COLOR, DWMWA_SYSTEMBACKDROP_TYPE,
        DWMWA_TEXT_COLOR, DWMWA_USE_IMMERSIVE_DARK_MODE, DWMWA_USE_IMMERSIVE_DARK_MODE_LEGACY,
        DWMWA_WINDOW_CORNER_PREFERENCE, ERR_INVALID_ARG, ERR_NOT_SUPPORTED, ERR_PLATFORM,
        TBPF_NOPROGRESS,
    };

    pub type Hwnd = *mut c_void;

    #[link(name = "ntdll")]
    extern "system" {
        fn RtlGetVersion(info: *mut OsVersionInfoW) -> i32;
    }
    #[repr(C)]
    struct OsVersionInfoW {
        size: u32,
        major: u32,
        minor: u32,
        build: u32,
        platform_id: u32,
        csd_version: [u16; 128],
    }

    #[link(name = "dwmapi")]
    extern "system" {
        fn DwmSetWindowAttribute(hwnd: Hwnd, attr: u32, value: *const c_void, size: u32) -> i32;
        fn DwmGetColorizationColor(color: *mut u32, opaque: *mut i32) -> i32;
    }

    #[link(name = "user32")]
    extern "system" {
        fn IsWindow(hwnd: Hwnd) -> i32;
        fn GetDpiForWindow(hwnd: Hwnd) -> u32;
        fn GetWindowRect(hwnd: Hwnd, rect: *mut Rect) -> i32;
        fn GetWindowTextW(hwnd: Hwnd, text: *mut u16, max: i32) -> i32;
        fn GetWindowTextLengthW(hwnd: Hwnd) -> i32;
        fn SetWindowPos(
            hwnd: Hwnd,
            insert_after: Hwnd,
            x: i32,
            y: i32,
            cx: i32,
            cy: i32,
            flags: u32,
        ) -> i32;
        fn GetSystemMetrics(index: i32) -> i32;
        fn SetProcessDpiAwarenessContext(context: isize) -> i32;
        fn SetWindowTextW(hwnd: Hwnd, text: *const u16) -> i32;
    }

    #[repr(C)]
    struct Rect {
        left: i32,
        top: i32,
        right: i32,
        bottom: i32,
    }

    #[link(name = "advapi32")]
    extern "system" {
        fn RegOpenKeyExW(
            key: isize,
            subkey: *const u16,
            options: u32,
            sam: u32,
            result: *mut isize,
        ) -> i32;
        fn RegQueryValueExW(
            key: isize,
            name: *const u16,
            reserved: *mut u32,
            value_type: *mut u32,
            data: *mut u8,
            size: *mut u32,
        ) -> i32;
        fn RegCloseKey(key: isize) -> i32;
    }

    #[link(name = "ole32")]
    extern "system" {
        fn CoInitializeEx(reserved: *mut c_void, coinit: u32) -> i32;
        fn CoUninitialize();
        fn CoCreateInstance(
            rclsid: *const Guid,
            outer: *mut c_void,
            context: u32,
            riid: *const Guid,
            result: *mut *mut c_void,
        ) -> i32;
    }

    #[repr(C)]
    #[derive(Clone, Copy)]
    struct Guid {
        d1: u32,
        d2: u16,
        d3: u16,
        d4: [u8; 8],
    }

    const CLSID_TASKBAR_LIST: Guid = Guid {
        d1: 0x56FD_F344,
        d2: 0xFD6D,
        d3: 0x11D0,
        d4: [0x95, 0x8A, 0x00, 0x60, 0x97, 0xC9, 0xA0, 0x90],
    };
    const IID_ITASKBAR_LIST3: Guid = Guid {
        d1: 0xEA1A_FB91,
        d2: 0x9E28,
        d3: 0x4B86,
        d4: [0x90, 0xE9, 0x9E, 0x9F, 0x8A, 0x5E, 0xEF, 0xAF],
    };

    const HKEY_CURRENT_USER: isize = 0x8000_0001u32 as i32 as isize;
    const KEY_READ: u32 = 0x0002_0019;
    const ERROR_SUCCESS: i32 = 0;

    fn platform_err(what: &str) -> (i32, String) {
        (ERR_PLATFORM, format!("{what} failed"))
    }

    pub fn valid_hwnd(hwnd: u64) -> CResult<Hwnd> {
        let h = hwnd as Hwnd;
        if h.is_null() || unsafe { IsWindow(h) } == 0 {
            return Err((ERR_INVALID_ARG, format!("{hwnd} is not a window handle")));
        }
        Ok(h)
    }

    pub fn build_number() -> u32 {
        let mut info = OsVersionInfoW {
            size: std::mem::size_of::<OsVersionInfoW>() as u32,
            major: 0,
            minor: 0,
            build: 0,
            platform_id: 0,
            csd_version: [0; 128],
        };
        let status = unsafe { RtlGetVersion(&mut info) };
        if status == 0 {
            info.build
        } else {
            0
        }
    }

    fn registry_dword(subkey: &str, name: &str) -> Option<u32> {
        let sub: Vec<u16> = subkey.encode_utf16().chain(std::iter::once(0)).collect();
        let nm: Vec<u16> = name.encode_utf16().chain(std::iter::once(0)).collect();
        let mut key: isize = 0;
        let rc = unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, sub.as_ptr(), 0, KEY_READ, &mut key) };
        if rc != ERROR_SUCCESS {
            return None;
        }
        let mut data = [0u8; 4];
        let mut size = 4u32;
        let mut ty = 0u32;
        let rc = unsafe {
            RegQueryValueExW(
                key,
                nm.as_ptr(),
                std::ptr::null_mut(),
                &mut ty,
                data.as_mut_ptr(),
                &mut size,
            )
        };
        unsafe { RegCloseKey(key) };
        if rc == ERROR_SUCCESS && ty == 4 && size == 4 {
            Some(u32::from_le_bytes(data))
        } else {
            None
        }
    }

    #[derive(Debug, Clone)]
    pub struct WindowsInfo {
        pub build: u32,
        pub dark_apps: bool,
        pub dark_system: bool,
        pub accent_argb: u32,
    }

    pub fn windows_info() -> CResult<WindowsInfo> {
        let personalize = "Software\\Microsoft\\Windows\\CurrentVersion\\Themes\\Personalize";
        let apps_light = registry_dword(personalize, "AppsUseLightTheme").unwrap_or(1);
        let sys_light = registry_dword(personalize, "SystemUsesLightTheme").unwrap_or(apps_light);
        let mut color: u32 = 0;
        let mut opaque: i32 = 0;
        let hr = unsafe { DwmGetColorizationColor(&mut color, &mut opaque) };
        Ok(WindowsInfo {
            build: build_number(),
            dark_apps: apps_light == 0,
            dark_system: sys_light == 0,
            accent_argb: if hr == 0 { color } else { 0 },
        })
    }

    fn dwm_set(hwnd: Hwnd, attr: u32, value: i32) -> CResult<()> {
        let hr = unsafe {
            DwmSetWindowAttribute(
                hwnd,
                attr,
                &value as *const i32 as *const c_void,
                std::mem::size_of::<i32>() as u32,
            )
        };
        if hr == 0 {
            Ok(())
        } else {
            Err((
                ERR_PLATFORM,
                format!(
                    "DwmSetWindowAttribute({attr}) failed with HRESULT 0x{:08X}",
                    hr as u32
                ),
            ))
        }
    }

    pub fn apply_dark_titlebar(hwnd: u64, enabled: bool) -> CResult<()> {
        let h = valid_hwnd(hwnd)?;
        let value: i32 = if enabled { 1 } else { 0 };
        // Attribute 20 (Windows 10 2004+/11); 19 on the earliest
        // dark-mode-capable builds.
        if dwm_set(h, DWMWA_USE_IMMERSIVE_DARK_MODE, value).is_ok() {
            return Ok(());
        }
        dwm_set(h, DWMWA_USE_IMMERSIVE_DARK_MODE_LEGACY, value).map_err(|_| {
            (
                ERR_NOT_SUPPORTED,
                "immersive dark title bar is not supported on this Windows build".into(),
            )
        })
    }

    pub fn set_backdrop(hwnd: u64, backdrop: i32) -> CResult<()> {
        let h = valid_hwnd(hwnd)?;
        if !crate::is_windows_11(build_number()) {
            return Err((
                ERR_NOT_SUPPORTED,
                "system backdrops (Mica/Acrylic) require Windows 11".into(),
            ));
        }
        if !(0..=4).contains(&backdrop) {
            return Err((ERR_INVALID_ARG, format!("unknown backdrop {backdrop}")));
        }
        dwm_set(h, DWMWA_SYSTEMBACKDROP_TYPE, backdrop)
    }

    pub fn set_corner_preference(hwnd: u64, preference: i32) -> CResult<()> {
        let h = valid_hwnd(hwnd)?;
        if !crate::is_windows_11(build_number()) {
            return Err((
                ERR_NOT_SUPPORTED,
                "window corner preference requires Windows 11".into(),
            ));
        }
        if !(0..=3).contains(&preference) {
            return Err((
                ERR_INVALID_ARG,
                format!("unknown corner preference {preference}"),
            ));
        }
        dwm_set(h, DWMWA_WINDOW_CORNER_PREFERENCE, preference)
    }

    fn rgb_to_colorref(rgb: u32) -> i32 {
        // COLORREF is 0x00BBGGRR.
        (((rgb & 0xFF) << 16) | (rgb & 0xFF00) | ((rgb >> 16) & 0xFF)) as i32
    }

    pub fn set_caption_color(hwnd: u64, rgb: u32) -> CResult<()> {
        let h = valid_hwnd(hwnd)?;
        if !crate::is_windows_11(build_number()) {
            return Err((
                ERR_NOT_SUPPORTED,
                "caption color requires Windows 11".into(),
            ));
        }
        dwm_set(h, DWMWA_CAPTION_COLOR, rgb_to_colorref(rgb))
    }

    pub fn set_border_color(hwnd: u64, rgb: u32) -> CResult<()> {
        let h = valid_hwnd(hwnd)?;
        if !crate::is_windows_11(build_number()) {
            return Err((ERR_NOT_SUPPORTED, "border color requires Windows 11".into()));
        }
        dwm_set(h, DWMWA_BORDER_COLOR, rgb_to_colorref(rgb))
    }

    pub fn set_text_color(hwnd: u64, rgb: u32) -> CResult<()> {
        let h = valid_hwnd(hwnd)?;
        if !crate::is_windows_11(build_number()) {
            return Err((
                ERR_NOT_SUPPORTED,
                "caption text color requires Windows 11".into(),
            ));
        }
        dwm_set(h, DWMWA_TEXT_COLOR, rgb_to_colorref(rgb))
    }

    pub fn get_dpi(hwnd: u64) -> CResult<u32> {
        let h = valid_hwnd(hwnd)?;
        let dpi = unsafe { GetDpiForWindow(h) };
        Ok(if dpi == 0 { 96 } else { dpi })
    }

    #[derive(Debug, Clone)]
    pub struct WindowInfo {
        pub title: String,
        pub width: i32,
        pub height: i32,
        pub dpi: u32,
    }

    pub fn window_info(hwnd: u64) -> CResult<WindowInfo> {
        let h = valid_hwnd(hwnd)?;
        let len = unsafe { GetWindowTextLengthW(h) };
        let mut buf = vec![0u16; (len.max(0) as usize) + 1];
        let got = unsafe { GetWindowTextW(h, buf.as_mut_ptr(), buf.len() as i32) };
        let title = if got > 0 {
            String::from_utf16_lossy(&buf[..got as usize])
        } else {
            String::new()
        };
        let mut rect = Rect {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        if unsafe { GetWindowRect(h, &mut rect) } == 0 {
            return Err(platform_err("GetWindowRect"));
        }
        Ok(WindowInfo {
            title,
            width: rect.right - rect.left,
            height: rect.bottom - rect.top,
            dpi: {
                let d = unsafe { GetDpiForWindow(h) };
                if d == 0 {
                    96
                } else {
                    d
                }
            },
        })
    }

    pub fn center_on_screen(hwnd: u64) -> CResult<()> {
        let h = valid_hwnd(hwnd)?;
        let mut rect = Rect {
            left: 0,
            top: 0,
            right: 0,
            bottom: 0,
        };
        if unsafe { GetWindowRect(h, &mut rect) } == 0 {
            return Err(platform_err("GetWindowRect"));
        }
        let (w, ht) = (rect.right - rect.left, rect.bottom - rect.top);
        let screen_w = unsafe { GetSystemMetrics(0) }; // SM_CXSCREEN
        let screen_h = unsafe { GetSystemMetrics(1) }; // SM_CYSCREEN
        let x = (screen_w - w) / 2;
        let y = (screen_h - ht) / 2;
        const SWP_NOSIZE: u32 = 0x0001;
        const SWP_NOZORDER: u32 = 0x0004;
        if unsafe {
            SetWindowPos(
                h,
                std::ptr::null_mut(),
                x,
                y,
                0,
                0,
                SWP_NOSIZE | SWP_NOZORDER,
            )
        } == 0
        {
            return Err(platform_err("SetWindowPos"));
        }
        Ok(())
    }

    pub fn set_always_on_top(hwnd: u64, on: bool) -> CResult<()> {
        let h = valid_hwnd(hwnd)?;
        const HWND_TOPMOST: isize = -1;
        const HWND_NOTOPMOST: isize = -2;
        const SWP_NOMOVE: u32 = 0x0002;
        const SWP_NOSIZE: u32 = 0x0001;
        let after = (if on { HWND_TOPMOST } else { HWND_NOTOPMOST }) as Hwnd;
        if unsafe { SetWindowPos(h, after, 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE) } == 0 {
            return Err(platform_err("SetWindowPos"));
        }
        Ok(())
    }

    pub fn set_process_dpi_aware() -> CResult<()> {
        // DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2 = (HANDLE)-4.
        let ok = unsafe { SetProcessDpiAwarenessContext(-4isize) };
        if ok != 0 {
            Ok(())
        } else {
            Err((
                ERR_PLATFORM,
                "SetProcessDpiAwarenessContext failed — awareness may already be set (e.g. by manifest)".into(),
            ))
        }
    }

    pub fn set_title(hwnd: u64, title: &str) -> CResult<()> {
        let h = valid_hwnd(hwnd)?;
        let wide: Vec<u16> = title.encode_utf16().chain(std::iter::once(0)).collect();
        if unsafe { SetWindowTextW(h, wide.as_ptr()) } == 0 {
            return Err(platform_err("SetWindowTextW"));
        }
        Ok(())
    }

    /// Run `f` with an ITaskbarList3 pointer, handling COM init:
    /// if the thread was already initialized with another model
    /// (RPC_E_CHANGED_MODE), proceed without uninitializing.
    fn with_taskbar<T, F: FnOnce(*mut c_void) -> CResult<T>>(f: F) -> CResult<T> {
        const COINIT_APARTMENTTHREADED: u32 = 0x2;
        const CLSCTX_ALL: u32 = 23;
        const S_OK: i32 = 0;
        const S_FALSE: i32 = 1;
        const RPC_E_CHANGED_MODE: i32 = 0x8001_0106u32 as i32;
        let hr = unsafe { CoInitializeEx(std::ptr::null_mut(), COINIT_APARTMENTTHREADED) };
        let should_uninit = hr == S_OK || hr == S_FALSE;
        if hr < 0 && hr != RPC_E_CHANGED_MODE {
            return Err((
                ERR_PLATFORM,
                format!("CoInitializeEx failed: 0x{:08X}", hr as u32),
            ));
        }
        let result = (|| -> CResult<T> {
            let mut obj: *mut c_void = std::ptr::null_mut();
            let hr = unsafe {
                CoCreateInstance(
                    &CLSID_TASKBAR_LIST,
                    std::ptr::null_mut(),
                    CLSCTX_ALL,
                    &IID_ITASKBAR_LIST3,
                    &mut obj,
                )
            };
            if hr < 0 || obj.is_null() {
                return Err((
                    ERR_PLATFORM,
                    format!("CoCreateInstance(TaskbarList) failed: 0x{:08X}", hr as u32),
                ));
            }
            let out = f(obj);
            unsafe {
                let vtbl = *(obj as *const *const usize);
                let release: unsafe extern "system" fn(*mut c_void) -> u32 =
                    std::mem::transmute(*vtbl.add(2));
                release(obj);
            }
            out
        })();
        if should_uninit {
            unsafe { CoUninitialize() };
        }
        result
    }

    unsafe fn vtbl_fn<T>(obj: *mut c_void, index: usize) -> T {
        let vtbl = *(obj as *const *const usize);
        std::mem::transmute_copy(&*vtbl.add(index))
    }

    pub fn taskbar_set_progress(hwnd: u64, completed: u64, total: u64) -> CResult<()> {
        let h = valid_hwnd(hwnd)?;
        with_taskbar(|obj| unsafe {
            let hr_init: unsafe extern "system" fn(*mut c_void) -> i32 = vtbl_fn(obj, 3);
            hr_init(obj);
            let set_value: unsafe extern "system" fn(*mut c_void, Hwnd, u64, u64) -> i32 =
                vtbl_fn(obj, 9);
            let hr = set_value(obj, h, completed, total);
            if hr < 0 {
                return Err((
                    ERR_PLATFORM,
                    format!("SetProgressValue failed: 0x{:08X}", hr as u32),
                ));
            }
            Ok(())
        })
    }

    pub fn taskbar_set_state(hwnd: u64, state: i32) -> CResult<()> {
        let h = valid_hwnd(hwnd)?;
        if ![0, 1, 2, 4, 8].contains(&state) {
            return Err((ERR_INVALID_ARG, format!("unknown taskbar state {state}")));
        }
        with_taskbar(|obj| unsafe {
            let hr_init: unsafe extern "system" fn(*mut c_void) -> i32 = vtbl_fn(obj, 3);
            hr_init(obj);
            let set_state: unsafe extern "system" fn(*mut c_void, Hwnd, i32) -> i32 =
                vtbl_fn(obj, 10);
            let hr = set_state(obj, h, state);
            if hr < 0 {
                return Err((
                    ERR_PLATFORM,
                    format!("SetProgressState failed: 0x{:08X}", hr as u32),
                ));
            }
            Ok(())
        })
    }

    pub fn taskbar_clear(hwnd: u64) -> CResult<()> {
        taskbar_set_state(hwnd, TBPF_NOPROGRESS)
    }
}

#[cfg(windows)]
pub use win::WindowInfo;

// Non-Windows stubs.
#[cfg(not(windows))]
pub struct WindowsInfo {
    pub build: u32,
    pub dark_apps: bool,
    pub dark_system: bool,
    pub accent_argb: u32,
}
#[cfg(not(windows))]
pub struct WindowInfo {
    pub title: String,
    pub width: i32,
    pub height: i32,
    pub dpi: u32,
}

#[cfg(not(windows))]
pub fn windows_info() -> CResult<WindowsInfo> {
    not_supported("windows_info")
}
#[cfg(not(windows))]
pub fn apply_dark_titlebar(_h: u64, _e: bool) -> CResult<()> {
    not_supported("apply_dark_titlebar")
}
#[cfg(not(windows))]
pub fn set_backdrop(_h: u64, _b: i32) -> CResult<()> {
    not_supported("set_backdrop")
}
#[cfg(not(windows))]
pub fn set_corner_preference(_h: u64, _p: i32) -> CResult<()> {
    not_supported("set_corner_preference")
}
#[cfg(not(windows))]
pub fn set_caption_color(_h: u64, _c: u32) -> CResult<()> {
    not_supported("set_caption_color")
}
#[cfg(not(windows))]
pub fn set_border_color(_h: u64, _c: u32) -> CResult<()> {
    not_supported("set_border_color")
}
#[cfg(not(windows))]
pub fn set_text_color(_h: u64, _c: u32) -> CResult<()> {
    not_supported("set_text_color")
}
#[cfg(not(windows))]
pub fn get_dpi(_h: u64) -> CResult<u32> {
    not_supported("get_dpi")
}
#[cfg(not(windows))]
pub fn window_info(_h: u64) -> CResult<WindowInfo> {
    not_supported("window_info")
}
#[cfg(not(windows))]
pub fn center_on_screen(_h: u64) -> CResult<()> {
    not_supported("center_on_screen")
}
#[cfg(not(windows))]
pub fn set_always_on_top(_h: u64, _o: bool) -> CResult<()> {
    not_supported("set_always_on_top")
}
#[cfg(not(windows))]
pub fn set_process_dpi_aware() -> CResult<()> {
    not_supported("set_process_dpi_aware")
}
#[cfg(not(windows))]
pub fn set_title(_h: u64, _t: &str) -> CResult<()> {
    not_supported("set_title")
}
#[cfg(not(windows))]
pub fn taskbar_set_progress(_h: u64, _c: u64, _t: u64) -> CResult<()> {
    not_supported("taskbar_set_progress")
}
#[cfg(not(windows))]
pub fn taskbar_set_state(_h: u64, _s: i32) -> CResult<()> {
    not_supported("taskbar_set_state")
}
#[cfg(not(windows))]
pub fn taskbar_clear(_h: u64) -> CResult<()> {
    not_supported("taskbar_clear")
}
