use std::fs;

use crate::ffmpeg::{
    copy_video, cut_silences_and_export, detect_silences, enhance_audio, get_video_duration,
};
use crate::models::{PipelineConfig, PipelineEvent, PipelineResult, TranscriptStats};
use crate::transcribe::{get_model_path, transcribe_with_progress};

pub fn process_video(
    input_path: &str,
    config: &PipelineConfig,
    progress_callback: impl Fn(PipelineEvent) -> Result<(), String>,
) -> Result<PipelineResult, String> {
    // Get actual video duration first
    let original_duration = get_video_duration(input_path)?;

    progress_callback(PipelineEvent::StageStarted {
        stage: "transcribe".to_string(),
    })?;

    let pcm_path = input_path.to_string() + ".pcm";

    let transcript = transcribe_with_progress(
        input_path,
        &pcm_path,
        &get_model_path(),
        config,
        |progress| {
            progress_callback(PipelineEvent::StageProgress {
                stage: "transcribe".to_string(),
                progress,
            })
        },
    )?;

    let output_path = input_path.to_string() + "_edited.mp4";

    let silence_threshold = config.silence_threshold_db;
    let silence_min_duration = config.silence_min_duration;
    let enable_enhancement = config.enhance_audio;
    let cut_silences = config.cut_silences;

    progress_callback(PipelineEvent::StageStarted {
        stage: "detect_silences".to_string(),
    })?;

    let silences = detect_silences(input_path, silence_threshold, silence_min_duration)?;

    progress_callback(PipelineEvent::StageCompleted {
        stage: "detect_silences".to_string(),
    })?;

    // Calculate total silence duration
    let total_silence: f64 = silences.iter().map(|(start, end)| end - start).sum();

    let mut keep_ranges = Vec::new();
    let cut_margin = config.cut_margin;

    if cut_silences && !silences.is_empty() {
        progress_callback(PipelineEvent::StageStarted {
            stage: "cut_silences".to_string(),
        })?;

        // Build keep_ranges from the gaps between silences (the non-silent parts)
        // Apply cut_margin to preserve a bit of padding around speech
        let mut last_end = 0.0;

        for (silence_start, silence_end) in &silences {
            // Keep from last_end to (silence_start + margin)
            let keep_end = (silence_start + cut_margin).min(original_duration);

            if keep_end > last_end {
                keep_ranges.push((last_end, keep_end));
            }

            // Next segment starts at (silence_end - margin), but not before current keep_end
            let next_start = (silence_end - cut_margin).max(0.0);
            last_end = next_start.max(keep_end); // Prevent overlap
        }

        // Keep final segment from last silence end to video end
        if last_end < original_duration {
            keep_ranges.push((last_end, original_duration));
        }

        eprintln!("📊 Keep ranges ({} segments): {:?}", keep_ranges.len(), keep_ranges);

        // Pass original video to cut_silences_and_export (not audio file)
        cut_silences_and_export(input_path, keep_ranges, &output_path, enable_enhancement)?;

        progress_callback(PipelineEvent::StageCompleted {
            stage: "cut_silences".to_string(),
        })?;
    } else if enable_enhancement {
        progress_callback(PipelineEvent::StageStarted {
            stage: "enhance_audio".to_string(),
        })?;

        enhance_audio(input_path, &output_path)?;

        progress_callback(PipelineEvent::StageCompleted {
            stage: "enhance_audio".to_string(),
        })?;
    } else {
        // No processing requested - just copy with faststart
        copy_video(input_path, &output_path)?;
    }

    let file_size = fs::metadata(&output_path).map(|m| m.len()).unwrap_or(0);

    let stats = TranscriptStats {
        original_duration,
        original_size_bytes: file_size,
        processed_duration: get_video_duration(&output_path)?,
        removed_silence_duration: total_silence,
        silence_percentage: (total_silence / original_duration) * 100.0,
    };

    let result = PipelineResult {
        output_path,
        transcript,
        stats,
    };

    let result_clone = result;
    Ok(result_clone)
}

pub fn clean_up_temp_files(input_path: &str) {
    let _ = fs::remove_file(input_path.to_string() + ".pcm");
    let _ = fs::remove_file(input_path.to_string() + ".enhanced.aac");
}
