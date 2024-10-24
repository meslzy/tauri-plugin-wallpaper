use serde::{Deserialize, Serialize};
use tauri::WebviewWindow;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachRequest {
    pub window_label: String,
}

impl AttachRequest {
    pub fn new(window_label: &str) -> Self {
        Self {
            window_label: window_label.to_string(),
        }
    }
    pub fn from_webview_window(webview_window: WebviewWindow) -> Self {
        Self {
            window_label: webview_window.label().to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DetachRequest {
    pub window_label: String,
}

impl DetachRequest {
    pub fn new(window_label: &str) -> Self {
        Self {
            window_label: window_label.to_string(),
        }
    }
    pub fn from_webview_window(webview_window: WebviewWindow) -> Self {
        Self {
            window_label: webview_window.label().to_string(),
        }
    }
}
