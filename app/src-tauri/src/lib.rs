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

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
