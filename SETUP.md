# AutoVideo Setup Guide

## Prerequisites

This project requires two large binary files that are not included in the repository:

### 1. FFmpeg Binary (Required)

Download FFmpeg for Apple Silicon:

```bash
# Option A: Using Homebrew
brew install ffmpeg

# Option B: Manual download
# Download from: https://evermeet.cx/ffmpeg/
# Place in: src-tauri/binaries/ffmpeg-aarch64-apple-darwin
# Make executable: chmod +x src-tauri/binaries/ffmpeg-aarch64-apple-darwin
```

### 2. Whisper Model (Required)

Download the Whisper base model:

```bash
# Create models directory if it doesn't exist
mkdir -p models

# Download ggml-base.bin (~141 MB)
curl -L "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin" \
  -o models/ggml-base.bin
```

### 3. Z.ai API Key (Optional)

For LLM-based transcript cleanup:

1. Sign up at https://z.ai
2. Get your API key
3. Enter it in the app's "LLM Post-Processing" field

## Running the App

```bash
# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev

# Build for production
pnpm tauri build
```

## Features

- 🎤 **Whisper Transcription** with word-level timestamps
- ✂️ **Text-Based Editing** - Click words to delete sections
- 🎬 **Real-Time Preview** - See edits before exporting
- 🤖 **LLM Cleanup** - Fix word fragments (optional)
- 🚀 **Hardware Accelerated** - Uses Metal (Whisper) and VideoToolbox (FFmpeg)

## Troubleshooting

**Video won't load in editor:**
- Check Tauri asset protocol permissions in `src-tauri/tauri.conf.json`

**Transcription fails:**
- Verify Whisper model is in `models/ggml-base.bin`
- Check FFmpeg is available: `ffmpeg -version`

**LLM cleanup fails:**
- Check Z.ai API key is valid
- Verify account has credits and is within rate limits
