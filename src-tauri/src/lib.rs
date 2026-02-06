mod ffmpeg;
mod llm;
mod models;
mod pipeline;
mod transcribe;

use crate::models::{PipelineConfig, PipelineEvent, PipelineResult, TranscriptResult};
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

    // Emit completion event with the result
    eprintln!("🎉 Pipeline completed successfully!");
    _app.emit("pipeline-progress", PipelineEvent::PipelineCompleted {
        result: result.clone(),
    }).map_err(|e| e.to_string())?;

    Ok(result)
}

#[tauri::command]
async fn transcribe_video(input_path: String, language: Option<String>, llm_api_key: Option<String>) -> Result<TranscriptResult, String> {
    let lang_ref = language.as_deref();
    transcribe::transcribe_video_for_editor(&input_path, lang_ref, llm_api_key.as_deref()).await
}

#[tauri::command]
async fn export_edited_video(input_path: String, keep_ranges: Vec<(f64, f64)>, enhance_audio: bool) -> Result<String, String> {
    let output_path = format!("{}_edited.mp4", input_path.trim_end_matches(".mp4").trim_end_matches(".MP4"));
    ffmpeg::cut_silences_and_export(&input_path, keep_ranges, &output_path, enhance_audio)?;
    Ok(output_path)
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
            process_video,
            transcribe_video,
            export_edited_video
        ])
        .run(tauri::generate_context!())
        .map_err(|e| e.into())
}
