#![cfg_attr(all(not(debug_assertions), target_os = "windows"), windows_subsystem = "windows")]

use tauri::Manager;

use tauri_plugin_wallpaper::WallpaperPlugin;

mod commands;

fn main() {
  tauri::Builder::default().plugin(
    WallpaperPlugin::init(),
  ).invoke_handler(
    tauri::generate_handler![
      commands::common::my_custom_command,
    ],
  ).setup(|app| {
    let window = app.get_window("main").unwrap();

    window.show().unwrap();

    let current_monitor = match tauri::Window::primary_monitor(&window).unwrap() {
      Some(monitor) => monitor,
      None => panic!("No monitor found"),
    };

    let position = current_monitor.position();
    let size = current_monitor.size();

    window.set_position(tauri::PhysicalPosition {
      x: position.x,
      y: position.y,
    }).unwrap();

    window.set_size(tauri::Size::Physical(tauri::PhysicalSize {
      width: size.width,
      height: size.height,
    })).unwrap();

    WallpaperPlugin::attach(&window);

    window.show().unwrap();

    return Ok(());
  }).run(
    tauri::generate_context!()
  ).expect(
    "error while running tauri application"
  );
}