use tauri::{command, AppHandle, Runtime};

use crate::models::*;
use crate::Result;
use crate::WallpaperExt;

#[command]
pub(crate) async fn attach<R: Runtime>(app: AppHandle<R>, payload: AttachRequest) -> Result<()> {
    app.wallpaper().attach(payload)
}

#[command]
pub(crate) async fn detach<R: Runtime>(app: AppHandle<R>, payload: DetachRequest) -> Result<()> {
    app.wallpaper().detach(payload)
}

#[command]
pub(crate) async fn reset<R: Runtime>(app: AppHandle<R>) -> Result<()> {
    app.wallpaper().reset()
}

#[command]
pub(crate) async fn pin<R: Runtime>(app: AppHandle<R>, payload: PinRequest) -> Result<()> {
    app.wallpaper().pin(payload)
}

#[command]
pub(crate) async fn unpin<R: Runtime>(app: AppHandle<R>, payload: UnpinRequest) -> Result<()> {
    app.wallpaper().unpin(payload)
}

#[command]
pub(crate) async fn capabilities<R: Runtime>(app: AppHandle<R>) -> Result<Capabilities> {
    Ok(app.wallpaper().capabilities())
}

#[command]
pub(crate) async fn is_attached<R: Runtime>(
    app: AppHandle<R>,
    payload: WindowLabelRequest,
) -> Result<bool> {
    Ok(app.wallpaper().is_attached(&payload.window_label))
}

#[command]
pub(crate) async fn is_pinned<R: Runtime>(
    app: AppHandle<R>,
    payload: WindowLabelRequest,
) -> Result<bool> {
    Ok(app.wallpaper().is_pinned(&payload.window_label))
}

#[command]
pub(crate) async fn set_interactive<R: Runtime>(
    app: AppHandle<R>,
    payload: InteractiveRequest,
) -> Result<()> {
    app.wallpaper().set_interactive(payload)
}

#[command]
pub(crate) async fn start_occlusion_monitor<R: Runtime>(app: AppHandle<R>) -> Result<()> {
    app.wallpaper().start_occlusion_monitor()
}

#[command]
pub(crate) async fn stop_occlusion_monitor<R: Runtime>(app: AppHandle<R>) -> Result<()> {
    app.wallpaper().stop_occlusion_monitor();
    Ok(())
}

#[command]
pub(crate) async fn set_wallpaper_image<R: Runtime>(
    app: AppHandle<R>,
    payload: WallpaperImageRequest,
) -> Result<()> {
    app.wallpaper().set_wallpaper_image(&payload.path)
}

#[command]
pub(crate) async fn get_wallpaper_image<R: Runtime>(app: AppHandle<R>) -> Result<String> {
    app.wallpaper().get_wallpaper_image()
}
