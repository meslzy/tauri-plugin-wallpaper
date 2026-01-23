const COMMANDS: &[&str] = &["attach", "detach", "reset", "pin", "unpin"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
