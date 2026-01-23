# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Tauri Plugin Wallpaper - A Windows-only Tauri plugin for advanced window positioning:
- **Wallpaper mode**: Attach windows behind desktop icons
- **Pin mode**: Keep windows always-on-top that survive Win+D (Show Desktop)

## Build Commands

```bash
# Build plugin
cargo build

# Build TypeScript bindings
npm run build

# Lint/format
npm run lint

# Run example app
cd app && npm run tauri
```

## Architecture

### Dual-Platform Structure

- **Rust plugin** (`src/`): Core Win32 integration compiled as Tauri plugin
- **TypeScript bindings** (`lib/main.ts`): JavaScript API that invokes Rust commands

### Core Rust Files

- `lib.rs`: Plugin entry point, exports `init()` and `WallpaperExt` trait
- `desktop.rs`: `Wallpaper<R>` state struct with all methods
- `commands.rs`: IPC command handlers for JavaScript bridge
- `models.rs`: Request DTOs (`AttachRequest`, `DetachRequest`, `PinRequest`, `UnpinRequest`)

**Wallpaper mode:**
- `attacher.rs`: Win32 logic to parent window under WorkerW (desktop layer)
- `detacher.rs`: Win32 logic to remove parent relationship
- `reseter.rs`: Win32 logic to reset desktop wallpaper

**Pin mode:**
- `pinner.rs`: Subclasses window procedure to intercept `WM_WINDOWPOSCHANGING`, blocks Win+D hide attempts, keeps window topmost
- `unpinner.rs`: Restores original window procedure and removes topmost flag

### Win32 Strategies

**Wallpaper (attach/detach):**
1. Find `Progman` window
2. Send message `0x052C` to spawn `WorkerW` layer
3. `SetParent()` to reparent window under `WorkerW`
4. Window renders behind desktop icons

**Pin (pin/unpin):**
1. Set window to `HWND_TOPMOST`
2. Subclass WNDPROC via `SetWindowLongPtrW`
3. Intercept `WM_WINDOWPOSCHANGING` - when `pos->x == -32000` (Win+D), set `SWP_NOMOVE | SWP_NOSIZE` flags to block
4. Always set `hwndInsertAfter = HWND_TOPMOST`

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

Commands: `plugin:wallpaper|{attach,detach,reset,pin,unpin}`

```typescript
import { attach, detach, reset, pin, unpin } from "tauri-plugin-wallpaper";
```

### Permissions

New commands must be added to `build.rs` COMMANDS array for permission auto-generation.

## Example App

`/app` contains a demo with three windows:
- **main**: Control panel with Wallpaper/Pin sections
- **wallpaper**: Clock display for wallpaper mode
- **pin**: Clock display for pin mode
