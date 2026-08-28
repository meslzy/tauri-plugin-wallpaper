# Tauri Plugin Wallpaper

> A Tauri plugin for advanced window positioning: wallpaper windows behind desktop icons, and pinned windows that survive Show Desktop

---

<div align="center">

![license](https://badgen.net/badge/license/MIT/blue)
![issues](https://badgen.net/github/issues/meslzy/tauri-plugin-wallpaper)
![stars](https://badgen.net/github/stars/meslzy/tauri-plugin-wallpaper)

</div>

---

## Features

### Wallpaper Mode (Attach/Detach)

Embed your window **behind desktop icons** - perfect for creating animated wallpapers, desktop widgets, or background applications.

- `attach` - Places window behind desktop icons (wallpaper layer)
- `detach` - Restores window to normal behavior
- `reset` - Resets the desktop wallpaper

### Pin Mode (Pin/Unpin)

Keep your window **always on top** and make it survive **Show Desktop** (Win+D / Mission Control) - perfect for overlay apps, sticky notes, or always-visible tools.

- `pin` - Window stays on top and ignores Show Desktop
- `unpin` - Restores normal window behavior

### Interactive Wallpaper (Input Forwarding)

Wallpaper windows normally receive no input — the desktop layer sits above them. On **Windows** the plugin can forward global mouse/keyboard input to attached windows (the technique from [electron-as-wallpaper](https://github.com/meslzy/electron-as-wallpaper)); on **macOS** `setInteractive` temporarily raises the window so it can be used.

- `attach({ forwardMouseInput, forwardKeyboardInput })` - Opt into forwarding at attach time
- `setInteractive` - Toggle interactivity for an attached window

### Per-Monitor Attach

- `attach({ monitor })` - Cover a specific monitor (names from `availableMonitors()`)

### State & Events

- `isAttached` / `isPinned` - Query current state
- Events: `wallpaper://attached`, `detached`, `pinned`, `unpinned`, `reattached`, `occlusion` (helpers: `onAttached`, `onDetached`, `onPinned`, `onUnpinned`, `onReattached`, `onOcclusionChanged`)
- **Windows**: if `explorer.exe` restarts (which destroys the wallpaper layer), attached windows are automatically re-attached and `wallpaper://reattached` fires

### Occlusion Monitoring

- `startOcclusionMonitor` / `stopOcclusionMonitor` - Emit `wallpaper://occlusion` when an attached window is covered by a fullscreen app, so you can pause rendering (~0% CPU/GPU while covered)

### Wallpaper Image

- `setWallpaperImage(path)` / `getWallpaperImage()` - Set/get the actual OS wallpaper image (works on all desktop platforms, including Wayland desktops)

### Capabilities

- `capabilities` - Reports what the current platform supports, so you can feature-detect instead of catching errors

---

## Platform Support

| Platform | Attach / Detach | Pin / Unpin | Input forwarding | Interactive | Occlusion | Wallpaper image | Reset |
|----------|-----------------|-------------|------------------|-------------|-----------|-----------------|-------|
| Windows  | ✅              | ✅          | ✅               | ✅          | ✅        | ✅              | ✅    |
| macOS    | ✅              | ✅          | ❌               | ✅          | ✅        | ✅              | ➖    |
| Linux (X11) | ✅           | ✅          | ❌               | ❌          | ❌        | ✅              | ➖    |
| Linux (Wayland) | ❌       | ❌          | ❌               | ❌          | ❌        | ✅              | ➖    |
| iOS / Android | ❌         | ❌          | ❌               | ❌          | ❌        | ❌              | ❌    |

Unsupported calls reject with a clear error message. Use `capabilities()` to check support at runtime — on Linux this detects X11 vs Wayland.

---

## Tutorial

A step-by-step walkthrough of every feature — setup, wallpaper mode, per-monitor, interactive wallpapers, pin mode, events, occlusion, wallpaper images, and troubleshooting — lives in **[TUTORIAL.md](./TUTORIAL.md)**.

---

## Installation

```bash
# Rust
cargo add tauri-plugin-wallpaper

# JavaScript/TypeScript
npm install tauri-plugin-wallpaper
```

---

## Usage

### Rust

```rust
use tauri_plugin_wallpaper::{WallpaperExt, AttachRequest, PinRequest};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_wallpaper::init())
        .setup(|app| {
            let handle = app.handle();

            // Wallpaper mode - window goes behind desktop icons
            handle.wallpaper().attach(AttachRequest::new("window_label"))?;
            handle.wallpaper().detach(DetachRequest::new("window_label"))?;
            handle.wallpaper().reset()?;

            // Pin mode - window stays on top, survives Win+D
            handle.wallpaper().pin(PinRequest::new("window_label"))?;
            handle.wallpaper().unpin(UnpinRequest::new("window_label"))?;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### JavaScript/TypeScript

```ts
import {
  attach, detach, reset, pin, unpin, capabilities,
  isAttached, setInteractive, startOcclusionMonitor,
  setWallpaperImage, getWallpaperImage,
  onReattached, onOcclusionChanged,
} from "tauri-plugin-wallpaper";

// Feature-detect what the current platform supports
const caps = await capabilities();

// Wallpaper mode - window goes behind desktop icons
if (caps.attach) {
  await attach("window-label");  // or attach() for current window

  // With options: cover a specific monitor, forward input (Windows)
  await attach({
    windowLabel: "window-label",
    monitor: "\\\\.\\DISPLAY1",
    forwardMouseInput: true,
    forwardKeyboardInput: false,
  });

  await isAttached("window-label"); // true

  // Windows: explorer.exe restarts are handled automatically
  await onReattached(({ windowLabel }) => console.log("re-attached", windowLabel));

  // Pause rendering while covered by a fullscreen app
  if (caps.occlusion) {
    await startOcclusionMonitor();
    await onOcclusionChanged(({ occluded }) => (occluded ? pauseRendering() : resumeRendering()));
  }

  // Let the user interact with the wallpaper, then send it back
  if (caps.interactive) {
    await setInteractive(true, "window-label");
    await setInteractive(false, "window-label");
  }

  await detach("window-label");
}
await reset(); // Windows only

// Pin mode - window stays on top, survives Show Desktop
if (caps.pin) {
  await pin("window-label");     // or pin() for current window
  await unpin("window-label");
}

// The actual OS wallpaper image (all desktop platforms)
if (caps.wallpaperImage) {
  await setWallpaperImage("/path/to/image.png");
  const current = await getWallpaperImage();
}
```

---

## Permissions

Add to your Tauri capabilities config:

```json
{
  "permissions": [
    "wallpaper:default"
  ]
}
```

See [permissions reference](./permissions/autogenerated/reference.md) for granular permissions.

---

## How It Works

### Wallpaper Mode

- **Windows**: the WorkerW technique - your window is parented under the desktop's WorkerW layer, behind icons but above the actual wallpaper. On Windows 11 the plugin also disables rounded corners for the attached window so it fills the wallpaper layer without gaps.
- **macOS**: the window level is set just below the desktop level (`CGWindowLevelForKey(desktopWindow) - 1`) with collection behaviors to join all Spaces and stay put during Mission Control.
- **Linux (X11)**: the `_NET_WM_WINDOW_TYPE_DESKTOP` window type hint via GTK, which window managers render at the desktop layer.

### Pin Mode

- **Windows**: subclasses the window procedure to intercept `WM_WINDOWPOSCHANGING`. When Win+D attempts to move the window to (-32000, -32000), the move is blocked, keeping the window visible and topmost.
- **macOS**: the floating window level (the same level always-on-top uses) plus `canJoinAllSpaces` and `stationary` collection behaviors so the window survives Mission Control's Show Desktop.
- **Linux (X11)**: `_NET_WM_STATE_ABOVE` + `_NET_WM_STATE_STICKY` via GTK's `set_keep_above` and `stick`.

All window-layer calls are dispatched to the main thread, which AppKit and GTK require.

### Input Forwarding (Windows)

A hidden helper window registers a global raw-input sink (`RegisterRawInputDevices` with `RIDEV_INPUTSINK`). Raw mouse input is translated into regular window messages (`WM_MOUSEMOVE`, `WM_LBUTTONDOWN`, …) and posted to the WebView2 input window (`Chrome_WidgetWin_1`, resolved at any depth under the Tauri window) of each attached window that opted in, with screen-to-client coordinate mapping. This is the same technique proven in [electron-as-wallpaper](https://github.com/meslzy/electron-as-wallpaper) and [Lively Wallpaper](https://github.com/rocksdanister/lively).

**Keyboard works differently**: Chromium drops synthetic key messages unless the window genuinely has focus (its text-input stack checks real thread focus). So while you are on the desktop, the plugin grants the wallpaper window *real* focus via Tauri — the OS then delivers keyboard input to it natively, and forwarding automatically steps aside to avoid doubled keys. Since the wallpaper window is a WorkerW child, focusing it raises nothing visually.

**Input is only forwarded while the desktop is the foreground window** (click the desktop or press Win+D first) — the same rule Lively uses. Without this gate, every mouse move system-wide would flood the wallpaper window with synthetic messages while you work in other apps. Synthetic mouse moves are also rate-capped, since posted `WM_MOUSEMOVE`s are not coalesced by Windows the way real ones are.

### Explorer Restart Resilience (Windows)

When `explorer.exe` restarts it destroys the WorkerW layer and wallpaper windows silently fall off the desktop. The helper window listens for the shell's `TaskbarCreated` broadcast and re-attaches every tracked window (with retries, since the desktop takes a moment to rebuild), then emits `wallpaper://reattached`.

### Occlusion Detection

- **Windows**: polls `SHQueryUserNotificationState` (the API Lively Wallpaper uses) — reports when a fullscreen app or presentation covers the desktop.
- **macOS**: reads `NSWindow.occlusionState` per attached window.

### Known Limitations

- Input forwarding is Windows-only; on macOS use `setInteractive` instead. On Linux the desktop-type window may receive clicks directly depending on the window manager.
- Wayland has no equivalent of X11 type hints; compositor-specific protocols (wlr-layer-shell) are not supported by GNOME. Wayland calls return an `Unsupported` error (except `setWallpaperImage`/`getWallpaperImage`, which go through desktop-environment tools).

---

## Before You Depend on This

If you're building something beyond a hobby project, consider implementing the platform logic directly in your application rather than depending on this library. The core techniques are documented in the [How It Works](#how-it-works) section and the source code is straightforward to adapt.

This gives you full control over the implementation and avoids dependency on a library that may not be actively maintained.

---

## License

MIT
