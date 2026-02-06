// TypeScript types mirroring Rust models
// CRITICAL: PipelineEvent uses Rust's tagged enum serialization
// e.g., { "StageStarted": { "stage": "transcribe" } }

export type PipelineEvent =
  | { StageStarted: { stage: string } }
  | { StageProgress: { stage: string; progress: number } }
  | { StageCompleted: { stage: string } }
  | { StageFailed: { stage: string; error: string } }
  | { PipelineCompleted: { result: PipelineResult } }
  | { PipelineFailed: { error: string } };

export interface PipelineConfig {
  enhance_audio: boolean;
  cut_silences: boolean;
  silence_threshold_db: number;
  silence_min_duration: number;
  cut_margin: number;
  language: string | null;
}

export interface PipelineResult {
  output_path: string;
  transcript: Transcript;
  stats: TranscriptStats;
}

export interface TranscriptStats {
  original_duration: number;
  original_size_bytes: number;
  processed_duration: number;
  removed_silence_duration: number;
  silence_percentage: number;
}

export interface Transcript {
  segments: Segment[];
  language: string | null;
}

export interface Segment {
  id: number;
  start: number;
  end: number;
  text: string;
  words: Word[] | null;
}

export interface Word {
  id: string;
  word: string;
  start: number;
  end: number;
}

export interface TranscriptResult {
  segments: Segment[];
  words: Word[];
  duration_seconds: number;
  input_path: string;
}
