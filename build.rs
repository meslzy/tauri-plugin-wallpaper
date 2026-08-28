const COMMANDS: &[&str] = &[
    "attach",
    "detach",
    "reset",
    "pin",
    "unpin",
    "capabilities",
    "is_attached",
    "is_pinned",
    "set_interactive",
    "start_occlusion_monitor",
    "stop_occlusion_monitor",
    "set_wallpaper_image",
    "get_wallpaper_image",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
