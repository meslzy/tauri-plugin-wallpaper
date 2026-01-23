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
