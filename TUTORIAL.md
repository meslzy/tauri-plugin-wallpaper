# Tutorial

A practical walkthrough of every feature in `tauri-plugin-wallpaper`. All snippets are TypeScript (frontend); the Rust equivalents are shown where the shape differs.

## Setup

```bash
cargo add tauri-plugin-wallpaper
npm install tauri-plugin-wallpaper
```

Register the plugin (`src-tauri/src/lib.rs`):

```rust
tauri::Builder::default()
    .plugin(tauri_plugin_wallpaper::init())
    // ...
```

Add the permission to your capabilities file (`src-tauri/capabilities/default.json`):

```json
{
  "permissions": ["wallpaper:default"]
}
```

Recommended config for a window you plan to attach (`tauri.conf.json`):

```json
{
  "label": "wallpaper",
  "url": "wallpaper.html",
  "visible": false,
  "decorations": false,
  "skipTaskbar": true
}
```

Show it after attaching, and size it yourself or use the `monitor` option below.

## 1. Feature detection — `capabilities()`

Always start here: the plugin runs everywhere, but not every feature exists everywhere (see the platform matrix in the readme). On Linux this is a live check that distinguishes X11 from Wayland.

```ts
import { capabilities } from "tauri-plugin-wallpaper";

const caps = await capabilities();
// { platform: "windows", attach: true, pin: true, inputForwarding: true,
//   interactive: true, occlusion: true, wallpaperImage: true, ... }

if (!caps.attach) {
  // hide the wallpaper UI instead of letting calls reject
}
```

Unsupported calls reject with a descriptive error (e.g. `"attach" is not supported on this platform: requires an X11 session; ...`) — so feature-detect up front and treat rejections as a backstop.

## 2. Wallpaper mode — `attach` / `detach` / `reset`

Puts a window behind the desktop icons, above the wallpaper image.

```ts
import { attach, detach, reset } from "tauri-plugin-wallpaper";

await attach("wallpaper");   // by label
await attach();              // or the current window

await detach("wallpaper");   // back to a normal window
await reset();               // Windows only: repaint the real wallpaper
                             // (clears any stale frame after detach)
```

Rust side:

```rust
use tauri_plugin_wallpaper::{AttachRequest, WallpaperExt};

app.wallpaper().attach(AttachRequest::new("wallpaper"))?;
// or, with a WebviewWindow in hand:
app.wallpaper().attach_window(&window)?;
```

Notes:
- Attaching is idempotent — calling it twice is fine.
- The window renders behind icons, so it receives **no input** by default; see feature 4.
- On Windows 11 the plugin disables the window's rounded corners while attached (restored on detach) so it fills the layer without gaps.

## 3. Per-monitor attach

Cover one specific display instead of wherever the window happens to be:

```ts
import { availableMonitors } from "@tauri-apps/api/window";
import { attach } from "tauri-plugin-wallpaper";

const monitors = await availableMonitors();
await attach({
  windowLabel: "wallpaper",
  monitor: monitors[0].name!,   // e.g. "\\\\.\\DISPLAY1"
});
```

An unknown name rejects with `monitor "..." not found`. One window covers one monitor; for a wallpaper on every display, create one window per monitor and attach each.

## 4. Interactive wallpaper — input forwarding & `setInteractive`

**Windows** — opt into raw input forwarding at attach time:

```ts
await attach({
  windowLabel: "wallpaper",
  forwardMouseInput: true,
  forwardKeyboardInput: true,
});
```

How it behaves (important):
- Input flows **only while the desktop is focused** — click the desktop or press Win+D first. While you work in other apps, nothing is forwarded (this is what keeps the system lag-free).
- Mouse events arrive in your page as normal DOM events (`mousemove`, `mousedown`, …).
- Keyboard works by the plugin giving the wallpaper window *real* (invisible) focus while you're on the desktop — keys then arrive natively. Clicking an icon takes focus back; the plugin re-grabs it about once a second while you stay on the desktop.

**macOS** — there is no forwarding; instead, temporarily raise the window so the user can interact with it, then send it back:

```ts
import { setInteractive } from "tauri-plugin-wallpaper";

await setInteractive(true, "wallpaper");   // usable like a normal window
await setInteractive(false, "wallpaper");  // back behind the icons
```

On Windows `setInteractive(true)` is a shortcut for enabling both forwarding flags.

## 5. Pin mode — `pin` / `unpin`

The opposite trick: keep a window always on top, surviving Show Desktop (Win+D / Mission Control). Good for overlays, notes, and widgets.

```ts
import { pin, unpin } from "tauri-plugin-wallpaper";

await pin("overlay");
await unpin("overlay");
```

## 6. State queries & events

Ask instead of tracking it yourself:

```ts
import { isAttached, isPinned } from "tauri-plugin-wallpaper";

await isAttached("wallpaper"); // boolean
await isPinned("overlay");
```

Subscribe to lifecycle events (each helper returns an unlisten function):

```ts
import {
  onAttached, onDetached, onPinned, onUnpinned, onReattached,
} from "tauri-plugin-wallpaper";

const unlisten = await onAttached(({ windowLabel }) => {
  console.log(`${windowLabel} is now a wallpaper`);
});
```

`onReattached` is Windows-only in practice: when `explorer.exe` restarts (which destroys the desktop layer), the plugin automatically re-attaches every wallpaper window and fires this event — you don't have to do anything, but it's the right moment to refresh visuals.

## 7. Occlusion monitoring — pause when covered

An animated wallpaper burns CPU/GPU even when a fullscreen app hides it. Opt into occlusion events and pause your render loop:

```ts
import { startOcclusionMonitor, onOcclusionChanged } from "tauri-plugin-wallpaper";

await startOcclusionMonitor(); // 1s poll while running
await onOcclusionChanged(({ windowLabel, occluded }) => {
  occluded ? pauseAnimation() : resumeAnimation();
});
// stopOcclusionMonitor() when you no longer care
```

Windows detects fullscreen apps/presentations (the same shell API Lively Wallpaper uses); macOS reads the window's own occlusion state.

## 8. OS wallpaper image — `setWallpaperImage` / `getWallpaperImage`

Set or read the *actual* wallpaper image (not a window). This is the one feature that also works on Wayland desktops.

```ts
import { setWallpaperImage, getWallpaperImage } from "tauri-plugin-wallpaper";

const previous = await getWallpaperImage();      // path/URI of current image
await setWallpaperImage("C:/images/night.png");  // absolute path
// ... later restore:
await setWallpaperImage(previous);
```

A nice pattern: capture a frame of your animated wallpaper on app exit and set it as the static wallpaper so the desktop doesn't "go blank".

## Troubleshooting

| Symptom | Cause / fix |
|---|---|
| Input forwarding "does nothing" | Input flows only while the desktop is focused — click the desktop or Win+D first. |
| Laggy/slow in dev | `tauri dev` is a debug build; test feel with `tauri dev --release` or a built app. |
| Wallpaper gone after Explorer crash | Handled automatically — listen to `onReattached`. |
| `Unsupported` errors on Linux | You're on Wayland; only `setWallpaperImage`/`getWallpaperImage` work there. Check `capabilities()`. |
| Stale image after detach (Windows) | Call `reset()`. |
| Nothing visible after attach | The window may be created `visible: false` — call `show()` after attaching. |
