use tauri::{AppHandle, Manager, Position, Size};

#[tauri::command]
fn show_wallpaper(app_handle: tauri::AppHandle) {
    println!("showing wallpaper");
    let window = app_handle.get_webview_window("wallpaper").unwrap();
    window.show().unwrap();
}

#[tauri::command]
fn hide_wallpaper(app_handle: tauri::AppHandle) {
    println!("hiding wallpaper");
    let window = app_handle.get_webview_window("wallpaper").unwrap();
    window.hide().unwrap();
}

#[tauri::command]
fn show_pin(app_handle: tauri::AppHandle) {
    println!("showing pin");
    let window = app_handle.get_webview_window("pin").unwrap();
    window.show().unwrap();
}

#[tauri::command]
fn hide_pin(app_handle: tauri::AppHandle) {
    println!("hiding pin");
    let window = app_handle.get_webview_window("pin").unwrap();
    window.hide().unwrap();
}

#[tauri::command]
fn quit(app_handle: AppHandle) {
    app_handle.exit(0);
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            show_wallpaper,
            hide_wallpaper,
            show_pin,
            hide_pin,
            quit
        ])
        .plugin(tauri_plugin_wallpaper::init())
        .setup(|app| {
            let app_handle = app.handle();

            let wallpaper_window = app_handle.get_webview_window("wallpaper").unwrap();
            if let Some(monitor) = app_handle.primary_monitor().unwrap() {
                let position = monitor.position();
                let size = monitor.size();
                wallpaper_window
                    .set_position(Position::Physical(*position))
                    .unwrap();
                wallpaper_window.set_size(Size::Physical(*size)).unwrap();
            }

            // Self-test: WALLPAPER_MONITOR_TEST=1 attaches to every monitor
            // and asserts the webview client area matches the monitor
            // bounds exactly (catches origin/inset regressions, especially
            // with monitors at negative coordinates).
            if std::env::var("WALLPAPER_MONITOR_TEST").is_ok() {
                use tauri_plugin_wallpaper::{AttachRequest, DetachRequest, WallpaperExt};
                let handle = app_handle.clone();
                std::thread::spawn(move || {
                    std::thread::sleep(std::time::Duration::from_secs(4));
                    let window = handle.get_webview_window("wallpaper").unwrap();
                    let _ = window.show();
                    let monitors = window.available_monitors().unwrap_or_default();
                    for monitor in monitors {
                        let Some(name) = monitor.name().cloned() else {
                            continue;
                        };
                        eprintln!(
                            "[wp-montest] {name}: scale_factor={}",
                            monitor.scale_factor()
                        );
                        let result = handle
                            .wallpaper()
                            .attach(AttachRequest::new("wallpaper").with_monitor(&name));
                        std::thread::sleep(std::time::Duration::from_millis(800));
                        // The plugin's promise: the CLIENT area (webview
                        // content) covers the monitor exactly.
                        let position = window.inner_position().unwrap();
                        let size = window.inner_size().unwrap();
                        let expected_position = monitor.position();
                        let expected_size = monitor.size();
                        let pass = position.x == expected_position.x
                            && position.y == expected_position.y
                            && size.width == expected_size.width
                            && size.height == expected_size.height;
                        eprintln!(
                            "[wp-montest] {name}: attach={result:?} expected=({},{} {}x{}) actual=({},{} {}x{}) => {}",
                            expected_position.x,
                            expected_position.y,
                            expected_size.width,
                            expected_size.height,
                            position.x,
                            position.y,
                            size.width,
                            size.height,
                            if pass { "PASS" } else { "FAIL" }
                        );
                    }
                    let _ = handle.wallpaper().detach(DetachRequest::new("wallpaper"));
                    let _ = window.hide();
                    eprintln!("[wp-montest] DONE");
                });
            }

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
