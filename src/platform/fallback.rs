//! Fallback for platforms without a desktop window layer (iOS, Android, …):
//! every operation reports Unsupported.

use tauri::{Runtime, WebviewWindow};

use crate::models::{AttachOptions, Capabilities};

fn unsupported(feature: &'static str) -> crate::Error {
    crate::Error::Unsupported {
        feature,
        reason: "this platform has no desktop window layer",
    }
}

pub fn attach<R: Runtime>(
    _webview_window: &WebviewWindow<R>,
    _options: AttachOptions,
) -> crate::Result<()> {
    Err(unsupported("attach"))
}

pub fn detach<R: Runtime>(_webview_window: &WebviewWindow<R>) -> crate::Result<()> {
    Err(unsupported("detach"))
}

pub fn pin<R: Runtime>(_webview_window: &WebviewWindow<R>) -> crate::Result<()> {
    Err(unsupported("pin"))
}

pub fn unpin<R: Runtime>(_webview_window: &WebviewWindow<R>) -> crate::Result<()> {
    Err(unsupported("unpin"))
}

pub fn reset() -> crate::Result<()> {
    Err(unsupported("reset"))
}

pub fn set_interactive<R: Runtime>(
    _webview_window: &WebviewWindow<R>,
    _interactive: bool,
) -> crate::Result<()> {
    Err(unsupported("setInteractive"))
}

pub fn is_occluded<R: Runtime>(_webview_window: &WebviewWindow<R>) -> crate::Result<bool> {
    Err(unsupported("occlusion"))
}

pub fn window_destroyed(_label: &str) {}

pub fn capabilities() -> Capabilities {
    Capabilities {
        platform: std::env::consts::OS,
        attach: false,
        detach: false,
        reset: false,
        pin: false,
        unpin: false,
        input_forwarding: false,
        interactive: false,
        occlusion: false,
        wallpaper_image: false,
    }
}
