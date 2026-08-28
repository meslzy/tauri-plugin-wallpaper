//! macOS implementation.
//!
//! Window layering uses documented AppKit/CoreGraphics APIs:
//! - Wallpaper mode: window level just below the desktop level, so the
//!   window sits behind desktop icons (desktop icons live at the
//!   `desktopIconWindow` level, far above `desktopWindow`).
//! - Pin mode: floating window level (what always-on-top uses) plus
//!   collection behaviors so the window stays put across Spaces and
//!   survives Mission Control's Show Desktop.
//!
//! References:
//! - https://developer.apple.com/documentation/coregraphics/cgwindowlevelkey
//! - https://developer.apple.com/documentation/appkit/nswindowcollectionbehavior
//! - https://github.com/Charlie-XIAO/tauri-plugin-desktop-underlay (proven
//!   implementation of the underlay technique, including the requirement
//!   that all calls happen on the main thread)

use objc2::msg_send;
use objc2::runtime::AnyObject;
use std::os::raw::c_ulong;
use tauri::{Runtime, WebviewWindow};

use crate::models::{AttachOptions, Capabilities};

extern "C" {
    fn CGWindowLevelForKey(key: i32) -> i32;
}

// CGWindowLevelKey values (see CoreGraphics CGWindowLevel.h)
const DESKTOP_WINDOW_LEVEL_KEY: i32 = 2;
const NORMAL_WINDOW_LEVEL_KEY: i32 = 4;
const FLOATING_WINDOW_LEVEL_KEY: i32 = 5;

// NSWindowCollectionBehavior bits
const CAN_JOIN_ALL_SPACES: c_ulong = 1 << 0;
const STATIONARY: c_ulong = 1 << 4;
const IGNORES_CYCLE: c_ulong = 1 << 6;

const WALLPAPER_BEHAVIOR: c_ulong = CAN_JOIN_ALL_SPACES | STATIONARY | IGNORES_CYCLE;
const PIN_BEHAVIOR: c_ulong = CAN_JOIN_ALL_SPACES | STATIONARY;

fn ns_window<R: Runtime>(webview_window: &WebviewWindow<R>) -> crate::Result<*mut AnyObject> {
    Ok(webview_window.ns_window()? as *mut AnyObject)
}

unsafe fn set_level(ns_window: *mut AnyObject, level_key: i32, offset: i32) {
    let level = (CGWindowLevelForKey(level_key) + offset) as isize;
    let () = msg_send![ns_window, setLevel: level];
}

unsafe fn add_behavior(ns_window: *mut AnyObject, bits: c_ulong) {
    let behavior: c_ulong = msg_send![ns_window, collectionBehavior];
    let () = msg_send![ns_window, setCollectionBehavior: behavior | bits];
}

unsafe fn remove_behavior(ns_window: *mut AnyObject, bits: c_ulong) {
    let behavior: c_ulong = msg_send![ns_window, collectionBehavior];
    let () = msg_send![ns_window, setCollectionBehavior: behavior & !bits];
}

pub fn attach<R: Runtime>(
    webview_window: &WebviewWindow<R>,
    _options: AttachOptions,
) -> crate::Result<()> {
    let ns_window = ns_window(webview_window)?;
    unsafe {
        // One below the desktop level: behind icons, above the wallpaper.
        set_level(ns_window, DESKTOP_WINDOW_LEVEL_KEY, -1);
        add_behavior(ns_window, WALLPAPER_BEHAVIOR);
    }
    Ok(())
}

pub fn detach<R: Runtime>(webview_window: &WebviewWindow<R>) -> crate::Result<()> {
    let ns_window = ns_window(webview_window)?;
    unsafe {
        set_level(ns_window, NORMAL_WINDOW_LEVEL_KEY, 0);
        remove_behavior(ns_window, WALLPAPER_BEHAVIOR);
    }
    Ok(())
}

pub fn pin<R: Runtime>(webview_window: &WebviewWindow<R>) -> crate::Result<()> {
    let ns_window = ns_window(webview_window)?;
    unsafe {
        set_level(ns_window, FLOATING_WINDOW_LEVEL_KEY, 0);
        add_behavior(ns_window, PIN_BEHAVIOR);
    }
    Ok(())
}

pub fn unpin<R: Runtime>(webview_window: &WebviewWindow<R>) -> crate::Result<()> {
    let ns_window = ns_window(webview_window)?;
    unsafe {
        set_level(ns_window, NORMAL_WINDOW_LEVEL_KEY, 0);
        remove_behavior(ns_window, PIN_BEHAVIOR);
    }
    Ok(())
}

pub fn reset() -> crate::Result<()> {
    // Only needed on Windows, where detaching from WorkerW can leave a
    // stale wallpaper frame. macOS never modifies the actual wallpaper.
    Err(crate::Error::Unsupported {
        feature: "reset",
        reason: "resetting the wallpaper is only needed on Windows",
    })
}

/// Plash's "browsing mode" pattern: temporarily raise the wallpaper window
/// to the normal level so the user can interact with it, then send it back.
pub fn set_interactive<R: Runtime>(
    webview_window: &WebviewWindow<R>,
    interactive: bool,
) -> crate::Result<()> {
    let ns_window = ns_window(webview_window)?;
    unsafe {
        if interactive {
            set_level(ns_window, NORMAL_WINDOW_LEVEL_KEY, 0);
        } else {
            set_level(ns_window, DESKTOP_WINDOW_LEVEL_KEY, -1);
        }
    }
    Ok(())
}

/// `NSWindowOcclusionStateVisible` is `1 << 1`; macOS 26 sometimes reports
/// visibility with an undocumented `0x2000` bit instead, so accept either.
pub fn is_occluded<R: Runtime>(webview_window: &WebviewWindow<R>) -> crate::Result<bool> {
    let ns_window = ns_window(webview_window)?;
    let state: c_ulong = unsafe { msg_send![ns_window, occlusionState] };
    Ok((state & 0x2002) == 0)
}

pub fn window_destroyed(_label: &str) {}

pub fn capabilities() -> Capabilities {
    Capabilities {
        platform: "macos",
        attach: true,
        detach: true,
        reset: false,
        pin: true,
        unpin: true,
        input_forwarding: false,
        interactive: true,
        occlusion: true,
        wallpaper_image: true,
    }
}
