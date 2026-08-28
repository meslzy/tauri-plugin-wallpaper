//! Raw-input forwarding to wallpaper windows.
//!
//! Wallpaper windows sit under the desktop layer and receive no input.
//! The helper window sinks global raw input (`RIDEV_INPUTSINK`) and this
//! module translates it into legacy mouse/keyboard messages posted to each
//! registered wallpaper window. Translation ported from the author's
//! proven implementation in meslzy/electron-as-wallpaper (src/input.rs).
//!
//! HWNDs are stored as `isize` because `HWND` is a raw pointer wrapper and
//! not `Send`; windows are only ever reparented (never recreated) so the
//! handles stay valid across explorer restarts. A failed `PostMessageA` is
//! the liveness signal: the target is pruned.

use std::sync::{LazyLock, Mutex};
use std::time::{Duration, Instant};

use windows::Win32::Foundation::{HWND, LPARAM, POINT, WPARAM};
use windows::Win32::Graphics::Gdi::ScreenToClient;
use windows::Win32::UI::Input::{
    GetRawInputData, HRAWINPUT, RAWINPUT, RAWINPUTHEADER, RID_DEVICE_INFO_TYPE, RID_INPUT,
    RIM_TYPEKEYBOARD, RIM_TYPEMOUSE,
};
use windows::Win32::UI::WindowsAndMessaging::{
    self, EnumChildWindows, GetAncestor, GetClassNameA, GetForegroundWindow, IsWindow,
    PostMessageA, GA_ROOT,
};

use super::helper;

#[derive(Debug, Clone)]
pub(super) struct ForwardTarget {
    /// The tauri window label, used to request real focus.
    pub label: String,
    /// The top-level tao window.
    pub host: isize,
    /// The window messages are posted to: the WebView2 input child
    /// (`Chrome_WidgetWin_1`) when present, else the host.
    pub input: isize,
    pub mouse: bool,
    pub keyboard: bool,
}

type FocusFn = std::sync::Arc<dyn Fn(&str) + Send + Sync + 'static>;

static TARGETS: LazyLock<Mutex<Vec<ForwardTarget>>> = LazyLock::new(|| Mutex::new(Vec::new()));
static LAST_MOUSE_MOVE: Mutex<Option<Instant>> = Mutex::new(None);
static LAST_FOCUS_REQUEST: Mutex<Option<Instant>> = Mutex::new(None);
static FOCUS_CALLBACK: LazyLock<Mutex<Option<FocusFn>>> = LazyLock::new(|| Mutex::new(None));

/// Registered in `desktop::init`; gives a wallpaper window REAL focus via
/// tauri (non-blocking — tao posts the activation to the event loop).
/// Chromium/WebView2 drops keyboard events unless the window genuinely has
/// focus (TSF checks real thread focus, so a posted WM_SETFOCUS is not
/// enough); mouse events are hit-tested by coordinates and don't care.
pub(crate) fn set_focus_callback(callback: FocusFn) {
    *FOCUS_CALLBACK.lock().unwrap() = Some(callback);
}

/// Posted WM_MOUSEMOVEs are queued individually (unlike real hardware
/// moves, which Windows coalesces), so cap the synthetic rate.
const MOUSE_MOVE_MIN_INTERVAL: Duration = Duration::from_millis(4);

fn hwnd(value: isize) -> HWND {
    HWND(value as *mut core::ffi::c_void)
}

/// Forward only while the desktop is focused — the same rule Lively
/// applies. Without this, every mouse move anywhere in the system floods
/// the wallpaper window's thread with synthetic messages while the user is
/// working in other apps.
///
/// The check uses the foreground window's ROOT ancestor: attached
/// wallpaper windows are children of WorkerW, so when the wallpaper window
/// (or its webview) has focus — which is what happens when the user clicks
/// the "desktop" once a wallpaper covers it — the root resolves to WorkerW
/// and forwarding stays active.
fn is_desktop_foreground() -> bool {
    unsafe {
        let foreground = GetForegroundWindow();
        if HWND::is_invalid(&foreground) {
            return false;
        }
        let root = GetAncestor(foreground, GA_ROOT);
        let window = if HWND::is_invalid(&root) {
            foreground
        } else {
            root
        };
        let mut class = [0u8; 16];
        let len = GetClassNameA(window, &mut class) as usize;
        matches!(&class[..len], b"Progman" | b"WorkerW")
    }
}

/// Finds a descendant window by class, at any depth (`EnumChildWindows`
/// recurses through the whole subtree).
fn find_descendant_by_class(root: HWND, class: &[u8]) -> Option<HWND> {
    struct Search<'a> {
        class: &'a [u8],
        found: isize,
    }

    unsafe extern "system" fn visit(child: HWND, lparam: LPARAM) -> windows::core::BOOL {
        let search = unsafe { &mut *(lparam.0 as *mut Search) };
        let mut buffer = [0u8; 64];
        let len = unsafe { GetClassNameA(child, &mut buffer) } as usize;
        if &buffer[..len] == search.class {
            search.found = child.0 as isize;
            return windows::core::BOOL(0);
        }
        windows::core::BOOL(1)
    }

    let mut search = Search { class, found: 0 };
    unsafe {
        let _ = EnumChildWindows(
            Some(root),
            Some(visit),
            LPARAM(&mut search as *mut Search as isize),
        );
    }
    (search.found != 0).then(|| hwnd(search.found))
}

/// Resolves the window that actually accepts input messages. In a Tauri
/// window the WebView2 chain is nested (`Tauri Window -> WRY_WEBVIEW ->
/// Chrome_WidgetWin_0 -> Chrome_WidgetWin_1`), and messages posted to the
/// top-level host never reach the web content — Lively targets the same
/// Chromium window for its WebView2 wallpapers.
fn resolve_input_hwnd(host: HWND) -> HWND {
    find_descendant_by_class(host, b"Chrome_WidgetWin_1")
        .or_else(|| find_descendant_by_class(host, b"Chrome_RenderWidgetHostHWND"))
        .unwrap_or(host)
}

/// Upserts a forwarding target and makes sure the helper's raw-input sink
/// is (or becomes) active.
pub(super) fn register_target(
    label: &str,
    host: isize,
    mouse: bool,
    keyboard: bool,
) -> crate::Result<()> {
    let helper = helper::ensure_helper()?;
    let input = resolve_input_hwnd(hwnd(host)).0 as isize;

    {
        let mut targets = TARGETS.lock().unwrap();
        if let Some(target) = targets.iter_mut().find(|t| t.host == host) {
            target.label = label.to_string();
            target.input = input;
            target.mouse = mouse;
            target.keyboard = keyboard;
        } else {
            targets.push(ForwardTarget {
                label: label.to_string(),
                host,
                input,
                mouse,
                keyboard,
            });
        }
    }

    unsafe {
        let _ = PostMessageA(
            Some(hwnd(helper)),
            helper::WM_APP_ENABLE_RAWINPUT,
            WPARAM(0),
            LPARAM(0),
        );
    }

    Ok(())
}

pub(super) fn unregister_target(host: isize) {
    let mut targets = TARGETS.lock().unwrap();
    targets.retain(|t| t.host != host);
}

/// Whether the current foreground window is (a child of) one of our
/// forwarding targets — i.e. the wallpaper window itself has real focus.
fn foreground_is_target() -> bool {
    let foreground = unsafe { GetForegroundWindow() };
    if HWND::is_invalid(&foreground) {
        return false;
    }
    let targets = TARGETS.lock().unwrap();
    let mut current = foreground;
    for _ in 0..12 {
        if targets.iter().any(|t| t.host == current.0 as isize) {
            return true;
        }
        let parent = unsafe { GetAncestor(current, WindowsAndMessaging::GA_PARENT) };
        if HWND::is_invalid(&parent) || parent == current {
            return false;
        }
        current = parent;
    }
    false
}

/// Requests REAL focus for the first keyboard target (throttled). Runs on
/// the helper thread; the callback only posts to the event loop, so the
/// never-block invariant holds.
fn request_keyboard_focus() {
    {
        let mut last = LAST_FOCUS_REQUEST.lock().unwrap();
        if let Some(previous) = *last {
            if previous.elapsed() < Duration::from_millis(1000) {
                return;
            }
        }
        *last = Some(Instant::now());
    }

    let label = {
        let targets = TARGETS.lock().unwrap();
        targets.iter().find(|t| t.keyboard).map(|t| t.label.clone())
    };
    let Some(label) = label else { return };

    let callback = FOCUS_CALLBACK.lock().unwrap().clone();
    if let Some(callback) = callback {
        callback(&label);
    }
}

/// Posts to the target's input window; on failure re-resolves the WebView2
/// child once (it can be recreated), pruning only when the host window
/// itself is gone. Returns false when the target should be dropped.
fn post_to_target(target: &mut ForwardTarget, msg: u32, w_param: WPARAM, l_param: LPARAM) -> bool {
    unsafe {
        if PostMessageA(Some(hwnd(target.input)), msg, w_param, l_param).is_ok() {
            return true;
        }
        if !IsWindow(Some(hwnd(target.host))).as_bool() {
            return false;
        }
        target.input = resolve_input_hwnd(hwnd(target.host)).0 as isize;
        PostMessageA(Some(hwnd(target.input)), msg, w_param, l_param).is_ok()
            || IsWindow(Some(hwnd(target.host))).as_bool()
    }
}

fn make_wparam(low: usize, high: usize) -> WPARAM {
    WPARAM((high << 16) | low)
}

fn make_lparam(low: isize, high: isize) -> LPARAM {
    LPARAM((high << 16) | (low & 0xFFFF))
}

fn send_mouse_input(msg: u32, point: POINT, w_param: WPARAM) {
    let mut targets = TARGETS.lock().unwrap();
    targets.retain_mut(|target| {
        if !target.mouse {
            return true;
        }

        let mut client_point = point;
        unsafe {
            let _ = ScreenToClient(hwnd(target.input), &mut client_point);
        }
        let l_param = make_lparam(client_point.x as isize, client_point.y as isize);
        post_to_target(target, msg, w_param, l_param)
    });
}

fn handle_mouse_input(raw_data: &RAWINPUT) {
    let mouse_data = unsafe { raw_data.data.mouse };
    let mouse_button_flags = unsafe { mouse_data.Anonymous.Anonymous.usButtonFlags } as u32;

    let mut point = POINT::default();
    unsafe {
        if WindowsAndMessaging::GetCursorPos(&mut point).is_err() {
            return;
        }
    }

    if mouse_button_flags & WindowsAndMessaging::RI_MOUSE_BUTTON_1_DOWN != 0 {
        send_mouse_input(
            WindowsAndMessaging::WM_LBUTTONDOWN,
            point,
            WPARAM::default(),
        );
        return;
    }
    if mouse_button_flags & WindowsAndMessaging::RI_MOUSE_BUTTON_1_UP != 0 {
        send_mouse_input(WindowsAndMessaging::WM_LBUTTONUP, point, WPARAM::default());
        return;
    }
    if mouse_button_flags & WindowsAndMessaging::RI_MOUSE_BUTTON_2_DOWN != 0 {
        send_mouse_input(
            WindowsAndMessaging::WM_RBUTTONDOWN,
            point,
            WPARAM::default(),
        );
        return;
    }
    if mouse_button_flags & WindowsAndMessaging::RI_MOUSE_BUTTON_2_UP != 0 {
        send_mouse_input(WindowsAndMessaging::WM_RBUTTONUP, point, WPARAM::default());
        return;
    }
    if mouse_button_flags & WindowsAndMessaging::RI_MOUSE_BUTTON_3_DOWN != 0 {
        send_mouse_input(
            WindowsAndMessaging::WM_MBUTTONDOWN,
            point,
            WPARAM::default(),
        );
        return;
    }
    if mouse_button_flags & WindowsAndMessaging::RI_MOUSE_BUTTON_3_UP != 0 {
        send_mouse_input(WindowsAndMessaging::WM_MBUTTONUP, point, WPARAM::default());
        return;
    }
    if mouse_button_flags & WindowsAndMessaging::RI_MOUSE_BUTTON_4_DOWN != 0 {
        send_mouse_input(
            WindowsAndMessaging::WM_XBUTTONDOWN,
            point,
            make_wparam(0, WindowsAndMessaging::XBUTTON1 as usize),
        );
        return;
    }
    if mouse_button_flags & WindowsAndMessaging::RI_MOUSE_BUTTON_4_UP != 0 {
        send_mouse_input(
            WindowsAndMessaging::WM_XBUTTONUP,
            point,
            make_wparam(0, WindowsAndMessaging::XBUTTON1 as usize),
        );
        return;
    }
    if mouse_button_flags & WindowsAndMessaging::RI_MOUSE_BUTTON_5_DOWN != 0 {
        send_mouse_input(
            WindowsAndMessaging::WM_XBUTTONDOWN,
            point,
            make_wparam(0, WindowsAndMessaging::XBUTTON2 as usize),
        );
        return;
    }
    if mouse_button_flags & WindowsAndMessaging::RI_MOUSE_BUTTON_5_UP != 0 {
        send_mouse_input(
            WindowsAndMessaging::WM_XBUTTONUP,
            point,
            make_wparam(0, WindowsAndMessaging::XBUTTON2 as usize),
        );
        return;
    }
    if mouse_button_flags
        & (WindowsAndMessaging::RI_MOUSE_HWHEEL | WindowsAndMessaging::RI_MOUSE_WHEEL)
        != 0
    {
        return;
    }

    // Rate-cap the remaining plain moves.
    {
        let mut last = LAST_MOUSE_MOVE.lock().unwrap();
        if let Some(previous) = *last {
            if previous.elapsed() < MOUSE_MOVE_MIN_INTERVAL {
                return;
            }
        }
        *last = Some(Instant::now());
    }

    send_mouse_input(WindowsAndMessaging::WM_MOUSEMOVE, point, WPARAM::default());
}

fn send_keyboard_input(msg: u32, w_param: WPARAM, l_param: LPARAM) {
    // With real focus, the OS already delivers keyboard input to the
    // wallpaper window — forwarding on top would double every key.
    if foreground_is_target() {
        return;
    }

    let mut targets = TARGETS.lock().unwrap();
    targets.retain_mut(|target| {
        if !target.keyboard {
            return true;
        }
        post_to_target(target, msg, w_param, l_param)
    });
}

// ref: https://learn.microsoft.com/en-us/windows/win32/inputdev/wm-keydown#parameters
fn keyboard_lparam(flags: u16, make_code: u16) -> LPARAM {
    // 0 = key down, 1 = key up
    let is_pressed = (flags as u32 & WindowsAndMessaging::RI_KEY_BREAK) == 0;

    // Repeat count 0 on keydown, 1 on keyup.
    let repeat_count: u32 = if is_pressed { 0 } else { 1 };
    let scan_code = make_code as u32;
    let is_extended: u32 = if scan_code > 0x80 { 1 } else { 0 };
    // Always 0 for WM_KEYDOWN.
    let context_code: u32 = 0;
    let previous_key_state: u32 = if is_pressed { 1 } else { 0 };
    let transition_state: u32 = if is_pressed { 0 } else { 1 };

    // repeat_count: bits 0-15, scan_code: 16-23, extended: 24,
    // context: 29, previous state: 30, transition: 31
    let l_param = repeat_count
        | (scan_code << 16)
        | (is_extended << 24)
        | (context_code << 29)
        | (previous_key_state << 30)
        | (transition_state << 31);

    LPARAM(l_param as isize)
}

fn handle_keyboard_input(raw_data: &RAWINPUT) {
    let keyboard_data = unsafe { raw_data.data.keyboard };

    let message = keyboard_data.Message;
    let key_code = keyboard_data.VKey;
    let flags = keyboard_data.Flags;
    let make_code = keyboard_data.MakeCode;

    let w_param = WPARAM(key_code as usize);
    let l_param = keyboard_lparam(flags, make_code);

    send_keyboard_input(message, w_param, l_param);
}

/// Runs on the helper thread for every `WM_INPUT`.
pub(super) fn handle_wm_input(l_param: LPARAM) {
    // Same rule as Lively: only forward while the desktop is focused.
    if !is_desktop_foreground() {
        return;
    }

    // While on the desktop, keep a keyboard-forwarding wallpaper window
    // genuinely focused: Chromium only accepts key events with real focus.
    // Once it IS focused, real keyboard input flows to it natively, so
    // forwarding skips keys to avoid doubled events (see
    // send_keyboard_input).
    if !foreground_is_target() {
        request_keyboard_focus();
    }

    let mut raw_data = RAWINPUT::default();
    let mut raw_data_size = std::mem::size_of::<RAWINPUT>() as u32;
    let raw_data_header_size = std::mem::size_of::<RAWINPUTHEADER>() as u32;

    let bytes_copied = unsafe {
        GetRawInputData(
            HRAWINPUT(l_param.0 as _),
            RID_INPUT,
            Some(&mut raw_data as *mut _ as *mut _),
            &mut raw_data_size,
            raw_data_header_size,
        )
    };

    if bytes_copied == u32::MAX {
        return;
    }

    let device_type = RID_DEVICE_INFO_TYPE(raw_data.header.dwType);
    if device_type == RIM_TYPEMOUSE {
        handle_mouse_input(&raw_data);
    } else if device_type == RIM_TYPEKEYBOARD {
        handle_keyboard_input(&raw_data);
    }
}
