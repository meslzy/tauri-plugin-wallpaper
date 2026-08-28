//! Linux (X11) implementation.
//!
//! - Wallpaper mode: the `_NET_WM_WINDOW_TYPE_DESKTOP` window type hint via
//!   GTK, which X11 window managers render at the desktop layer.
//! - Pin mode: `_NET_WM_STATE_ABOVE` + `_NET_WM_STATE_STICKY` via GTK's
//!   `set_keep_above` and `stick` (the same call tao uses for
//!   always-on-top; sticky keeps the window on all workspaces).
//!
//! Type hints and WM states are X11 concepts: on Wayland GTK silently
//! ignores them, so rather than pretending to succeed we detect the session
//! type and return an Unsupported error. Wayland would need the
//! wlr-layer-shell protocol, which GNOME does not implement.
//!
//! References:
//! - https://github.com/Charlie-XIAO/tauri-plugin-desktop-underlay (shipped
//!   implementation of the desktop type hint approach)
//! - https://gtk-rs.org/gtk3-rs/stable/latest/docs/gdk/enum.WindowTypeHint.html

use gdk::WindowTypeHint;
use gtk::prelude::GtkWindowExt;
use tauri::{Runtime, WebviewWindow};

use crate::models::{AttachOptions, Capabilities};

fn is_x11() -> bool {
    // GDK_BACKEND=x11 forces X11 (XWayland) even inside a Wayland session.
    if let Ok(backend) = std::env::var("GDK_BACKEND") {
        if backend.split(',').any(|b| b.trim() == "x11") {
            return true;
        }
    }
    if std::env::var("WAYLAND_DISPLAY").is_ok() {
        return false;
    }
    !matches!(std::env::var("XDG_SESSION_TYPE"), Ok(session) if session == "wayland")
}

fn ensure_x11(feature: &'static str) -> crate::Result<()> {
    if is_x11() {
        Ok(())
    } else {
        Err(crate::Error::Unsupported {
            feature,
            reason: "requires an X11 session; Wayland does not support window type hints",
        })
    }
}

pub fn attach<R: Runtime>(
    webview_window: &WebviewWindow<R>,
    _options: AttachOptions,
) -> crate::Result<()> {
    ensure_x11("attach")?;
    let gtk_window = webview_window.gtk_window()?;
    gtk_window.set_type_hint(WindowTypeHint::Desktop);
    Ok(())
}

pub fn detach<R: Runtime>(webview_window: &WebviewWindow<R>) -> crate::Result<()> {
    ensure_x11("detach")?;
    let gtk_window = webview_window.gtk_window()?;
    gtk_window.set_type_hint(WindowTypeHint::Normal);
    Ok(())
}

pub fn pin<R: Runtime>(webview_window: &WebviewWindow<R>) -> crate::Result<()> {
    ensure_x11("pin")?;
    let gtk_window = webview_window.gtk_window()?;
    gtk_window.set_keep_above(true);
    gtk_window.stick();
    Ok(())
}

pub fn unpin<R: Runtime>(webview_window: &WebviewWindow<R>) -> crate::Result<()> {
    ensure_x11("unpin")?;
    let gtk_window = webview_window.gtk_window()?;
    gtk_window.set_keep_above(false);
    gtk_window.unstick();
    Ok(())
}

pub fn reset() -> crate::Result<()> {
    Err(crate::Error::Unsupported {
        feature: "reset",
        reason: "resetting the wallpaper is only needed on Windows",
    })
}

pub fn set_interactive<R: Runtime>(
    _webview_window: &WebviewWindow<R>,
    _interactive: bool,
) -> crate::Result<()> {
    Err(crate::Error::Unsupported {
        feature: "setInteractive",
        reason: "not implemented on Linux",
    })
}

pub fn is_occluded<R: Runtime>(_webview_window: &WebviewWindow<R>) -> crate::Result<bool> {
    Err(crate::Error::Unsupported {
        feature: "occlusion",
        reason: "occlusion monitoring is not implemented on Linux",
    })
}

pub fn window_destroyed(_label: &str) {}

pub fn capabilities() -> Capabilities {
    let x11 = is_x11();
    Capabilities {
        platform: "linux",
        attach: x11,
        detach: x11,
        reset: false,
        pin: x11,
        unpin: x11,
        input_forwarding: false,
        interactive: false,
        occlusion: false,
        // The `wallpaper` crate goes through DE tools (gsettings, swaybg, …),
        // so this works on Wayland desktops too.
        wallpaper_image: true,
    }
}
