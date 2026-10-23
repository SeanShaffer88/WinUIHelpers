/* WinUIHelpers.dll — window polish behind one clean C ABI: dark
 * title bars, Mica/Acrylic backdrops, rounded corners, caption
 * colors, DPI helpers, and taskbar progress. All strings UTF-8,
 * NUL-terminated. char* results are caller-owned: free with
 * wh_free_string(). Window handles are passed as uint64_t (HWNDs are
 * pointer-sized). Capability gaps on older Windows builds return
 * WH_ERR_NOT_SUPPORTED with an explanation, never a silent no-op. */
#ifndef WINUIHELPERS_H
#define WINUIHELPERS_H
#include <stdint.h>
#ifdef __cplusplus
extern "C" {
#endif
#if defined(_WIN32) && defined(WH_BUILD)
#define WH_API __declspec(dllexport)
#elif defined(_WIN32)
#define WH_API __declspec(dllimport)
#else
#define WH_API
#endif
#define WH_OK 0
#define WH_ERR_INVALID_ARG 1
#define WH_ERR_NOT_SUPPORTED 2
#define WH_ERR_NOT_FOUND 3
#define WH_ERR_ACCESS_DENIED 4
#define WH_ERR_PLATFORM 5
#define WH_ERR_INTERNAL 6

WH_API const char *wh_version(void);
WH_API int wh_version_major(void);
WH_API int wh_version_minor(void);
WH_API int wh_version_patch(void);
WH_API const char *wh_last_error(void);
WH_API int wh_last_error_code(void);
WH_API void wh_free_string(char *p);

/* System queries. */
WH_API char *wh_windows_info_json(void); /* build, windows11, dark flags, accent */
WH_API int wh_apps_dark_mode(void);      /* 1 dark, 0 light, -1 error */
WH_API uint32_t wh_accent_color(void);   /* ARGB, 0 on error */

/* Title bar and frame. */
WH_API int wh_dark_titlebar(uint64_t hwnd, int enabled);
WH_API int wh_set_backdrop(uint64_t hwnd, int backdrop); /* 0 auto 1 none 2 mica 3 acrylic 4 mica-alt */
WH_API int wh_set_corners(uint64_t hwnd, int preference); /* 0 default 1 square 2 round 3 small */
WH_API int wh_set_caption_color(uint64_t hwnd, uint32_t rgb);
WH_API int wh_set_border_color(uint64_t hwnd, uint32_t rgb);
WH_API int wh_set_caption_text_color(uint64_t hwnd, uint32_t rgb);

/* DPI and geometry. */
WH_API int wh_get_dpi(uint64_t hwnd); /* 96 = 100%; -1 on error */
WH_API int wh_scale_px(int value, uint32_t dpi);
WH_API int wh_set_process_dpi_aware(void);
WH_API char *wh_window_info_json(uint64_t hwnd);
WH_API int wh_center_on_screen(uint64_t hwnd);
WH_API int wh_set_always_on_top(uint64_t hwnd, int on);
WH_API int wh_set_title(uint64_t hwnd, const char *title);

/* Taskbar progress (ITaskbarList3). */
WH_API int wh_taskbar_progress(uint64_t hwnd, uint64_t completed, uint64_t total);
WH_API int wh_taskbar_state(uint64_t hwnd, int state); /* 0 none 1 indet 2 normal 4 error 8 paused */
WH_API int wh_taskbar_clear(uint64_t hwnd);

#ifdef __cplusplus
}
#endif
#endif
