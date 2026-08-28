# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Tauri Plugin Wallpaper - A cross-platform Tauri plugin for advanced window positioning:
- **Wallpaper mode**: Attach windows behind desktop icons (Windows, macOS, Linux/X11); per-monitor via `attach({ monitor })`
- **Pin mode**: Keep windows always-on-top that survive Show Desktop (Windows, macOS, Linux/X11)
- **Input forwarding** (Windows): global raw-input sink forwards mouse/keyboard to wallpaper windows
- **Explorer-restart resilience** (Windows): TaskbarCreated broadcast triggers auto re-attach
- **State & events**: `isAttached`/`isPinned`; events `wallpaper://attached|detached|pinned|unpinned|reattached|occlusion`
- **Occlusion monitoring**: opt-in poll so frontends can pause rendering under fullscreen apps
- **Wallpaper image**: set/get the actual OS wallpaper via the `wallpaper` crate
- **Capabilities**: Runtime feature detection; unsupported platforms (Wayland, mobile) return `Error::Unsupported`

macOS and Linux code cannot be run/tested on this machine: macOS is verified via `cargo check --target aarch64-apple-darwin`; Linux (gtk) cannot even be cross-checked on Windows, so changes there must be based on verified sources, mirroring the patterns in https://github.com/Charlie-XIAO/tauri-plugin-desktop-underlay.

## Build Commands

```bash
# Build plugin
cargo build

# Run Rust tests (unit + mock-runtime integration tests)
cargo test

# Build TypeScript bindings
npm run build

# Lint/format
npm run check

# Run TypeScript tests (vitest)
npm test

# Run example app
cd app && npm run tauri
```

## Architecture

### Dual-Platform Structure

- **Rust plugin** (`src/`): Core Win32 integration compiled as Tauri plugin
- **TypeScript bindings** (`lib/main.ts`): JavaScript API that invokes Rust commands

### Core Rust Files

- `lib.rs`: Plugin entry point, exports `init()`, `WallpaperExt`, event name consts; `on_event` hook cleans state on window Destroyed
- `desktop.rs`: `Wallpaper<R>` state struct; delegates to `platform::imp` via main-thread dispatch; emits events; runs the occlusion poll thread; registers the Windows reattach callback; wallpaper-image via `wallpaper` crate
- `state.rs`: pure-Rust attached/pinned bookkeeping (`WindowStates`), unit-tested without OS calls
- `commands.rs`: IPC command handlers for JavaScript bridge
- `models.rs`: Request DTOs, `AttachOptions`, `Capabilities`, event payloads
- `error.rs`: `Error` enum incl. `Unsupported { feature, reason }`; serialized as message strings over IPC

### Platform Backends (`src/platform/`)

- `mod.rs`: cfg-selects the backend as `imp`; `dispatch()` runs closures on the main thread (blocking, deadlock-safe - tauri runs inline if already on main thread). AppKit/GTK require the main thread; Win32 subclassing must happen on the window's owning thread.
- `windows/`: WorkerW attach with retry (`attacher.rs`), WNDPROC-subclass pin (`pinner.rs`), wallpaper reset (`reseter.rs`), Win11 rounded-corner fix (`corners.rs`), hidden helper window + message pump (`helper.rs` - hosts raw input and TaskbarCreated; its WNDPROC must NEVER block), raw-input translation/fan-out (`input.rs`, ported from electron-as-wallpaper), attached-registry + reattach callback (`reattach.rs`), occlusion via `SHQueryUserNotificationState`
- Input forwarding rules (`input.rs`, empirically verified on this machine — do not remove): forward ONLY while the desktop is foreground (check the foreground's ROOT ancestor class ∈ {Progman, WorkerW} — attached windows are WorkerW children so their root is WorkerW), post mouse to the WebView2 input child (`Chrome_WidgetWin_1`, found via recursive `EnumChildWindows` — it is NOT a direct child: `Tauri Window -> WRY_WEBVIEW -> Chrome_WidgetWin_0 -> Chrome_WidgetWin_1`), skip wheel events, rate-cap synthetic mouse moves (posted WM_MOUSEMOVE is not coalesced by the OS). Removing any of these floods the main thread and makes the app lag.
- Keyboard CANNOT be forwarded synthetically: Chromium/TSF drops key messages without real focus (WM_SETFOCUS fakes don't work). Instead `input.rs` requests REAL focus via a callback into tauri (`set_focus_callback`, registered in `desktop::init`) while the gate is open, and skips keyboard forwarding when a target is focused — real keys then arrive natively (forwarding on top would double them).
- `macos.rs`: objc2 `msg_send!` - attach = `CGWindowLevelForKey(desktopWindow) - 1`, pin = floating level; collection behaviors (canJoinAllSpaces/stationary/ignoresCycle); `setInteractive` = level toggle; occlusion via `occlusionState` (visible = `state & 0x2002`, macOS 26 caveat)
- `linux.rs`: GTK - attach = `WindowTypeHint::Desktop`, pin = `set_keep_above` + `stick`; X11 only, Wayland detected via env and returns `Unsupported`
- `fallback.rs`: mobile/other - everything `Unsupported`

### Threading Rules (Windows)

- The helper window's WNDPROC never blocks and never uses `SendMessage` — `PostMessageA` only (deadlock invariant)
- No static ever stores `HWND` (not `Send`); store `isize` and reconstruct at call sites
- The reattach callback runs on a fresh thread spawned by the WNDPROC; it may block on `platform::dispatch`

### Win32 Strategies

**Wallpaper (attach/detach):**
1. Find `Progman` window
2. Send message `0x052C` to spawn `WorkerW` layer
3. `SetParent()` to reparent window under `WorkerW`
4. Window renders behind desktop icons
5. Disable Win11 rounded corners (`DWMWCP_DONOTROUND`); restored on detach

**Pin (pin/unpin):**
1. Set window to `HWND_TOPMOST`
2. Subclass WNDPROC via `SetWindowLongPtrW`
3. Intercept `WM_WINDOWPOSCHANGING` - when `pos->x == -32000` (Win+D), set `SWP_NOMOVE | SWP_NOSIZE` flags to block
4. Always set `hwndInsertAfter = HWND_TOPMOST`

### Dependency Constraints

- The `windows` crate version must track what tauri (via tao/wry) uses, or `HWND` types mismatch
- `tauri` is depended on with `default-features = false` (no wry) like official plugins; the consuming app enables the runtime
- Platform deps are target-gated (`windows`/`objc2`/`gtk`+`gdk`)

### Plugin Pattern

```rust
use tauri_plugin_wallpaper::WallpaperExt;

// Wallpaper mode
app_handle.wallpaper().attach_window(&window)?;
app_handle.wallpaper().detach_window(&window)?;

// Pin mode
app_handle.wallpaper().pin_window(&window)?;
app_handle.wallpaper().unpin_window(&window)?;
```

### JavaScript API

Commands: `plugin:wallpaper|{attach,detach,reset,pin,unpin,capabilities,is_attached,is_pinned,set_interactive,start_occlusion_monitor,stop_occlusion_monitor,set_wallpaper_image,get_wallpaper_image}`

```typescript
import {
  attach, detach, reset, pin, unpin, capabilities,
  isAttached, isPinned, setInteractive,
  startOcclusionMonitor, stopOcclusionMonitor,
  setWallpaperImage, getWallpaperImage,
  onAttached, onDetached, onPinned, onUnpinned, onReattached, onOcclusionChanged,
} from "tauri-plugin-wallpaper";
```

### Permissions

New commands must be added to `build.rs` COMMANDS array for permission auto-generation.

## Example App

`/app` contains a demo with three windows:
- **main**: Control panel with Wallpaper/Pin sections
- **wallpaper**: Clock display for wallpaper mode
- **pin**: Clock display for pin mode
