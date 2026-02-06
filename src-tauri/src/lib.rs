use tauri_plugin_shell::ShellExt;

#[tauri::command]
async fn get_ffmpeg_version(app: tauri::AppHandle) -> Result<String, String> {
    let output = app.shell()
        .sidecar("ffmpeg").map_err(|e| e.to_string())?
        .args(["-version"])
        .output().await.map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![get_ffmpeg_version])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
