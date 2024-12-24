use windows::{
    core::s,
    Win32::{
        Foundation::{BOOL, HWND, LPARAM, WPARAM},
        UI::WindowsAndMessaging,
    },
};

use crate::Error;

extern "system" fn enum_window(window: HWND, ref_worker_w: LPARAM) -> BOOL {
    unsafe {
        let shell_dll_def_view = WindowsAndMessaging::FindWindowExA(
            window,
            HWND::default(),
            s!("SHELLDLL_DefView"),
            None,
        )
        .unwrap_or(HWND::default());

        if HWND::is_invalid(&shell_dll_def_view) {
            return BOOL(1);
        }

        let worker_w =
            WindowsAndMessaging::FindWindowExA(HWND::default(), window, s!("WorkerW"), None)
                .unwrap_or(HWND::default());

        if HWND::is_invalid(&worker_w) {
            return BOOL(1);
        }

        *(ref_worker_w.0 as *mut HWND) = worker_w;

        BOOL(1)
    }
}

pub fn attach<R: tauri::Runtime>(webview_window: tauri::WebviewWindow<R>) -> crate::Result<()> {
    let hwnd = webview_window.hwnd().unwrap();

    unsafe {
        let progman_hwnd = WindowsAndMessaging::FindWindowA(s!("Progman"), None).unwrap();

        WindowsAndMessaging::SendMessageA(progman_hwnd, 0x052C, WPARAM(0xD), LPARAM(0));
        WindowsAndMessaging::SendMessageA(progman_hwnd, 0x052C, WPARAM(0xD), LPARAM(1));

        let mut worker_w: HWND = HWND::default();

        WindowsAndMessaging::EnumWindows(
            Some(enum_window),
            LPARAM(&mut worker_w as *mut HWND as isize),
        )
        .unwrap();

        if HWND::is_invalid(&worker_w) {
            return Err(Error::WorkerWindowNotFound);
        }

        WindowsAndMessaging::SetParent(hwnd, worker_w).unwrap();
    }

    Ok(())
}
