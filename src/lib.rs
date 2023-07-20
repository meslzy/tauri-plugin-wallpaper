use tauri::{
  AppHandle,
  Invoke,
  Manager,
  plugin::Plugin,
  Runtime,
  window::Window
};

mod platforms;

#[tauri::command]
// invoke("plugin:wallpaper|attach")
fn attach<R: Runtime>(app_handle: AppHandle<R>, window_label: String) {
  let window = app_handle.get_window(&window_label).expect("window not found");
  WallpaperPlugin::attach(&window);
}

#[tauri::command]
// invoke("plugin:wallpaper|detach")
fn detach<R: Runtime>(app_handle: AppHandle<R>, window_label: String) {
  let window = app_handle.get_window(&window_label).expect("window not found");
  WallpaperPlugin::detach(&window);
}

pub struct WallpaperPlugin<R: Runtime> {
  invoke_handler: Box<dyn Fn(Invoke<R>) + Send + Sync>,
}

impl<R: Runtime> WallpaperPlugin<R> {
  pub fn init() -> Self {
    Self {
      invoke_handler: Box::new(tauri::generate_handler![
        attach,
        detach,
      ]),
    }
  }

  pub fn attach(window: &Window<R>) {
    #[cfg(target_os = "windows")]
    {
      let hwnd = window.hwnd().unwrap();
      platforms::windows::attach(hwnd);
      return;
    }

    #[cfg(target_os = "macos")]
    {
      let ns_window = window.ns_window().unwrap();
      platforms::macos::attach(ns_window);
      return;
    }

    panic!("attach not implemented for this platform");
  }

  pub fn detach(window: &Window<R>) {
    #[cfg(target_os = "windows")]
    {
      let hwnd = window.hwnd().unwrap();
      platforms::windows::detach(hwnd);
      return;
    }

    #[cfg(target_os = "macos")]
    {
      let ns_window = window.ns_window().unwrap();
      platforms::macos::detach(ns_window);
      return;
    }

    panic!("detach not implemented for this platform");
  }
}

impl<R: Runtime> Plugin<R> for WallpaperPlugin<R> {
  fn name(&self) -> &'static str {
    "wallpaper"
  }

  fn extend_api(&mut self, message: Invoke<R>) {
    (self.invoke_handler)(message)
  }
}