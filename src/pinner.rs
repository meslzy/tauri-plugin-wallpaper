use std::collections::HashMap;
use std::sync::Mutex;

use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::{
        self, CallWindowProcW, SetWindowLongPtrW, SetWindowPos, GWLP_WNDPROC, HWND_TOPMOST,
        SWP_NOMOVE, SWP_NOSIZE, WINDOWPOS, WM_WINDOWPOSCHANGING, WNDPROC,
    },
};

static ORIGINAL_WNDPROCS: Mutex<Option<HashMap<isize, isize>>> = Mutex::new(None);

fn get_original_wndproc(hwnd: HWND) -> Option<WNDPROC> {
    let map = ORIGINAL_WNDPROCS.lock().unwrap();
    if let Some(ref m) = *map {
        m.get(&(hwnd.0 as isize))
            .map(|&ptr| unsafe { std::mem::transmute(ptr) })
    } else {
        None
    }
}

fn set_original_wndproc(hwnd: HWND, wndproc: isize) {
    let mut map = ORIGINAL_WNDPROCS.lock().unwrap();
    if map.is_none() {
        *map = Some(HashMap::new());
    }
    if let Some(ref mut m) = *map {
        m.insert(hwnd.0 as isize, wndproc);
    }
}

pub fn remove_original_wndproc(hwnd: HWND) -> Option<isize> {
    let mut map = ORIGINAL_WNDPROCS.lock().unwrap();
    if let Some(ref mut m) = *map {
        m.remove(&(hwnd.0 as isize))
    } else {
        None
    }
}

pub fn is_pinned(hwnd: HWND) -> bool {
    let map = ORIGINAL_WNDPROCS.lock().unwrap();
    if let Some(ref m) = *map {
        m.contains_key(&(hwnd.0 as isize))
    } else {
        false
    }
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

pub fn pin<R: tauri::Runtime>(webview_window: tauri::WebviewWindow<R>) -> crate::Result<()> {
    let hwnd = webview_window.hwnd().unwrap();

    if is_pinned(hwnd) {
        return Ok(());
    }

    unsafe {
        SetWindowPos(
            hwnd,
            Some(HWND_TOPMOST),
            0,
            0,
            0,
            0,
            SWP_NOMOVE | SWP_NOSIZE,
        )
        .unwrap();

        let original_wndproc =
            SetWindowLongPtrW(hwnd, GWLP_WNDPROC, pin_wndproc as *const () as isize);

        set_original_wndproc(hwnd, original_wndproc);
    }

    Ok(())
}
