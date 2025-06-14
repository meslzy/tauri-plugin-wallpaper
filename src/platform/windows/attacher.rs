use windows::{
    core::s,
    Win32::{
        Foundation::{HWND, LPARAM, WPARAM},
        UI::WindowsAndMessaging,
    },
};
use windows_core::BOOL;

extern "system" fn enum_window(window: HWND, ref_worker_w: LPARAM) -> BOOL {
    unsafe {
        let shell_dll_def_view = WindowsAndMessaging::FindWindowExA(
            Some(window),
            Some(HWND::default()),
            s!("SHELLDLL_DefView"),
            None,
        )
        .unwrap_or(HWND::default());

        if !HWND::is_invalid(&shell_dll_def_view) {
            let worker_w =
                WindowsAndMessaging::FindWindowExA(Some(HWND::default()), Some(window), s!("WorkerW"), None)
                    .unwrap_or(HWND::default());

            if !HWND::is_invalid(&worker_w) {
                *(ref_worker_w.0 as *mut HWND) = worker_w;
            }
        }

        BOOL(1)
    }
}

pub fn attach<R: tauri::Runtime>(webview_window: tauri::WebviewWindow<R>) -> crate::Result<()> {
    let hwnd = webview_window.hwnd().unwrap();

    unsafe {
        let progman_hwnd = WindowsAndMessaging::FindWindowA(s!("Progman"), None).unwrap();

        WindowsAndMessaging::SendMessageTimeoutA(
            progman_hwnd,
            0x052C,
            WPARAM(0xD),
            LPARAM(0x1),
            WindowsAndMessaging::SMTO_NORMAL,
            1000,
            None,
        );

        let mut worker_w: HWND = HWND::default();

        WindowsAndMessaging::EnumWindows(
            Some(enum_window),
            LPARAM(&mut worker_w as *mut HWND as isize),
        )
        .unwrap();

        if HWND::is_invalid(&worker_w) {
            worker_w = WindowsAndMessaging::FindWindowExA(
                Some(progman_hwnd),
                Some(HWND::default()),
                s!("WorkerW"),
                None,
            ).unwrap();
        }

        WindowsAndMessaging::SetParent(hwnd, Some(worker_w)).unwrap();
    }

    Ok(())
}
