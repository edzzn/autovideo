use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters, WhisperState};

use crate::ffmpeg::{extract_audio, get_video_duration};
use crate::models::{PipelineConfig, Segment, Transcript, TranscriptResult, Word};

pub fn get_model_path() -> String {
    // Try multiple possible locations for the model
    let possible_paths = vec![
        "models/ggml-base.bin",
        "../models/ggml-base.bin",
        "src-tauri/models/ggml-base.bin",
    ];

    for path in possible_paths {
        if std::path::Path::new(path).exists() {
            eprintln!("✅ Found Whisper model at: {}", path);
            return path.to_string();
        } else {
            eprintln!("⚠️ Model not found at: {}", path);
        }
    }

    // Default to the standard location even if not found
    eprintln!("❌ Model not found in any standard location, using default path");
    "models/ggml-base.bin".to_string()
}

/// Extract word-level timestamps from a segment's tokens
fn extract_words_from_segment(
    state: &WhisperState,
    segment_index: i32,
    global_word_index: &mut usize,
) -> Result<Vec<Word>, String> {
    let mut words = Vec::new();

    let num_tokens = state
        .full_n_tokens(segment_index)
        .map_err(|e| format!("Failed to get token count for segment {}: {}", segment_index, e))?;

    for t in 0..num_tokens {
        let token_text = state
            .full_get_token_text(segment_index, t)
            .map_err(|e| format!("Failed to get token text: {}", e))?;

        // Skip special tokens (timestamps, control tokens)
        if token_text.starts_with('[') || token_text.starts_with('<') {
            continue;
        }

        let token_data = state
            .full_get_token_data(segment_index, t)
            .map_err(|e| format!("Failed to get token data: {}", e))?;

        // t0 and t1 are in centiseconds (100ths of a second)
        let word_start = token_data.t0 as f64 / 100.0;
        let word_end = token_data.t1 as f64 / 100.0;

        // Skip tokens with invalid timestamps
        if word_start < 0.0 || word_end <= word_start {
            continue;
        }

        let trimmed_word = token_text.trim().to_string();

        // Skip empty words
        if !trimmed_word.is_empty() {
            words.push(Word {
                id: format!("w{}", *global_word_index),
                word: trimmed_word,
                start: word_start,
                end: word_end,
            });
            *global_word_index += 1;
        }
    }

    Ok(words)
}

/// Extract segments with word-level timestamps from whisper state
fn extract_segments_with_words(state: &WhisperState) -> Result<Vec<Segment>, String> {
    let num_segments = state
        .full_n_segments()
        .map_err(|e| format!("Failed to get segment count: {}", e))?;

    let mut result_segments = Vec::new();
    let mut global_word_index = 0usize;

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

        let words = extract_words_from_segment(state, i, &mut global_word_index)?;

        result_segments.push(Segment {
            id: i as usize,
            start: start_ms as f64 / 1000.0,
            end: end_ms as f64 / 1000.0,
            text,
            words,
        });
    }

    Ok(result_segments)
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

    let segments = extract_segments_with_words(&state)?;

    Ok(Transcript {
        segments,
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

    let segments = extract_segments_with_words(&state)?;

    Ok(Transcript {
        segments,
        language: None,
    })
}

/// Transcribe video and return TranscriptResult with word-level timestamps
/// This is used by the text-based editor flow
pub async fn transcribe_video_for_editor(
    input_path: &str,
    language: Option<&str>,
    llm_api_key: Option<&str>,
) -> Result<TranscriptResult, String> {
    let pcm_path = format!("{}.pcm", input_path);
    let model_path = get_model_path();

    // Get video duration
    let duration_seconds = get_video_duration(input_path)?;

    eprintln!("🎬 Transcribing video for editor: {}", input_path);
    eprintln!("📏 Video duration: {:.2}s", duration_seconds);

    extract_audio(input_path, &pcm_path, 16000, 1)?;

    let context = WhisperContext::new_with_params(&model_path, WhisperContextParameters::default())
        .map_err(|e| format!("Failed to load Whisper model: {}", e))?;

    let pcm_data_bytes =
        std::fs::read(&pcm_path).map_err(|e| format!("Failed to read PCM data: {}", e))?;

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
    params.set_language(language);

    let mut state = context
        .create_state()
        .map_err(|e| format!("Failed to create state: {}", e))?;
    state
        .full(params, &pcm_data)
        .map_err(|e| format!("Failed to transcribe: {}", e))?;

    let mut segments = extract_segments_with_words(&state)?;

    // Clean up transcript with LLM if API key provided
    if let Some(api_key) = llm_api_key {
        eprintln!("🧹 Cleaning transcript with LLM...");
        match crate::llm::clean_transcript_with_llm(&segments, api_key).await {
            Ok(cleaned_segments) => {
                eprintln!("✨ LLM cleanup successful");
                segments = cleaned_segments;
            }
            Err(e) => {
                eprintln!("⚠️ LLM cleanup failed, using original: {}", e);
                // Continue with original segments if LLM fails
            }
        }
    }

    // Flatten all words from all segments
    let all_words: Vec<Word> = segments
        .iter()
        .flat_map(|s| s.words.clone())
        .collect();

    eprintln!("✅ Transcription complete: {} segments, {} words", segments.len(), all_words.len());

    // Clean up temp file
    let _ = std::fs::remove_file(&pcm_path);

    Ok(TranscriptResult {
        segments,
        words: all_words,
        duration_seconds,
        input_path: input_path.to_string(),
    })
}
