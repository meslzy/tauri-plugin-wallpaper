use std::ptr::null_mut;

use windows::{
  core::s,
  Win32::{
    Foundation::{
      BOOL,
      HWND,
      LPARAM,
      WPARAM
    },
    UI::WindowsAndMessaging,
  },
};

extern "system" fn enum_window(window: HWND, ref_worker_w: LPARAM) -> BOOL {
  unsafe {
    let shell_dll_def_view = WindowsAndMessaging::FindWindowExA(window, None, s!("SHELLDLL_DefView"), None);
    if shell_dll_def_view == HWND(0) {
      return true.into();
    }

    let worker_w = WindowsAndMessaging::FindWindowExA(None, window, s!("WorkerW"), None);
    if worker_w == HWND(0) {
      return true.into();
    }

    *(ref_worker_w.0 as *mut HWND) = worker_w;

    return false.into();
  }
}

pub fn attach(hwnd: HWND) {
  unsafe {
    let progman = WindowsAndMessaging::FindWindowA(s!("Progman"), None);

    WindowsAndMessaging::SendMessageA(progman, 0x052C, WPARAM(0x0000000D), LPARAM(0));
    WindowsAndMessaging::SendMessageA(progman, 0x052C, WPARAM(0x0000000D), LPARAM(1));

    let mut worker_w: HWND = HWND(0);

    WindowsAndMessaging::EnumWindows(
      Some(enum_window),
      LPARAM(&mut worker_w as *mut HWND as isize),
    );

    if worker_w == HWND(0) {
      panic!("Could not find worker_w window");
    }

    WindowsAndMessaging::SetParent(hwnd, worker_w);
  };
}

pub fn detach(hwnd: HWND) {
  unsafe {
    WindowsAndMessaging::SetParent(hwnd, None);
    WindowsAndMessaging::SystemParametersInfoA(
      WindowsAndMessaging::SPI_SETDESKWALLPAPER,
      0,
      null_mut(),
      WindowsAndMessaging::SPIF_UPDATEINIFILE,
    );
  };
}