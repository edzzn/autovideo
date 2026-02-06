use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use crate::ffmpeg::extract_audio;
use crate::models::{PipelineConfig, Segment, Transcript};

pub fn get_model_path() -> &'static str {
    "models/ggml-base.bin"
}

pub fn transcribe_audio(
    input_path: &str,
    output_pcm_path: &str,
    model_path: &str,
    config: &PipelineConfig,
) -> Result<Transcript, String> {
    extract_audio(input_path, output_pcm_path, 16000, 1)?;

    let context = WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
        .map_err(|e| format!("Failed to load Whisper model: {}", e))?;

    let pcm_data_bytes =
        std::fs::read(output_pcm_path).map_err(|e| format!("Failed to read PCM data: {}", e))?;

    let pcm_data: Vec<f32> = pcm_data_bytes
        .chunks_exact(4)
        .map(|chunk| {
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(chunk);
            f32::from_le_bytes(bytes)
        })
        .collect();

    let sampling_strategy = SamplingStrategy::Greedy { best_of: 1 };
    let mut params = FullParams::new(sampling_strategy);
    params.set_no_timestamps(false);
    params.set_token_timestamps(true);
    params.set_language(if let Some(ref lang) = config.language {
        Some(lang.as_str())
    } else {
        None
    });

    let mut state = context
        .create_state()
        .map_err(|e| format!("Failed to create state: {}", e))?;
    state
        .full(params, &pcm_data)
        .map_err(|e| format!("Failed to transcribe: {}", e))?;

    let num_segments = state
        .full_n_segments()
        .map_err(|e| format!("Failed to get segment count: {}", e))?;
    let mut result_segments = Vec::new();

    for i in 0..num_segments {
        let start_ms = state
            .full_get_segment_t0(i)
            .map_err(|e| format!("Failed to get segment start for segment {}: {}", i, e))?;
        let end_ms = state
            .full_get_segment_t1(i)
            .map_err(|e| format!("Failed to get segment end for segment {}: {}", i, e))?;
        let text = state
            .full_get_segment_text(i)
            .map_err(|e| format!("Failed to get segment text for segment {}: {}", i, e))?;

        result_segments.push(Segment {
            id: i as usize,
            start: start_ms as f64 / 1000.0,
            end: end_ms as f64 / 1000.0,
            text,
            words: None,
        });
    }

    Ok(Transcript {
        segments: result_segments,
        language: None,
    })
}

pub fn transcribe_with_progress<F>(
    input_path: &str,
    output_pcm_path: &str,
    model_path: &str,
    config: &PipelineConfig,
    progress_callback: F,
) -> Result<Transcript, String>
where
    F: Fn(f64) -> Result<(), String>,
{
    extract_audio(input_path, output_pcm_path, 16000, 1)?;

    progress_callback(0.5)?;

    let context = WhisperContext::new_with_params(model_path, WhisperContextParameters::default())
        .map_err(|e| format!("Failed to load Whisper model: {}", e))?;

    let pcm_data_bytes =
        std::fs::read(output_pcm_path).map_err(|e| format!("Failed to read PCM data: {}", e))?;

    let pcm_data: Vec<f32> = pcm_data_bytes
        .chunks_exact(4)
        .map(|chunk| {
            let mut bytes = [0u8; 4];
            bytes.copy_from_slice(chunk);
            f32::from_le_bytes(bytes)
        })
        .collect();

    let sampling_strategy = SamplingStrategy::Greedy { best_of: 1 };
    let mut params = FullParams::new(sampling_strategy);
    params.set_no_timestamps(false);
    params.set_token_timestamps(true);
    params.set_language(if let Some(ref lang) = config.language {
        Some(lang.as_str())
    } else {
        None
    });

    let mut state = context
        .create_state()
        .map_err(|e| format!("Failed to create state: {}", e))?;
    state
        .full(params, &pcm_data)
        .map_err(|e| format!("Failed to transcribe: {}", e))?;

    progress_callback(1.0)?;

    let num_segments = state
        .full_n_segments()
        .map_err(|e| format!("Failed to get segment count: {}", e))?;
    let mut result_segments = Vec::new();

    for i in 0..num_segments {
        let start_ms = state
            .full_get_segment_t0(i)
            .map_err(|e| format!("Failed to get segment start for segment {}: {}", i, e))?;
        let end_ms = state
            .full_get_segment_t1(i)
            .map_err(|e| format!("Failed to get segment end for segment {}: {}", i, e))?;
        let text = state
            .full_get_segment_text(i)
            .map_err(|e| format!("Failed to get segment text for segment {}: {}", i, e))?;

        result_segments.push(Segment {
            id: i as usize,
            start: start_ms as f64 / 1000.0,
            end: end_ms as f64 / 1000.0,
            text,
            words: None,
        });
    }

    Ok(Transcript {
        segments: result_segments,
        language: None,
    })
}
