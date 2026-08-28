use serde::{ser::Serializer, Serialize};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),
    #[error(transparent)]
    Tauri(#[from] tauri::Error),
    #[cfg(windows)]
    #[error(transparent)]
    Windows(#[from] windows::core::Error),
    #[error("window with label \"{0}\" not found")]
    WindowNotFound(String),
    #[error("monitor \"{0}\" not found")]
    MonitorNotFound(String),
    #[error("wallpaper image operation failed: {0}")]
    WallpaperImage(String),
    #[error("wallpaper helper thread failed to start")]
    HelperThread,
    #[error("wallpaper helper window could not be created")]
    HelperWindow,
    #[error("\"{feature}\" is not supported on this platform: {reason}")]
    Unsupported {
        feature: &'static str,
        reason: &'static str,
    },
    #[error("failed to run on main thread")]
    MainThread,
    #[error("Progman window not found")]
    ProgmanWindowNotFound,
    #[error("WorkerW window not found")]
    WorkerWindowNotFound,
    #[error("failed to subclass window procedure")]
    SubclassFailed,
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.to_string().as_ref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serializes_to_message_string() {
        let error = Error::WindowNotFound("main".to_string());
        let json = serde_json::to_string(&error).unwrap();
        assert_eq!(json, r#""window with label \"main\" not found""#);
    }

    #[test]
    fn worker_window_not_found_message() {
        let error = Error::WorkerWindowNotFound;
        assert_eq!(error.to_string(), "WorkerW window not found");
    }
}
