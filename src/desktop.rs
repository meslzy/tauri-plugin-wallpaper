use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

use serde::de::DeserializeOwned;
use tauri::{plugin::PluginApi, AppHandle, Emitter, Manager, Runtime, WebviewWindow};

use crate::models::*;
use crate::platform;
use crate::state::WindowStates;

pub const EVENT_ATTACHED: &str = "wallpaper://attached";
pub const EVENT_DETACHED: &str = "wallpaper://detached";
pub const EVENT_PINNED: &str = "wallpaper://pinned";
pub const EVENT_UNPINNED: &str = "wallpaper://unpinned";
pub const EVENT_REATTACHED: &str = "wallpaper://reattached";
pub const EVENT_OCCLUSION: &str = "wallpaper://occlusion";

pub fn init<R: Runtime, C: DeserializeOwned>(
    app: &AppHandle<R>,
    _api: PluginApi<R, C>,
) -> crate::Result<Wallpaper<R>> {
    // Keyboard forwarding needs the wallpaper window genuinely focused
    // (Chromium drops key events otherwise); the helper thread requests it
    // through this callback while the user is on the desktop.
    #[cfg(target_os = "windows")]
    {
        let handle = app.clone();
        platform::windows::input::set_focus_callback(Arc::new(move |label: &str| {
            if let Some(window) = handle.get_webview_window(label) {
                let _ = window.set_focus();
            }
        }));
    }

    // Explorer restarts destroy WorkerW; the helper window broadcasts back
    // through this callback so every attached window gets re-parented.
    #[cfg(target_os = "windows")]
    {
        let handle = app.clone();
        platform::windows::reattach::set_callback(Arc::new(move || {
            for _attempt in 0..5 {
                // The desktop takes a moment to rebuild after TaskbarCreated.
                std::thread::sleep(std::time::Duration::from_millis(500));

                let entries = platform::windows::reattach::attached();
                if entries.is_empty() {
                    return;
                }

                let mut all_ok = true;
                let mut reattached = Vec::new();
                for (label, options) in entries {
                    match handle.get_webview_window(&label) {
                        Some(window) => {
                            let result = platform::dispatch(&window, move |window| {
                                platform::imp::attach(window, options)
                            });
                            match result {
                                Ok(()) => reattached.push(label),
                                Err(_) => all_ok = false,
                            }
                        }
                        None => platform::windows::reattach::note_detached(&label),
                    }
                }

                if all_ok {
                    for label in reattached {
                        let _ = handle.emit(
                            EVENT_REATTACHED,
                            WindowEventPayload {
                                window_label: label,
                            },
                        );
                    }
                    return;
                }
            }
        }));
    }

    Ok(Wallpaper {
        app: app.clone(),
        states: WindowStates::default(),
        occlusion_running: Arc::new(AtomicBool::new(false)),
    })
}

pub struct Wallpaper<R: Runtime> {
    app: AppHandle<R>,
    states: WindowStates,
    occlusion_running: Arc<AtomicBool>,
}

impl<R: Runtime> Wallpaper<R> {
    fn get_webview_window(&self, label: &str) -> crate::Result<WebviewWindow<R>> {
        self.app
            .get_webview_window(label)
            .ok_or_else(|| crate::Error::WindowNotFound(label.to_string()))
    }

    fn emit_window_event(&self, event: &str, label: &str) {
        let _ = self.app.emit(
            event,
            WindowEventPayload {
                window_label: label.to_string(),
            },
        );
    }

    /// Resolves a monitor by name; positions the window over it. Windows
    /// must reposition after reparenting (child coordinates are relative
    /// to WorkerW), so there the rect travels along in the options.
    fn apply_monitor(
        &self,
        webview_window: &WebviewWindow<R>,
        name: &str,
        options: &mut AttachOptions,
    ) -> crate::Result<()> {
        let monitor = webview_window
            .available_monitors()?
            .into_iter()
            .find(|monitor| monitor.name().map(|n| n.as_str() == name).unwrap_or(false))
            .ok_or_else(|| crate::Error::MonitorNotFound(name.to_string()))?;

        let position = *monitor.position();
        let size = *monitor.size();

        #[cfg(target_os = "windows")]
        {
            options.monitor_rect = Some(MonitorRect {
                x: position.x,
                y: position.y,
                width: size.width,
                height: size.height,
            });
        }
        #[cfg(not(target_os = "windows"))]
        {
            let _ = options;
            webview_window.set_position(position)?;
            webview_window.set_size(size)?;
        }

        Ok(())
    }

    pub fn attach(&self, payload: AttachRequest) -> crate::Result<()> {
        let webview_window = self.get_webview_window(&payload.window_label)?;
        let mut options = AttachOptions::from(&payload);
        if let Some(monitor) = &payload.monitor {
            self.apply_monitor(&webview_window, monitor, &mut options)?;
        }
        self.attach_window_with(&webview_window, options)
    }

    pub fn attach_window(&self, webview_window: &WebviewWindow<R>) -> crate::Result<()> {
        self.attach_window_with(webview_window, AttachOptions::default())
    }

    pub fn attach_window_with(
        &self,
        webview_window: &WebviewWindow<R>,
        options: AttachOptions,
    ) -> crate::Result<()> {
        platform::dispatch(webview_window, move |window| {
            platform::imp::attach(window, options)
        })?;
        if self.states.set_attached(webview_window.label(), true) {
            self.emit_window_event(EVENT_ATTACHED, webview_window.label());
        }
        Ok(())
    }

    pub fn detach(&self, payload: DetachRequest) -> crate::Result<()> {
        let webview_window = self.get_webview_window(&payload.window_label)?;
        self.detach_window(&webview_window)
    }

    pub fn detach_window(&self, webview_window: &WebviewWindow<R>) -> crate::Result<()> {
        platform::dispatch(webview_window, |window| platform::imp::detach(window))?;
        if self.states.set_attached(webview_window.label(), false) {
            self.emit_window_event(EVENT_DETACHED, webview_window.label());
        }
        Ok(())
    }

    pub fn reset(&self) -> crate::Result<()> {
        platform::imp::reset()
    }

    pub fn pin(&self, payload: PinRequest) -> crate::Result<()> {
        let webview_window = self.get_webview_window(&payload.window_label)?;
        self.pin_window(&webview_window)
    }

    pub fn pin_window(&self, webview_window: &WebviewWindow<R>) -> crate::Result<()> {
        platform::dispatch(webview_window, |window| platform::imp::pin(window))?;
        if self.states.set_pinned(webview_window.label(), true) {
            self.emit_window_event(EVENT_PINNED, webview_window.label());
        }
        Ok(())
    }

    pub fn unpin(&self, payload: UnpinRequest) -> crate::Result<()> {
        let webview_window = self.get_webview_window(&payload.window_label)?;
        self.unpin_window(&webview_window)
    }

    pub fn unpin_window(&self, webview_window: &WebviewWindow<R>) -> crate::Result<()> {
        platform::dispatch(webview_window, |window| platform::imp::unpin(window))?;
        if self.states.set_pinned(webview_window.label(), false) {
            self.emit_window_event(EVENT_UNPINNED, webview_window.label());
        }
        Ok(())
    }

    pub fn is_attached(&self, label: &str) -> bool {
        self.states.is_attached(label)
    }

    pub fn is_pinned(&self, label: &str) -> bool {
        self.states.is_pinned(label)
    }

    pub fn set_interactive(&self, payload: InteractiveRequest) -> crate::Result<()> {
        let webview_window = self.get_webview_window(&payload.window_label)?;
        self.set_interactive_window(&webview_window, payload.interactive)
    }

    pub fn set_interactive_window(
        &self,
        webview_window: &WebviewWindow<R>,
        interactive: bool,
    ) -> crate::Result<()> {
        platform::dispatch(webview_window, move |window| {
            platform::imp::set_interactive(window, interactive)
        })
    }

    /// Starts emitting `wallpaper://occlusion` events (1s poll) for every
    /// attached window, so frontends can pause rendering while covered.
    pub fn start_occlusion_monitor(&self) -> crate::Result<()> {
        if !platform::imp::capabilities().occlusion {
            return Err(crate::Error::Unsupported {
                feature: "occlusion",
                reason: "occlusion monitoring is not supported on this platform",
            });
        }

        if self.occlusion_running.swap(true, Ordering::SeqCst) {
            return Ok(());
        }

        let app = self.app.clone();
        let running = self.occlusion_running.clone();
        std::thread::spawn(move || {
            let mut last: std::collections::HashMap<String, bool> = std::collections::HashMap::new();
            while running.load(Ordering::SeqCst) {
                let wallpaper = app.state::<Wallpaper<R>>();
                for label in wallpaper.states.attached_labels() {
                    let Some(window) = app.get_webview_window(&label) else {
                        continue;
                    };
                    let Ok(occluded) =
                        platform::dispatch(&window, |window| platform::imp::is_occluded(window))
                    else {
                        continue;
                    };
                    if last.get(&label) != Some(&occluded) {
                        last.insert(label.clone(), occluded);
                        let _ = app.emit(
                            EVENT_OCCLUSION,
                            OcclusionEvent {
                                window_label: label.clone(),
                                occluded,
                            },
                        );
                    }
                }
                std::thread::sleep(std::time::Duration::from_secs(1));
            }
        });

        Ok(())
    }

    pub fn stop_occlusion_monitor(&self) {
        self.occlusion_running.store(false, Ordering::SeqCst);
    }

    /// Sets the OS wallpaper image (the real wallpaper, not a window).
    pub fn set_wallpaper_image(&self, path: &str) -> crate::Result<()> {
        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
        {
            wallpaper::set_from_path(path)
                .map_err(|error| crate::Error::WallpaperImage(error.to_string()))
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            let _ = path;
            Err(crate::Error::Unsupported {
                feature: "setWallpaperImage",
                reason: "this platform has no desktop wallpaper",
            })
        }
    }

    /// Returns the path/URI of the current OS wallpaper image.
    pub fn get_wallpaper_image(&self) -> crate::Result<String> {
        #[cfg(any(target_os = "windows", target_os = "macos", target_os = "linux"))]
        {
            wallpaper::get().map_err(|error| crate::Error::WallpaperImage(error.to_string()))
        }
        #[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
        {
            Err(crate::Error::Unsupported {
                feature: "getWallpaperImage",
                reason: "this platform has no desktop wallpaper",
            })
        }
    }

    pub fn capabilities(&self) -> Capabilities {
        platform::imp::capabilities()
    }

    /// Called from the plugin's run-event hook when a window is destroyed.
    pub(crate) fn handle_window_destroyed(&self, label: &str) {
        self.states.remove(label);
        platform::imp::window_destroyed(label);
    }
}
