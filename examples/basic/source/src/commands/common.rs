#[tauri::command]
pub fn my_custom_command() -> Result<String, String> {
    println!("I was invoked from Webview!");
    Ok("This worked!".into())
}