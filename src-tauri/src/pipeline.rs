use std::fs;

use crate::ffmpeg::{
    cut_silences_and_export, detect_silences, enhance_audio, export_audio_only, get_video_duration,
};
use crate::models::{PipelineConfig, PipelineEvent, PipelineResult, TranscriptStats};
use crate::transcribe::{get_model_path, transcribe_with_progress};

pub fn process_video(
    input_path: &str,
    config: &PipelineConfig,
    progress_callback: impl Fn(PipelineEvent) -> Result<(), String>,
) -> Result<PipelineResult, String> {
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

    let original_duration = transcript.segments.last().map(|s| s.end).unwrap_or(0.0);

    let audio_path = input_path.to_string() + ".enhanced.aac";
    let output_path = input_path.to_string() + "_edited.mp4";

    let silence_threshold = config.silence_threshold_db;
    let silence_min_duration = config.silence_min_duration;
    let enable_enhancement = config.enhance_audio;
    let cut_silences = config.cut_silences;

    progress_callback(PipelineEvent::StageStarted {
        stage: "detect_silences".to_string(),
    })?;

    let mut silences = detect_silences(input_path, silence_threshold, silence_min_duration)?;

    progress_callback(PipelineEvent::StageCompleted {
        stage: "detect_silences".to_string(),
    })?;

    let total_silence = silences.iter().sum::<f64>();

    let mut keep_ranges = Vec::new();

    if cut_silences && !silences.is_empty() {
        progress_callback(PipelineEvent::StageStarted {
            stage: "cut_silences".to_string(),
        })?;

        if let Some(first) = silences.first_mut() {
            *first = first.min(silence_min_duration / 2.0);
        }

        if let Some(last) = silences.last_mut() {
            *last = last.min(silence_min_duration / 2.0);
        }

        let mut current_start = 0.0;
        for silence_end in silences {
            let silence_start = current_start + silence_end;
            keep_ranges.push((current_start, silence_start));
            current_start = silence_start;
        }
        keep_ranges.push((current_start, original_duration));

        if enable_enhancement {
            enhance_audio(input_path, &audio_path)?;
        } else {
            export_audio_only(input_path, &audio_path)?;
        };

        cut_silences_and_export(&audio_path, keep_ranges, &output_path, config)?;

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
        export_audio_only(input_path, &output_path)?;
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
