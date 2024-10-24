use windows::Win32::UI::WindowsAndMessaging;

pub fn detach<R: tauri::Runtime>(webview_window: tauri::WebviewWindow<R>) -> crate::Result<()> {
    let hwnd = webview_window.hwnd().unwrap();

    unsafe {
        WindowsAndMessaging::SetParent(hwnd, None).unwrap();
    }

    Ok(())
}
