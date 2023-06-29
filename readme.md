# Tauri Plugin Wallpaper (✨)

> A Tauri plugin to set your window as wallpaper behind desktop icons

---

<div align="center">

![license](https://badgen.net/badge/license/MIT/blue)
![issues](https://badgen.net/github/issues/meslzy/tauri-plugin-wallpaper)

</div>

---

# Available platforms (🚧)

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
use tauri_plugin_wallpaper::Wallpaper;

fn main() {
  tauri::Builder::default().plugin(
    Wallpaper::init(),
  );

  // to attach
  Wallpaper::attach(&window);
  // to detach
  Wallpaper::detach(&window);
}
```

- npm

```ts
import wallpaper from "tauri-plugin-wallpaper";

wallpaper.attach();
// or
const windowLabel = "My Window Label";
wallpaper.attach(windowLabel);

// to detach
wallpaper.detach();
```

---

## The End (💘)
