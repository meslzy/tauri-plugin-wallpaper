use tauri::{Runtime, WebviewWindow};

#[cfg(target_os = "windows")]
pub(crate) mod windows;
#[cfg(target_os = "windows")]
pub(crate) use self::windows as imp;

#[cfg(target_os = "macos")]
pub(crate) mod macos;
#[cfg(target_os = "macos")]
pub(crate) use self::macos as imp;

#[cfg(target_os = "linux")]
pub(crate) mod linux;
#[cfg(target_os = "linux")]
pub(crate) use self::linux as imp;

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub(crate) mod fallback;
#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
pub(crate) use self::fallback as imp;

/// Runs `f` on the main thread and blocks until it returns.
///
/// Window-layer APIs are not thread-safe everywhere: AppKit panics off the
/// main thread and GTK requires it, while Win32 subclassing must happen on
/// the thread that owns the window. Tauri executes the closure inline when
/// already on the main thread, so this cannot deadlock when called from
/// setup code.
pub(crate) fn dispatch<R, T, F>(webview_window: &WebviewWindow<R>, f: F) -> crate::Result<T>
where
    R: Runtime,
    T: Send + 'static,
    F: FnOnce(&WebviewWindow<R>) -> crate::Result<T> + Send + 'static,
{
    let (tx, rx) = std::sync::mpsc::channel();
    let window = webview_window.clone();
    webview_window.run_on_main_thread(move || {
        let _ = tx.send(f(&window));
    })?;
    rx.recv().map_err(|_| crate::Error::MainThread)?
}
