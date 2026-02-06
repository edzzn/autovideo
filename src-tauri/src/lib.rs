mod ffmpeg;
mod models;
mod pipeline;
mod transcribe;

use crate::models::{PipelineConfig, PipelineEvent, PipelineResult};
use tauri::Emitter;

#[tauri::command]
async fn get_ffmpeg_version(_app: tauri::AppHandle) -> Result<String, String> {
    Ok("FFmpeg is available".to_string())
}

#[tauri::command]
async fn process_video<R: tauri::Runtime>(
    input_path: String,
    config: PipelineConfig,
    app: tauri::AppHandle<R>,
) -> Result<PipelineResult, String> {
    let result = pipeline::process_video(
        &input_path,
        &config,
        |event: PipelineEvent| {
            app.emit("pipeline-progress", event).map_err(|e| e.to_string())
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
