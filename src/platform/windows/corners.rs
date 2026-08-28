use windows::Win32::Foundation::HWND;
use windows::Win32::Graphics::Dwm::{
    DwmSetWindowAttribute, DWMWA_WINDOW_CORNER_PREFERENCE, DWM_WINDOW_CORNER_PREFERENCE,
};

/// Windows 11 rounds the corners of top-level windows, which leaves visible
/// gaps once a window is reparented under WorkerW as wallpaper
/// (https://github.com/Charlie-XIAO/tauri-plugin-desktop-underlay/issues/85).
/// Best effort: the attribute doesn't exist before Windows 11, where corners
/// are square anyway.
pub fn set_corner_preference(hwnd: HWND, preference: DWM_WINDOW_CORNER_PREFERENCE) {
    unsafe {
        let _ = DwmSetWindowAttribute(
            hwnd,
            DWMWA_WINDOW_CORNER_PREFERENCE,
            &preference as *const _ as *const core::ffi::c_void,
            std::mem::size_of::<DWM_WINDOW_CORNER_PREFERENCE>() as u32,
        );
    }
}
