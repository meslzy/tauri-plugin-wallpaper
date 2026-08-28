use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, RunEvent, Runtime, WindowEvent,
};

pub use models::*;

mod commands;
mod desktop;
mod error;
mod models;
mod platform;
mod state;

pub use desktop::{
    Wallpaper, EVENT_ATTACHED, EVENT_DETACHED, EVENT_OCCLUSION, EVENT_PINNED, EVENT_REATTACHED,
    EVENT_UNPINNED,
};
pub use error::{Error, Result};

pub trait WallpaperExt<R: Runtime> {
    fn wallpaper(&self) -> &Wallpaper<R>;
}

impl<R: Runtime, T: Manager<R>> crate::WallpaperExt<R> for T {
    fn wallpaper(&self) -> &Wallpaper<R> {
        self.state::<Wallpaper<R>>().inner()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("wallpaper")
        .invoke_handler(tauri::generate_handler![
            commands::attach,
            commands::detach,
            commands::reset,
            commands::pin,
            commands::unpin,
            commands::capabilities,
            commands::is_attached,
            commands::is_pinned,
            commands::set_interactive,
            commands::start_occlusion_monitor,
            commands::stop_occlusion_monitor,
            commands::set_wallpaper_image,
            commands::get_wallpaper_image
        ])
        .setup(|app, api| {
            let wallpaper = desktop::init(app, api)?;
            app.manage(wallpaper);
            Ok(())
        })
        .on_event(|app, event| {
            if let RunEvent::WindowEvent {
                label,
                event: WindowEvent::Destroyed,
                ..
            } = event
            {
                if let Some(wallpaper) = app.try_state::<Wallpaper<R>>() {
                    wallpaper.handle_window_destroyed(label);
                }
            }
        })
        .build()
}
