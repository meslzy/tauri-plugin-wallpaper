const COMMANDS: &[&str] = &["attach", "detach", "reset"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
