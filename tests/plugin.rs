use tauri::test::{mock_builder, mock_context, noop_assets, MockRuntime};
use tauri_plugin_wallpaper::{
    AttachRequest, DetachRequest, Error, InteractiveRequest, PinRequest, UnpinRequest,
    WallpaperExt,
};

fn create_app() -> tauri::App<MockRuntime> {
    mock_builder()
        .plugin(tauri_plugin_wallpaper::init())
        .build(mock_context(noop_assets()))
        .expect("failed to build mock app with wallpaper plugin")
}

#[test]
fn plugin_initializes_and_registers_state() {
    let app = create_app();
    let _wallpaper = app.wallpaper();
}

#[test]
fn attach_returns_window_not_found_for_unknown_label() {
    let app = create_app();
    let result = app.wallpaper().attach(AttachRequest::new("does-not-exist"));
    assert!(matches!(result, Err(Error::WindowNotFound(label)) if label == "does-not-exist"));
}

#[test]
fn detach_returns_window_not_found_for_unknown_label() {
    let app = create_app();
    let result = app.wallpaper().detach(DetachRequest::new("does-not-exist"));
    assert!(matches!(result, Err(Error::WindowNotFound(_))));
}

#[test]
fn pin_returns_window_not_found_for_unknown_label() {
    let app = create_app();
    let result = app.wallpaper().pin(PinRequest::new("does-not-exist"));
    assert!(matches!(result, Err(Error::WindowNotFound(_))));
}

#[test]
fn unpin_returns_window_not_found_for_unknown_label() {
    let app = create_app();
    let result = app.wallpaper().unpin(UnpinRequest::new("does-not-exist"));
    assert!(matches!(result, Err(Error::WindowNotFound(_))));
}

#[test]
fn capabilities_reports_current_platform() {
    let app = create_app();
    let capabilities = app.wallpaper().capabilities();
    assert_eq!(capabilities.platform, std::env::consts::OS);

    #[cfg(target_os = "windows")]
    {
        assert!(capabilities.attach);
        assert!(capabilities.detach);
        assert!(capabilities.reset);
        assert!(capabilities.pin);
        assert!(capabilities.unpin);
        assert!(capabilities.input_forwarding);
        assert!(capabilities.interactive);
        assert!(capabilities.occlusion);
        assert!(capabilities.wallpaper_image);
    }
}

#[test]
fn capabilities_serializes_camel_case() {
    let app = create_app();
    let json = serde_json::to_value(app.wallpaper().capabilities()).unwrap();
    for field in [
        "platform",
        "attach",
        "detach",
        "reset",
        "pin",
        "unpin",
        "inputForwarding",
        "interactive",
        "occlusion",
        "wallpaperImage",
    ] {
        assert!(json.get(field).is_some(), "missing capability field {field}");
    }
}

#[test]
fn is_attached_and_is_pinned_default_to_false() {
    let app = create_app();
    assert!(!app.wallpaper().is_attached("main"));
    assert!(!app.wallpaper().is_pinned("main"));
    assert!(!app.wallpaper().is_attached("does-not-exist"));
}

#[test]
fn attach_with_unknown_monitor_fails() {
    let app = create_app();
    let window =
        tauri::WebviewWindowBuilder::new(&app, "with-monitor", tauri::WebviewUrl::default())
            .build()
            .expect("failed to create mock window");

    let request = AttachRequest::from_webview_window(&window).with_monitor("no-such-monitor");
    let result = app.wallpaper().attach(request);
    assert!(matches!(result, Err(Error::MonitorNotFound(name)) if name == "no-such-monitor"));
}

#[test]
fn set_interactive_returns_window_not_found_for_unknown_label() {
    let app = create_app();
    let result = app.wallpaper().set_interactive(InteractiveRequest {
        window_label: "does-not-exist".into(),
        interactive: true,
    });
    assert!(matches!(result, Err(Error::WindowNotFound(_))));
}

#[test]
fn stop_occlusion_monitor_is_safe_without_start() {
    let app = create_app();
    app.wallpaper().stop_occlusion_monitor();
}

#[test]
fn attach_request_builder_sets_options() {
    let request = AttachRequest::new("w")
        .with_input_forwarding(true, false)
        .with_monitor("DISPLAY1");
    assert!(request.forward_mouse_input);
    assert!(!request.forward_keyboard_input);
    assert_eq!(request.monitor.as_deref(), Some("DISPLAY1"));
}

#[test]
fn window_not_found_error_serializes_for_ipc() {
    let error = Error::WindowNotFound("main".into());
    let json = serde_json::to_string(&error).unwrap();
    assert_eq!(json, r#""window with label \"main\" not found""#);
}
