//! Hidden helper window on a dedicated thread.
//!
//! Hosts two responsibilities that both need a Win32 message pump:
//! - receiving raw input (`WM_INPUT`) for input forwarding ([`super::input`])
//! - receiving the `TaskbarCreated` broadcast when explorer.exe restarts,
//!   which destroys WorkerW and drops wallpaper windows ([`super::reattach`])
//!
//! The window must be a REAL hidden top-level window: message-only
//! (`HWND_MESSAGE`) windows never receive `RegisterWindowMessage`
//! broadcasts. `WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE` keeps it out of
//! alt-tab and it is never shown.
//!
//! INVARIANT: the window procedure must never block and never call
//! `SendMessage` toward other threads — `PostMessageA` only. Blocking here
//! can deadlock against the main thread's own sends.

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::{mpsc, LazyLock, Mutex};

use windows::core::s;
use windows::Win32::Devices::HumanInterfaceDevice::{
    HID_USAGE_GENERIC_KEYBOARD, HID_USAGE_GENERIC_MOUSE, HID_USAGE_PAGE_GENERIC,
};
use windows::Win32::Foundation::{
    GetLastError, ERROR_CLASS_ALREADY_EXISTS, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleA;
use windows::Win32::UI::Input::{
    RegisterRawInputDevices, RAWINPUTDEVICE, RIDEV_INPUTSINK,
};
use windows::Win32::UI::WindowsAndMessaging::{
    self, CreateWindowExA, DefWindowProcA, DispatchMessageW, GetMessageW, RegisterClassA,
    RegisterWindowMessageA, TranslateMessage, MSG, WM_APP, WNDCLASSA, WS_EX_NOACTIVATE,
    WS_EX_TOOLWINDOW, WS_POPUP,
};

/// Posted to the helper window to enable the global raw-input sink; done on
/// the helper thread because raw input is delivered to the registering
/// window's thread.
pub(super) const WM_APP_ENABLE_RAWINPUT: u32 = WM_APP + 1;

static HELPER_HWND: LazyLock<Mutex<Option<isize>>> = LazyLock::new(|| Mutex::new(None));
static TASKBAR_CREATED_MSG: AtomicU32 = AtomicU32::new(0);
static RAWINPUT_ENABLED: AtomicBool = AtomicBool::new(false);

/// Starts the helper thread on first call; idempotent afterwards.
/// Holding the lock across spawn is safe: the helper thread never touches
/// `HELPER_HWND`.
pub(super) fn ensure_helper() -> crate::Result<isize> {
    let mut cached = HELPER_HWND.lock().unwrap();
    if let Some(hwnd) = *cached {
        return Ok(hwnd);
    }

    let (ready_tx, ready_rx) = mpsc::channel();
    std::thread::Builder::new()
        .name("wallpaper-helper".into())
        .spawn(move || helper_thread_main(ready_tx))
        .map_err(|_| crate::Error::HelperThread)?;

    let hwnd = ready_rx.recv().map_err(|_| crate::Error::HelperThread)??;
    *cached = Some(hwnd);
    Ok(hwnd)
}

fn helper_thread_main(ready_tx: mpsc::Sender<crate::Result<isize>>) {
    unsafe {
        // Idempotent; returns the same message id in every process.
        let taskbar_created = RegisterWindowMessageA(s!("TaskbarCreated"));
        TASKBAR_CREATED_MSG.store(taskbar_created, Ordering::SeqCst);

        let instance = match GetModuleHandleA(None) {
            Ok(instance) => instance,
            Err(_) => {
                let _ = ready_tx.send(Err(crate::Error::HelperWindow));
                return;
            }
        };

        let class_name = s!("tauri_wallpaper_helper");
        let wnd_class = WNDCLASSA {
            lpfnWndProc: Some(helper_wndproc),
            hInstance: HINSTANCE::from(instance),
            lpszClassName: class_name,
            ..WNDCLASSA::default()
        };

        // Atom 0 with ERROR_CLASS_ALREADY_EXISTS is fine (plugin re-init).
        if RegisterClassA(&wnd_class) == 0 && GetLastError() != ERROR_CLASS_ALREADY_EXISTS {
            let _ = ready_tx.send(Err(crate::Error::HelperWindow));
            return;
        }

        let hwnd = match CreateWindowExA(
            WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
            class_name,
            s!("tauri_wallpaper_helper"),
            WS_POPUP,
            0,
            0,
            0,
            0,
            None,
            None,
            Some(HINSTANCE::from(instance)),
            None,
        ) {
            Ok(hwnd) => hwnd,
            Err(_) => {
                let _ = ready_tx.send(Err(crate::Error::HelperWindow));
                return;
            }
        };

        let _ = ready_tx.send(Ok(hwnd.0 as isize));

        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

/// Registers the global raw-input sink. Runs on the helper thread (the
/// registering thread receives the input), triggered via
/// [`WM_APP_ENABLE_RAWINPUT`]. Only ever done once, and only if some
/// window actually requested forwarding.
fn enable_raw_input(helper: HWND) {
    if RAWINPUT_ENABLED.swap(true, Ordering::SeqCst) {
        return;
    }

    let devices = [
        RAWINPUTDEVICE {
            usUsagePage: HID_USAGE_PAGE_GENERIC,
            usUsage: HID_USAGE_GENERIC_MOUSE,
            dwFlags: RIDEV_INPUTSINK,
            hwndTarget: helper,
        },
        RAWINPUTDEVICE {
            usUsagePage: HID_USAGE_PAGE_GENERIC,
            usUsage: HID_USAGE_GENERIC_KEYBOARD,
            dwFlags: RIDEV_INPUTSINK,
            hwndTarget: helper,
        },
    ];

    unsafe {
        if RegisterRawInputDevices(&devices, std::mem::size_of::<RAWINPUTDEVICE>() as u32).is_err()
        {
            RAWINPUT_ENABLED.store(false, Ordering::SeqCst);
        }
    }
}

unsafe extern "system" fn helper_wndproc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    if msg == WindowsAndMessaging::WM_INPUT {
        super::input::handle_wm_input(lparam);
        return LRESULT(0);
    }

    if msg == WM_APP_ENABLE_RAWINPUT {
        enable_raw_input(hwnd);
        return LRESULT(0);
    }

    let taskbar_created = TASKBAR_CREATED_MSG.load(Ordering::SeqCst);
    if taskbar_created != 0 && msg == taskbar_created {
        super::reattach::on_taskbar_created();
        return LRESULT(0);
    }

    DefWindowProcA(hwnd, msg, wparam, lparam)
}
