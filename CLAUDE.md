# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

AutoVideo is a native macOS app for automated vlog editing on Apple Silicon. Built with Tauri v2 (Rust backend) + SvelteKit (frontend). The app transcribes video using Whisper (Metal acceleration), detects/cuts silences, enhances audio, and exports edited videos using FFmpeg with VideoToolbox hardware encoding.

## Development Commands

### Frontend (SvelteKit)
```bash
pnpm dev              # Start dev server (runs on http://localhost:1420 via Tauri)
pnpm build            # Build frontend for production
pnpm check            # Run Svelte type checking
pnpm check:watch      # Run type checking in watch mode
```

### Tauri App
```bash
pnpm tauri dev        # Run app in dev mode (auto-runs pnpm dev)
pnpm tauri build      # Build production app bundle
```

### Rust Backend
```bash
cd src-tauri
cargo build           # Build Rust backend
cargo build --release # Build optimized release binary
cargo test            # Run Rust tests
```

## Architecture

### Pipeline Flow
The video processing pipeline (`src-tauri/src/pipeline.rs`) orchestrates these stages:

1. **Transcribe** (`transcribe.rs`): Extract audio → PCM → Whisper (Metal) → timestamped segments
2. **Detect Silences** (`ffmpeg.rs::detect_silences`): Use FFmpeg silencedetect filter
3. **Cut Silences** (optional): Calculate keep ranges, apply select/aselect filters
4. **Enhance Audio** (optional): Apply afftdn (noise reduction) + loudnorm
5. **Export**: VideoToolbox H.264 encoding, AAC audio, faststart for streaming

### Key Modules

**`src-tauri/src/lib.rs`** — Tauri entry point, command registration
**`src-tauri/src/pipeline.rs`** — Main pipeline orchestration, progress callbacks
**`src-tauri/src/transcribe.rs`** — Whisper integration (Metal feature)
**`src-tauri/src/ffmpeg.rs`** — FFmpeg command wrappers for all video operations
**`src-tauri/src/models.rs`** — Shared data models (Transcript, PipelineConfig, PipelineEvent)

### Tauri Commands

Two main commands exposed to frontend:

- `get_ffmpeg_version()` — FFmpeg availability check
- `process_video(input_path, config, app)` — Main pipeline, emits `pipeline-progress` events

### FFmpeg Sidecar

FFmpeg binary bundled as sidecar in `src-tauri/binaries/ffmpeg` (configured in `tauri.conf.json`). The Rust code invokes it via `Command::new("binaries/ffmpeg")`. Binary must be codesigned before bundling:

```bash
codesign --force --options runtime --timestamp --sign "Developer ID..." binaries/ffmpeg-aarch64-apple-darwin
```

Naming convention: `ffmpeg-<target-triple>` (e.g., `ffmpeg-aarch64-apple-darwin`)

### VideoToolbox Encoding

Always use these FFmpeg flags for macOS hardware encoding (in `cut_silences_and_export`):
- `-c:v h264_videotoolbox`
- `-b:v 8M` (bitrate targeting, not CRF)
- `-pix_fmt yuv420p`
- `-movflags +faststart`
- Profile: `high`

### Whisper Model

Model path hardcoded: `models/ggml-base.bin` (GGML format, not GGUF). Uses `whisper-rs` with `metal` feature for Apple Silicon GPU acceleration. Model loading happens per-request (future: cache context).

### Progress Events

Pipeline emits `PipelineEvent` enum via `app.emit("pipeline-progress", event)`:
- `StageStarted { stage }`
- `StageProgress { stage, progress }` (0.0–1.0)
- `StageCompleted { stage }`
- `PipelineCompleted { result }` (includes stats)

Frontend listens via Tauri event system.

### Temporary Files

Pipeline creates temp files alongside input:
- `{input}.pcm` — Extracted audio for Whisper
- `{input}.enhanced.aac` — Enhanced audio (when cutting silences)
- `{input}_edited.mp4` — Final output

Cleanup handled by `clean_up_temp_files()` after pipeline completion.

## Frontend Architecture

Single-page app (SPA mode via `adapter-static`). Entry: `src/routes/+page.svelte`. SSR disabled in `src/routes/+layout.ts`.

Tauri API usage:
```typescript
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';

// Invoke command
const result = await invoke('process_video', { inputPath, config });

// Listen to events
const unlisten = await listen('pipeline-progress', (event) => {
  console.log('Progress:', event.payload);
});
```

## Key Technical Constraints

- **Apple Silicon only**: Whisper Metal + VideoToolbox require macOS ARM64
- **VFR videos**: iPhone footage is variable frame rate (VFR) HEVC; may cause A/V sync issues with select/aselect filters (fallback: segment-based concat)
- **HDR content**: Dolby Vision needs tone-mapping to SDR for Instagram compatibility
- **LGPL licensing**: FFmpeg build excludes GPL codecs (libx264); VideoToolbox replaces it

## Configuration Defaults

See `PipelineConfig::default()` in `models.rs`:
- `enhance_audio: true` — Apply noise reduction + loudnorm
- `cut_silences: true` — Remove silent portions
- `silence_threshold_db: -30.0` — Silence detection threshold
- `silence_min_duration: 0.5` — Minimum silence length (seconds)
- `cut_margin: 0.2` — Preserve margin around speech
- `language: None` — Auto-detect (or specify "en", "es", etc.)

## Testing Notes

- No test suite currently implemented
- Manual testing: place video in project root, run `pnpm tauri dev`, select file
- Whisper model: download `ggml-base.bin` from [whisper.cpp models](https://huggingface.co/ggerganov/whisper.cpp) → place in `models/` directory

## Future Improvements

- Tauri Channels for progress (not events) — better backpressure handling
- Whisper context caching across requests
- Segment-based concat fallback for VFR videos with A/V sync issues
- Error recovery and partial pipeline resume
