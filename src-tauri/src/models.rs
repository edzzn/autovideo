use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Transcript {
    pub segments: Vec<Segment>,
    pub language: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TranscriptResult {
    pub segments: Vec<Segment>,
    pub words: Vec<Word>,
    pub duration_seconds: f64,
    pub input_path: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Segment {
    pub id: usize,
    pub start: f64,
    pub end: f64,
    pub text: String,
    pub words: Vec<Word>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Word {
    pub id: String,
    pub word: String,
    pub start: f64,
    pub end: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PipelineConfig {
    pub enhance_audio: bool,
    pub cut_silences: bool,
    pub silence_threshold_db: f64,
    pub silence_min_duration: f64,
    pub cut_margin: f64,
    pub language: Option<String>,
}

impl Default for PipelineConfig {
    fn default() -> Self {
        PipelineConfig {
            enhance_audio: true,
            cut_silences: true,
            silence_threshold_db: -30.0,
            silence_min_duration: 0.5,
            cut_margin: 0.2,
            language: None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PipelineResult {
    pub output_path: String,
    pub transcript: Transcript,
    pub stats: TranscriptStats,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TranscriptStats {
    pub original_duration: f64,
    pub original_size_bytes: u64,
    pub processed_duration: f64,
    pub removed_silence_duration: f64,
    pub silence_percentage: f64,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum PipelineEvent {
    StageStarted { stage: String },
    StageProgress { stage: String, progress: f64 },
    StageCompleted { stage: String },
    StageFailed { stage: String, error: String },
    PipelineCompleted { result: PipelineResult },
    PipelineFailed { error: String },
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy)]
pub enum PipelineStage {
    Transcribe,
    DetectSilences,
    EnhanceAudio,
    CutSilences,
    Export,
}
