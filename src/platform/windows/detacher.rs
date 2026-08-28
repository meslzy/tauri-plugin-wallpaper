use windows::Win32::Graphics::Dwm::DWMWCP_DEFAULT;
use windows::Win32::UI::WindowsAndMessaging;

pub fn detach<R: tauri::Runtime>(webview_window: &tauri::WebviewWindow<R>) -> crate::Result<()> {
    let hwnd = webview_window.hwnd()?;

    unsafe {
        WindowsAndMessaging::SetParent(hwnd, None)?;
    }

    super::corners::set_corner_preference(hwnd, DWMWCP_DEFAULT);
    super::reattach::note_detached(webview_window.label());
    super::input::unregister_target(hwnd.0 as isize);

    Ok(())
}
