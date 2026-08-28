use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::{
        self, CallWindowProcW, SetWindowLongPtrW, SetWindowPos, GWLP_WNDPROC, HWND_TOPMOST,
        SWP_NOMOVE, SWP_NOSIZE, WINDOWPOS, WM_WINDOWPOSCHANGING, WNDPROC,
    },
};

static ORIGINAL_WNDPROCS: LazyLock<Mutex<HashMap<isize, isize>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

fn get_original_wndproc(hwnd: HWND) -> Option<WNDPROC> {
    let map = ORIGINAL_WNDPROCS.lock().unwrap();
    map.get(&(hwnd.0 as isize))
        .map(|&ptr| unsafe { std::mem::transmute(ptr) })
}

fn set_original_wndproc(hwnd: HWND, wndproc: isize) {
    let mut map = ORIGINAL_WNDPROCS.lock().unwrap();
    map.insert(hwnd.0 as isize, wndproc);
}

pub fn remove_original_wndproc(hwnd: HWND) -> Option<isize> {
    let mut map = ORIGINAL_WNDPROCS.lock().unwrap();
    map.remove(&(hwnd.0 as isize))
}

pub fn is_pinned(hwnd: HWND) -> bool {
    let map = ORIGINAL_WNDPROCS.lock().unwrap();
    map.contains_key(&(hwnd.0 as isize))
}

unsafe extern "system" fn pin_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WM_WINDOWPOSCHANGING {
        let pos = lparam.0 as *mut WINDOWPOS;
        if !pos.is_null() {
            let pos_ref = &mut *pos;
            if pos_ref.x == -32000 {
                pos_ref.flags |= SWP_NOMOVE | SWP_NOSIZE;
            }
            pos_ref.hwndInsertAfter = HWND_TOPMOST;
        }
    }

    if let Some(original) = get_original_wndproc(hwnd) {
        CallWindowProcW(original, hwnd, msg, wparam, lparam)
    } else {
        WindowsAndMessaging::DefWindowProcW(hwnd, msg, wparam, lparam)
    }
}

pub fn pin<R: tauri::Runtime>(webview_window: &tauri::WebviewWindow<R>) -> crate::Result<()> {
    let hwnd = webview_window.hwnd()?;

    if is_pinned(hwnd) {
        return Ok(());
    }

    unsafe {
        let original_wndproc =
            SetWindowLongPtrW(hwnd, GWLP_WNDPROC, pin_wndproc as *const () as isize);

        if original_wndproc == 0 {
            return Err(crate::Error::SubclassFailed);
        }

        set_original_wndproc(hwnd, original_wndproc);

        if let Err(error) = SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE,
        ) {
            if let Some(original) = remove_original_wndproc(hwnd) {
                SetWindowLongPtrW(hwnd, GWLP_WNDPROC, original);
            }
            return Err(error.into());
        }
    }

    Ok(())
}
