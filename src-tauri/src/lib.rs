mod ffmpeg;
mod models;
mod pipeline;
mod transcribe;

use crate::models::{PipelineConfig, PipelineEvent, PipelineResult};
use tauri::Emitter;

#[tauri::command]
async fn get_ffmpeg_version(app: tauri::AppHandle) -> Result<String, String> {
    use tauri_plugin_shell::ShellExt;

    let output = app
        .shell()
        .sidecar("ffmpeg")
        .map_err(|e| e.to_string())?
        .args(["-version"])
        .output()
        .await
        .map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

#[tauri::command]
async fn process_video(_app: tauri::AppHandle, input_path: String, config: PipelineConfig) -> Result<PipelineResult, String> {
    let result = pipeline::process_video(
        &input_path,
        &config,
        |event: PipelineEvent| {
            _app.emit("pipeline-progress", event).map_err(|e| e.to_string())
        },
    )?;

    pipeline::clean_up_temp_files(&input_path);

    Ok(result)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() -> Result<(), tauri::Error> {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_shell::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_notification::init())
        .plugin(tauri_plugin_fs::init())
        .invoke_handler(tauri::generate_handler![
            get_ffmpeg_version,
            process_video
        ])
        .run(tauri::generate_context!())
        .map_err(|e| e.into())
}
