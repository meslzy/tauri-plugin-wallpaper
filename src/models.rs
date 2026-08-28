use serde::{Deserialize, Serialize};
use tauri::WebviewWindow;

/// What the current platform supports, so frontends can feature-detect
/// instead of relying on rejected calls.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Capabilities {
    pub platform: &'static str,
    pub attach: bool,
    pub detach: bool,
    pub reset: bool,
    pub pin: bool,
    pub unpin: bool,
    /// Raw mouse/keyboard forwarding to wallpaper windows (Windows only).
    pub input_forwarding: bool,
    /// `setInteractive` support (Windows via forwarding, macOS via window level).
    pub interactive: bool,
    /// Occlusion monitoring (`wallpaper://occlusion` events).
    pub occlusion: bool,
    /// Setting/getting the OS wallpaper image.
    pub wallpaper_image: bool,
}

/// Target monitor bounds in physical pixels, resolved before attaching.
#[derive(Debug, Clone, Copy)]
pub struct MonitorRect {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Attach behavior options. Input forwarding is Windows-only; other
/// platforms ignore the flags (reflected in [`Capabilities`]).
#[derive(Debug, Clone, Copy, Default)]
pub struct AttachOptions {
    pub forward_mouse_input: bool,
    pub forward_keyboard_input: bool,
    pub monitor_rect: Option<MonitorRect>,
}

impl From<&AttachRequest> for AttachOptions {
    fn from(request: &AttachRequest) -> Self {
        Self {
            forward_mouse_input: request.forward_mouse_input,
            forward_keyboard_input: request.forward_keyboard_input,
            monitor_rect: None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AttachRequest {
    pub window_label: String,
    #[serde(default)]
    pub forward_mouse_input: bool,
    #[serde(default)]
    pub forward_keyboard_input: bool,
    #[serde(default)]
    pub monitor: Option<String>,
}

impl AttachRequest {
    pub fn new(window_label: &str) -> Self {
        Self {
            window_label: window_label.to_string(),
            forward_mouse_input: false,
            forward_keyboard_input: false,
            monitor: None,
        }
    }
    pub fn with_input_forwarding(mut self, mouse: bool, keyboard: bool) -> Self {
        self.forward_mouse_input = mouse;
        self.forward_keyboard_input = keyboard;
        self
    }
    pub fn with_monitor(mut self, monitor: &str) -> Self {
        self.monitor = Some(monitor.to_string());
        self
    }
    pub fn from_webview_window<R: tauri::Runtime>(webview_window: &WebviewWindow<R>) -> Self {
        Self::new(webview_window.label())
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
    pub fn from_webview_window<R: tauri::Runtime>(webview_window: &WebviewWindow<R>) -> Self {
        Self {
            window_label: webview_window.label().to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PinRequest {
    pub window_label: String,
}

impl PinRequest {
    pub fn new(window_label: &str) -> Self {
        Self {
            window_label: window_label.to_string(),
        }
    }
    pub fn from_webview_window<R: tauri::Runtime>(webview_window: &WebviewWindow<R>) -> Self {
        Self {
            window_label: webview_window.label().to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UnpinRequest {
    pub window_label: String,
}

impl UnpinRequest {
    pub fn new(window_label: &str) -> Self {
        Self {
            window_label: window_label.to_string(),
        }
    }
    pub fn from_webview_window<R: tauri::Runtime>(webview_window: &WebviewWindow<R>) -> Self {
        Self {
            window_label: webview_window.label().to_string(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowLabelRequest {
    pub window_label: String,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InteractiveRequest {
    pub window_label: String,
    pub interactive: bool,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WallpaperImageRequest {
    pub path: String,
}

/// Payload of `wallpaper://attached|detached|pinned|unpinned|reattached`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WindowEventPayload {
    pub window_label: String,
}

/// Payload of `wallpaper://occlusion`.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OcclusionEvent {
    pub window_label: String,
    pub occluded: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_deserialize_camel_case_payload() {
        let json = r#"{"windowLabel":"main"}"#;

        let attach: AttachRequest = serde_json::from_str(json).unwrap();
        assert_eq!(attach.window_label, "main");

        let detach: DetachRequest = serde_json::from_str(json).unwrap();
        assert_eq!(detach.window_label, "main");

        let pin: PinRequest = serde_json::from_str(json).unwrap();
        assert_eq!(pin.window_label, "main");

        let unpin: UnpinRequest = serde_json::from_str(json).unwrap();
        assert_eq!(unpin.window_label, "main");
    }

    #[test]
    fn attach_request_new_fields_default_for_old_payloads() {
        let attach: AttachRequest = serde_json::from_str(r#"{"windowLabel":"main"}"#).unwrap();
        assert!(!attach.forward_mouse_input);
        assert!(!attach.forward_keyboard_input);
        assert!(attach.monitor.is_none());
    }

    #[test]
    fn attach_request_deserializes_all_options() {
        let json = r#"{"windowLabel":"w","forwardMouseInput":true,"forwardKeyboardInput":true,"monitor":"DISPLAY1"}"#;
        let attach: AttachRequest = serde_json::from_str(json).unwrap();
        assert!(attach.forward_mouse_input);
        assert!(attach.forward_keyboard_input);
        assert_eq!(attach.monitor.as_deref(), Some("DISPLAY1"));

        let options = AttachOptions::from(&attach);
        assert!(options.forward_mouse_input);
        assert!(options.forward_keyboard_input);
        assert!(options.monitor_rect.is_none());
    }

    #[test]
    fn requests_serialize_camel_case() {
        let attach = AttachRequest::new("wallpaper");
        let json = serde_json::to_value(&attach).unwrap();
        assert_eq!(json["windowLabel"], "wallpaper");
        assert_eq!(json["forwardMouseInput"], false);

        let detach = DetachRequest::new("wallpaper");
        assert_eq!(
            serde_json::to_string(&detach).unwrap(),
            r#"{"windowLabel":"wallpaper"}"#
        );
    }

    #[test]
    fn requests_reject_snake_case_payload() {
        let json = r#"{"window_label":"main"}"#;
        assert!(serde_json::from_str::<AttachRequest>(json).is_err());
    }

    #[test]
    fn interactive_request_deserializes() {
        let json = r#"{"windowLabel":"pin","interactive":true}"#;
        let request: InteractiveRequest = serde_json::from_str(json).unwrap();
        assert_eq!(request.window_label, "pin");
        assert!(request.interactive);
    }

    #[test]
    fn event_payloads_serialize_camel_case() {
        let event = OcclusionEvent {
            window_label: "wallpaper".into(),
            occluded: true,
        };
        assert_eq!(
            serde_json::to_string(&event).unwrap(),
            r#"{"windowLabel":"wallpaper","occluded":true}"#
        );
    }
}
