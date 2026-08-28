use windows::{
    core::{s, BOOL},
    Win32::{
        Foundation::{HWND, LPARAM, POINT, RECT, WPARAM},
        Graphics::Gdi::ClientToScreen,
        UI::WindowsAndMessaging::{
            self, GetClientRect, GetSystemMetrics, GetWindowRect, SetWindowPos,
            SM_XVIRTUALSCREEN, SM_YVIRTUALSCREEN, SWP_NOACTIVATE, SWP_NOZORDER,
        },
    },
};

use crate::models::AttachOptions;

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
            let worker_w: HWND = WindowsAndMessaging::FindWindowExA(
                Some(HWND::default()),
                Some(window),
                s!("WorkerW"),
                None,
            )
            .unwrap_or(HWND::default());
            if !HWND::is_invalid(&worker_w) {
                *(ref_worker_w.0 as *mut HWND) = worker_w;
            }
        }

        BOOL(1)
    }
}

/// Finds (or spawns) the WorkerW desktop layer. Retries a few times: right
/// after boot or an explorer restart the layer may not exist yet.
fn find_worker_w() -> crate::Result<HWND> {
    unsafe {
        let progman_hwnd = WindowsAndMessaging::FindWindowA(s!("Progman"), None)
            .map_err(|_| crate::Error::ProgmanWindowNotFound)?;

        for attempt in 0..3 {
            if attempt > 0 {
                std::thread::sleep(std::time::Duration::from_millis(100));
            }

            // Ask Progman to spawn the WorkerW layer between wallpaper and icons.
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
            let _ = WindowsAndMessaging::EnumWindows(
                Some(enum_window),
                LPARAM(&mut worker_w as *mut HWND as isize),
            );

            if HWND::is_invalid(&worker_w) {
                // Windows 11 24H2+: WorkerW is a child of Progman instead of a sibling
                worker_w = WindowsAndMessaging::FindWindowExA(
                    Some(progman_hwnd),
                    Some(HWND::default()),
                    s!("WorkerW"),
                    None,
                )
                .unwrap_or(HWND::default());
            }

            if !HWND::is_invalid(&worker_w) {
                return Ok(worker_w);
            }
        }

        Err(crate::Error::WorkerWindowNotFound)
    }
}

pub fn attach<R: tauri::Runtime>(
    webview_window: &tauri::WebviewWindow<R>,
    options: AttachOptions,
) -> crate::Result<()> {
    let hwnd = webview_window.hwnd()?;
    let worker_w = find_worker_w()?;

    unsafe {
        WindowsAndMessaging::SetParent(hwnd, Some(worker_w))?;

        // Avoid Windows 11 rounded-corner gaps on the wallpaper layer.
        super::corners::set_corner_preference(
            hwnd,
            windows::Win32::Graphics::Dwm::DWMWCP_DONOTROUND,
        );

        // Child coordinates are relative to WorkerW, which spans the whole
        // virtual screen — translate the monitor rect accordingly.
        if let Some(rect) = options.monitor_rect {
            let virtual_x = GetSystemMetrics(SM_XVIRTUALSCREEN);
            let virtual_y = GetSystemMetrics(SM_YVIRTUALSCREEN);
            let x = rect.x - virtual_x;
            let y = rect.y - virtual_y;
            SetWindowPos(
                hwnd,
                None,
                x,
                y,
                rect.width as i32,
                rect.height as i32,
                SWP_NOZORDER | SWP_NOACTIVATE,
            )?;

            // Undecorated-but-resizable tauri windows keep an invisible
            // resize frame (WM_NCCALCSIZE insets, typically 8px on
            // left/right/bottom), which would leave a black stripe at the
            // monitor edge. Expand the window so the CLIENT area — what
            // the webview actually paints — covers the monitor exactly.
            // (`resizable: false` windows have no insets; this is a no-op.)
            let mut window_rect = RECT::default();
            let mut client_rect = RECT::default();
            let mut client_origin = POINT::default();
            GetWindowRect(hwnd, &mut window_rect)?;
            GetClientRect(hwnd, &mut client_rect)?;
            let _ = ClientToScreen(hwnd, &mut client_origin);

            let inset_left = client_origin.x - window_rect.left;
            let inset_top = client_origin.y - window_rect.top;
            let inset_right =
                (window_rect.right - window_rect.left) - client_rect.right - inset_left;
            let inset_bottom =
                (window_rect.bottom - window_rect.top) - client_rect.bottom - inset_top;

            if inset_left != 0 || inset_top != 0 || inset_right != 0 || inset_bottom != 0 {
                SetWindowPos(
                    hwnd,
                    None,
                    x - inset_left,
                    y - inset_top,
                    rect.width as i32 + inset_left + inset_right,
                    rect.height as i32 + inset_top + inset_bottom,
                    SWP_NOZORDER | SWP_NOACTIVATE,
                )?;
            }
        }
    }

    super::helper::ensure_helper()?;
    super::reattach::note_attached(webview_window.label(), options);

    if options.forward_mouse_input || options.forward_keyboard_input {
        super::input::register_target(
            webview_window.label(),
            hwnd.0 as isize,
            options.forward_mouse_input,
            options.forward_keyboard_input,
        )?;
    } else {
        // Re-attach with forwarding turned off must clear an old registration.
        super::input::unregister_target(hwnd.0 as isize);
    }

    Ok(())
}
