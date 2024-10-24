use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Manager, Runtime, WebviewWindow};

use crate::models::*;

#[cfg(target_os = "windows")]
use crate::platform::windows as platform;

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Wallpaper<R>> {
    Ok(Wallpaper(app.clone()))
}

pub struct Wallpaper<R: Runtime>(AppHandle<R>);

impl<R: Runtime> Wallpaper<R> {
    fn get_webview_window(&self, label: &str) -> tauri::WebviewWindow<R> {
        self.0.get_webview_window(label).unwrap()
    }

    pub fn attach(&self, payload: AttachRequest) -> crate::Result<()> {
        let webview_window = self.get_webview_window(&payload.window_label);
        platform::attacher::attach(webview_window)
    }

    pub fn attach_window(&self, webview_window: &WebviewWindow<R>) -> crate::Result<()> {
        platform::attacher::attach(webview_window.clone())
    }

    pub fn detach(&self, payload: DetachRequest) -> crate::Result<()> {
        let webview_window = self.get_webview_window(&payload.window_label);
        platform::detacher::detach(webview_window)
    }

    pub fn detach_window(&self, webview_window: &WebviewWindow<R>) -> crate::Result<()> {
        platform::detacher::detach(webview_window.clone())
    }

    pub fn reset(&self) -> crate::Result<()> {
        platform::reseter::reset()
    }
}
