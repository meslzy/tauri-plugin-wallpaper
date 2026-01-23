use windows::Win32::UI::WindowsAndMessaging::{
    SetWindowLongPtrW, SetWindowPos, GWLP_WNDPROC, HWND_NOTOPMOST, SWP_NOMOVE, SWP_NOSIZE,
};

use crate::pinner;

pub fn unpin<R: tauri::Runtime>(webview_window: tauri::WebviewWindow<R>) -> crate::Result<()> {
    let hwnd = webview_window.hwnd().unwrap();

    if !pinner::is_pinned(hwnd) {
        return Ok(());
    }

    unsafe {
        if let Some(original_wndproc) = pinner::remove_original_wndproc(hwnd) {
            SetWindowLongPtrW(hwnd, GWLP_WNDPROC, original_wndproc);
        }

        SetWindowPos(
            hwnd,
            Some(HWND_NOTOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE,
        )
        .unwrap();
    }

    Ok(())
}
