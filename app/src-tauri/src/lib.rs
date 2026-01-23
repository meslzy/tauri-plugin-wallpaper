use tauri::{Manager, Position, Size};

#[tauri::command]
fn show(app_handle: tauri::AppHandle) {
    println!("showing wallpaper");
    let window = app_handle.get_webview_window("wallpaper").unwrap();
    window.show().unwrap();
}

#[tauri::command]
fn hide(app_handle: tauri::AppHandle) {
    println!("hiding wallpaper");
    let window = app_handle.get_webview_window("wallpaper").unwrap();
    window.hide().unwrap();
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![show, hide])
        .plugin(tauri_plugin_wallpaper::init())
        .setup(|app| {
            let app_handle = app.handle();
            let window = app_handle.get_webview_window("wallpaper").unwrap();
            if let Some(monitor) = app_handle.primary_monitor().unwrap() {
                let position = monitor.position();
                let size = monitor.size();
                window.set_position(Position::Physical(*position)).unwrap();
                window.set_size(Size::Physical(*size)).unwrap();
            }
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
