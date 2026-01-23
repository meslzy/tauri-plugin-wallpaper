# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Tauri Plugin Wallpaper - A Windows-only Tauri plugin that attaches/detaches windows as desktop wallpaper (behind icons) using Win32 APIs.

## Architecture

### Dual-Platform Structure

- **Rust plugin** (`src/`): Core Win32 integration compiled as Tauri plugin
- **TypeScript bindings** (`lib/main.ts`): JavaScript API that invokes Rust commands

### Core Rust Files

- `lib.rs`: Plugin entry point, exports `init()` and `WallpaperExt` trait
- `desktop.rs`: `Wallpaper<R>` state struct with `attach`/`detach`/`reset` methods
- `attacher.rs`: Win32 logic to parent window under WorkerW (desktop layer)
- `detacher.rs`: Win32 logic to remove parent relationship
- `commands.rs`: IPC command handlers for JavaScript bridge
- `models.rs`: `AttachRequest`/`DetachRequest` DTOs

### Win32 Attachment Strategy

1. Find `Progman` window (main desktop window)
2. Send message `0x052C` to spawn `WorkerW` layer
3. Call `SetParent()` to reparent target window under `WorkerW`
4. Window now renders behind desktop icons

### Plugin Pattern

```rust
// Access from AppHandle via trait extension
use tauri_plugin_wallpaper::WallpaperExt;
app_handle.wallpaper().attach_window(&window)?;
```

### JavaScript API

Commands follow Tauri plugin format: `plugin:wallpaper|{attach,detach,reset}`

```typescript
import { attach, detach, reset } from "tauri-plugin-wallpaper";
attach("window-label");  // or attach() for current window
```

### Permissions

Apps must include `"wallpaper:default"` in their Tauri permissions config to use JavaScript bindings.

## Example App

`/app` contains a working demo with:

- Control window (attach/detach/reset buttons)
- Wallpaper window (clock display that becomes desktop wallpaper)
