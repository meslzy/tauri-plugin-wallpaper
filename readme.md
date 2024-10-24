# Tauri Plugin Wallpaper (✨)

> A Tauri plugin to set your window as wallpaper behind desktop icons

---

<div align="center">

![license](https://badgen.net/badge/license/MIT/blue)
![issues](https://badgen.net/github/issues/meslzy/tauri-plugin-wallpaper)
![stars](https://badgen.net/github/stars/meslzy/tauri-plugin-wallpaper)

</div>

---

## Available platforms (🚧)

- [x] Windows
- [ ] MacOS
- [ ] Linux

---

## Getting Started (✅)

- ### Installation (⏬)

  - cargo

    ```bash
    cargo add tauri-plugin-wallpaper
    ```

  - npm

    ```bash
    npm install tauri-plugin-wallpaper
    ```

### How to use (🌠)

- cargo

```rust
#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![show, hide])
        .plugin(tauri_plugin_wallpaper::init())
        .setup(|app| {
            let app_handle = app.handle();
            let webview_window = app_handle.get_webview_window("wallpaper").unwrap();
            // attach window to wallpaper
            app_handle
                .wallpaper()
                .attach_window(&webview_window)
                .unwrap();
            // attach using window label identifier
            app_handle
                .wallpaper()
                .attach(AttachRequest::new("window_label"))
                .unwrap();

            // detach window from wallpaper
            app_handle
                .wallpaper()
                .detach_window(&webview_window)
                .unwrap();

            // detach using window label identifier
            app_handle
                .wallpaper()
                .detach(DetachRequest::new("window_label"))
                .unwrap();

            // reset wallpaper
            app_handle.wallpaper().reset().unwrap();
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

- npm

```ts
import wallpaper from "tauri-plugin-wallpaper";
// or import { attach, detach, reset } from "tauri-plugin-wallpaper";

// Attach current window as wallpaper
wallpaper.attach();
// or using window label identifier
wallpaper.attach("window-label");

// detach current window from wallpaper
wallpaper.detach();
// or using window label identifier
wallpaper.detach("window-label");

// reset wallpaper
wallpaper.reset();
```

---

## The End (💘)
