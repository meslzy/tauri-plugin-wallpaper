use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

pub use models::*;

#[cfg(desktop)]
mod desktop;

mod commands;
mod error;
mod models;

mod platform;

pub use error::{Error, Result};

#[cfg(desktop)]
use desktop::Wallpaper;

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
            commands::reset
        ])
        .setup(|app, api| {
            #[cfg(desktop)]
            let wallpaper = desktop::init(app, api)?;
            app.manage(wallpaper);
            Ok(())
        })
        .build()
}
