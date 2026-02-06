# AutoVideo — Native Mac App

> **Goal**: Drop a video, get a clean edit. Silence removed, audio enhanced, transcript generated. All local, all hardware-accelerated on Apple Silicon. ~30 seconds per 7-minute clip.

---

## 1. Problem & Solution

Edisson records weekly Instagram vlogs about his startup journey. Manual editing takes forever: cutting silences, cleaning audio, exporting. A previous Python pipeline on an Intel N100 mini PC took 8+ minutes per 7-minute clip.

**AutoVideo** is a native macOS app that does this in ~30 seconds on an M4 Mac Mini. Drop video in, get clean video out. No accounts, no uploads, no subscriptions.

---

## 2. MVP Features (v0.1)

Single screen. Three states. No settings page.

**Runs automatically on drop:**
1. **Transcribe** — whisper.cpp (Metal GPU) generates word-level transcript
2. **Detect silences** — FFmpeg silencedetect parses quiet ranges

**Toggleable before export (checkboxes, on by default):**
3. **Enhance audio** — FFmpeg afftdn (noise reduction) + loudnorm (normalization)
4. **Cut silences** — FFmpeg select/aselect removes dead air

**Always:**
5. **Export** — h264_videotoolbox, saves `_edited.mp4` next to original
6. **Progress UI** — stepper showing current pipeline stage

### NOT in v0.1 (deferred)
- Filler word removal → v0.2
- Auto captions → v0.2
- Transcript editor / text-based editing → v0.3
- Video preview player → v0.3
- Format presets / aspect ratio conversion → v0.3
- Bad take detection → v0.3
- Settings page → v0.2 (hardcode sensible defaults first)

---

## 3. User Flow

### State 1: Empty (Drop Zone)
```
┌──────────────────────────────────┐
│                                  │
│                                  │
│      Drop video file here        │
│        or click to browse        │
│                                  │
│         .mov  .mp4  .mkv         │
│                                  │
│                                  │
└──────────────────────────────────┘
```

### State 2: Processing
```
┌──────────────────────────────────┐
│  IMG_1482.MOV                    │
│                                  │
│  ● Transcribing...       ━━━━░░ │
│  ○ Enhance audio                 │
│  ○ Cut silences                  │
│  ○ Export                        │
│                                  │
│  [x] Enhance audio               │
│  [x] Cut silences                │
│                                  │
└──────────────────────────────────┘
```
Toggles are visible during processing so the user can enable/disable features while transcription runs. The pipeline adapts based on what's checked at export time.

### State 3: Done
```
┌──────────────────────────────────┐
│  IMG_1482.MOV                    │
│                                  │
│  ✓ Transcribed (Spanish)         │
│  ✓ Audio enhanced                │
│  ✓ Silences cut                  │
│                                  │
│  Removed: 1:23 of silence (19%) │
│  Duration: 7:12 → 5:49          │
│                                  │
│  ┌────────────┐ ┌──────────────┐ │
│  │ Show in    │ │ Process      │ │
│  │ Finder     │ │ another      │ │
│  └────────────┘ └──────────────┘ │
└──────────────────────────────────┘
```

### Flow Rules
- Original file is **never modified**
- Each export creates `filename_edited.mp4` next to the original
- Transcription always runs first (it's the foundation for silence detection)
- Toggles define what gets applied at export time
- macOS notification when done

---

## 4. Architecture

### Project Structure
```
autovideo/                          ← Tauri v2 project root
├── src/                            ← SvelteKit frontend
│   ├── routes/
│   │   ├── +layout.ts             ← export const ssr = false
│   │   └── +page.svelte           ← Single page: drop → process → done
│   ├── lib/
│   │   ├── components/
│   │   │   └── pipeline-progress.svelte
│   │   ├── stores/
│   │   │   └── pipeline.ts        ← Pipeline state (idle/processing/done)
│   │   └── tauri.ts               ← Tauri invoke/channel wrappers
│   ├── app.html
│   └── app.css                    ← Tailwind
│
├── src-tauri/                      ← Rust backend
│   ├── src/
│   │   ├── main.rs                ← Desktop entry point
│   │   ├── lib.rs                 ← Shared: commands, plugins, setup
│   │   ├── pipeline.rs            ← Orchestrates: transcribe → enhance → cut → export
│   │   ├── transcribe.rs          ← whisper-rs (Metal)
│   │   ├── ffmpeg.rs              ← FFmpeg sidecar wrapper + stderr parser
│   │   └── models.rs              ← Transcript, Segment, PipelineConfig types
│   ├── binaries/
│   │   └── ffmpeg-aarch64-apple-darwin
│   ├── capabilities/
│   │   └── default.json           ← shell:allow-execute, fs permissions
│   ├── Cargo.toml
│   └── tauri.conf.json
│
├── models/                         ← Whisper GGML models (gitignored)
│   └── ggml-base.bin              ← ~142MB, downloaded on first run
│
├── package.json
├── svelte.config.js               ← adapter-static, fallback: 'index.html'
├── tailwind.config.js
└── vite.config.ts
```

### Data Flow
```
User drops video file
    ↓
[Rust] Extract audio → 16kHz mono f32 PCM (FFmpeg)           ← ~2s
    ↓
[Rust] whisper.cpp: transcribe with word timestamps (Metal)   ← ~15-20s
    ↓
[Rust] FFmpeg: silencedetect → parse silence ranges           ← ~2s
    ↓
[Rust] FFmpeg: afftdn + loudnorm (if enhance enabled)        ← ~5s
    ↓
[Rust] FFmpeg: select/aselect + h264_videotoolbox             ← ~10-15s
    ↓
[Svelte] Show results: stats + "Show in Finder"
    ↓
✅ filename_edited.mp4 saved next to original
```

### Tauri IPC: Channels for Progress
```rust
// Rust: stream progress via Channel (not events)
#[tauri::command]
async fn process_video(
    input: String,
    config: PipelineConfig,
    on_progress: Channel<PipelineEvent>,
) -> Result<PipelineResult, String> {
    on_progress.send(PipelineEvent::StageStarted { stage: "transcribe" })?;
    let transcript = transcribe(&input, &on_progress).await?;

    on_progress.send(PipelineEvent::StageStarted { stage: "enhance" })?;
    // ...
}
```
```typescript
// Svelte: listen to Channel
const onProgress = new Channel<PipelineEvent>();
onProgress.onmessage = (event) => {
  currentStage = event.stage;
};
await invoke('process_video', { input: path, config, onProgress });
```

---

## 5. Technical Decisions (Research-Validated)

| Decision | Choice | Why (from research) |
|----------|--------|---------------------|
| Framework | **Tauri v2** (stable since Oct 2024) | SvelteKit frontend, ~5MB base, WKWebView on macOS |
| Frontend | **SvelteKit** + adapter-static | SPA mode, `ssr: false`, `fallback: 'index.html'` |
| UI components | **shadcn-svelte** + **extras** | File Drop Zone, Stepper, Meter components ready to use |
| Transcription | **whisper-rs** with **`metal` feature** | Simpler than CoreML (no model conversion), comparable speed, GPU-accelerated |
| Whisper model | **ggml-base.bin** (~142MB, GGML format) | Good accuracy for clear Spanish speech, ~250MB RAM, fast |
| Silence detection | **FFmpeg silencedetect** | `-30dB` threshold, `0.5s` min duration, runs on enhanced audio |
| Silence cutting | **FFmpeg select/aselect** | Single command, `setpts=N/FRAME_RATE/TB` for timestamp reset |
| Audio enhance | **FFmpeg afftdn + loudnorm** | Both LGPL, included in all FFmpeg builds |
| Video encoding | **h264_videotoolbox** | Hardware media engine, 3-10x faster than libx264 |
| Quality control | **`-b:v 8M -maxrate 10M`** (not `-q:v`) | Bitrate targeting is more predictable than constant quality |
| FFmpeg build | **LGPL-only** | VideoToolbox replaces libx264, minimal licensing obligations |
| FFmpeg sidecar | Binary in `src-tauri/binaries/` | Target triple suffix: `ffmpeg-aarch64-apple-darwin` |
| Progress streaming | **Tauri Channels** (not events) | Type-safe, efficient, designed for command-scoped streaming |
| Distribution | **DMG + notarization** | Sign FFmpeg with `--options runtime --timestamp` before bundling |
| Target | **ARM64 only** | Apple Silicon is the target audience |
| Critical flags | Always: `-movflags +faststart`, `-pix_fmt yuv420p` | Required for social media compatibility |

### Defaults (Hardcoded in v0.1)

```rust
const SILENCE_THRESHOLD_DB: f64 = -30.0;
const SILENCE_MIN_DURATION: f64 = 0.5;
const CUT_MARGIN: f64 = 0.2;           // seconds of padding around speech
const VIDEO_BITRATE: &str = "8M";
const VIDEO_MAXRATE: &str = "10M";
const VIDEO_BUFSIZE: &str = "16M";
const VIDEO_PROFILE: &str = "high";
const AUDIO_BITRATE: &str = "192k";
const AUDIO_SAMPLERATE: u32 = 44100;
const WHISPER_LANGUAGE: &str = "auto";  // auto-detect (reliable for Spanish)
```

---

## 6. Dependencies

### Rust (Cargo.toml)
```toml
[dependencies]
tauri = { version = "2", features = [] }
tauri-plugin-shell = "2"                # Sidecar (FFmpeg)
tauri-plugin-dialog = "2"               # Native file picker
tauri-plugin-notification = "2"         # macOS notifications
tauri-plugin-fs = "2"                   # File system access
whisper-rs = { version = "0.13", features = ["metal"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
regex = "1"
```

### Frontend (package.json)
```json
{
  "devDependencies": {
    "@sveltejs/adapter-static": "latest",
    "@tauri-apps/cli": "^2",
    "tailwindcss": "latest"
  },
  "dependencies": {
    "@tauri-apps/api": "^2",
    "@tauri-apps/plugin-shell": "^2",
    "@tauri-apps/plugin-dialog": "^2",
    "@tauri-apps/plugin-notification": "^2",
    "@tauri-apps/plugin-fs": "^2"
  }
}
```

### shadcn-svelte Components
```bash
npx shadcn-svelte@next init
npx jsrepo add ui/file-drop-zone    # Drag-and-drop import
npx jsrepo add ui/stepper           # Pipeline progress steps
npx jsrepo add ui/meter             # Progress bars within steps
```

### Bundled Binaries
- **FFmpeg** static ARM64 (~70-90MB) — LGPL-only, from evermeet.cx or custom build
- **ggml-base.bin** (~142MB) — downloaded on first run from Hugging Face

---

## 7. Build Order

### Phase 1: Scaffold + Sidecar
- [ ] `pnpm create tauri-app` (SvelteKit template)
- [ ] Configure adapter-static: `fallback: 'index.html'`, `ssr: false`
- [ ] Set up Tailwind + shadcn-svelte + extras
- [ ] Download FFmpeg ARM64 static binary → `src-tauri/binaries/ffmpeg-aarch64-apple-darwin`
- [ ] Configure sidecar in `tauri.conf.json` (`bundle.externalBin`)
- [ ] Add shell permissions in `capabilities/default.json`
- [ ] Verify: invoke `ffmpeg -version` from Rust, log output
- [ ] **Test**: `pnpm tauri dev` shows SvelteKit app, FFmpeg runs from Rust

### Phase 2: Pipeline Commands (Rust)
- [ ] `ffmpeg.rs` — sidecar wrapper: spawn, stream stderr, parse progress
- [ ] `transcribe.rs` — whisper-rs with Metal: load model, extract audio via FFmpeg, transcribe, return segments with word timestamps
- [ ] `pipeline.rs` — orchestrator: transcribe → detect silences → enhance → cut+export
  - Extract audio to PCM (FFmpeg) → feed to whisper
  - Run silencedetect on audio → parse ranges → invert to "keep" ranges
  - Apply enhance + cut + encode in final FFmpeg command
- [ ] `models.rs` — types: `Transcript`, `Segment`, `Word`, `PipelineConfig`, `PipelineResult`, `PipelineEvent`
- [ ] Wire up Channel-based progress streaming
- [ ] **Test**: call `process_video` from Rust tests with a real .mov file

### Phase 3: Frontend
- [ ] Install File Drop Zone, Stepper, Meter from shadcn-svelte-extras
- [ ] `+page.svelte` — single page with 3 states:
  - **Idle**: File Drop Zone (accepts .mov, .mp4, .mkv)
  - **Processing**: Stepper (4 steps) + toggles (enhance audio, cut silences)
  - **Done**: Stats (duration saved, %) + "Show in Finder" + "Process another"
- [ ] `pipeline.ts` store — tracks: state, current stage, progress, result
- [ ] Wire Tauri Channel → Svelte store updates
- [ ] **Test**: drop a video, see progress, see result

### Phase 4: Integration + Polish
- [ ] End-to-end test with real iPhone MOV (HEVC, VFR, possibly HDR)
- [ ] Error handling: file not found, FFmpeg crash, unsupported format, whisper failure
- [ ] macOS notification on completion: "Done! Removed 1:23 from IMG_1482.MOV"
- [ ] Handle edge cases:
  - No silence detected → export unchanged (with enhance if enabled)
  - Entire file is silence → show warning
  - Very short keep-segments (< 0.1s) → filter out to avoid flash frames
- [ ] Test with: MOV (HEVC), MP4 (H.264), vertical 9:16, 4K, 1080p
- [ ] App icon
- [ ] **Test**: full flow works reliably with multiple file types

---

## 8. FFmpeg Command Reference

### Extract audio for Whisper
```bash
ffmpeg -i input.mov -ar 16000 -ac 1 -f f32le -acodec pcm_f32le output.pcm
```

### Silence detection
```bash
ffmpeg -i input.mov -af silencedetect=noise=-30dB:d=0.5 -f null - 2>&1
# Parse: silence_start: 3.504 / silence_end: 5.839
```

### Audio enhance only (copy video)
```bash
ffmpeg -i input.mov \
  -af "afftdn=nf=-25,loudnorm=I=-16:TP=-1.5:LRA=11" \
  -c:v copy -y enhanced.mov
```

### Cut silences + enhance + encode (single pass)
```bash
ffmpeg -i input.mov \
  -vf "select='between(t,0,3.5)+between(t,5.8,12.0)+between(t,14.3,60)',setpts=N/FRAME_RATE/TB" \
  -af "aselect='between(t,0,3.5)+between(t,5.8,12.0)+between(t,14.3,60)',asetpts=N/SR/TB,afftdn=nf=-25,loudnorm=I=-16:TP=-1.5:LRA=11" \
  -c:v h264_videotoolbox -b:v 8M -maxrate 10M -bufsize 16M -profile:v high \
  -c:a aac -b:a 192k -ar 44100 \
  -pix_fmt yuv420p -movflags +faststart \
  -y output_edited.mp4
```

### Enhance only (no cuts)
```bash
ffmpeg -i input.mov \
  -af "afftdn=nf=-25,loudnorm=I=-16:TP=-1.5:LRA=11" \
  -c:v h264_videotoolbox -b:v 8M -maxrate 10M -bufsize 16M -profile:v high \
  -c:a aac -b:a 192k -ar 44100 \
  -pix_fmt yuv420p -movflags +faststart \
  -y output_edited.mp4
```

---

## 9. Competitive Landscape

### How competitors work (from research)

**Descript** ($24/mo) — Desktop (Electron). Text-based video editor: edit transcript to edit video. Strikethrough for deleted content. Transcript + timeline + canvas layout. Cloud-rendered exports. Filler word removal, Studio Sound, AI voice. Pain points: resource-heavy, cloud-dependent, precision limited to word-level.

**Gling** ($15/mo) — Web-based. Upload video → AI detects silences + bad takes → review cuts on timeline → export or send to NLE (Premiere, Resolve). Shows detected cuts color-coded by reason. Pain points: requires upload (slow), web-only, no offline.

**CapCut** ($8/mo Pro) — Desktop (native). Full timeline editor with AI features: auto-captions, silence removal, templates, effects. Traditional NLE layout. Pain points: complex UI for simple tasks, ByteDance data concerns.

**Timebolt** ($100 one-time) — Desktop. Focused silence remover. Shows waveform with highlighted silences, toggle segments on/off, export or generate EDL for NLE. Closest to our approach but no transcription or audio enhancement.

**Opus Clip** ($29/mo) — Web. Long-form → short-form repurposing. Paste a YouTube link → AI extracts best clips → auto-captions + reframing. Not a general editor.

### Our positioning

| | Descript | Gling | CapCut | Timebolt | **AutoVideo** |
|---|---|---|---|---|---|
| Price | $24/mo | $15/mo | $8/mo | $100 | **Free** |
| Platform | Desktop | Web | Desktop | Desktop | **Desktop** |
| Offline | Partial | No | Yes | Yes | **Yes** |
| Speed (7min clip) | ~2-3min | ~3-5min | ~1-2min | ~30s | **~30s** |
| Silence removal | Yes | Yes | Yes | Yes | **Yes** |
| Audio enhance | Yes | No | No | No | **Yes** |
| Transcription | Yes | Yes | Yes | No | **Yes** |
| Privacy (local) | Partial | No | No | Yes | **Yes** |
| Spanish-first | No | No | No | No | **Yes** |

---

## 10. Version Roadmap

### v0.1 — Drop & Clean (MVP)
Drop video → enhanced audio + silences removed → export. Single screen, no settings.

### v0.2 — Smart Cuts + Settings
- Filler word detection (Spanish + English) from transcript timestamps
- Auto captions (SRT/VTT export from transcript)
- Settings panel: silence threshold, margin, model size, language
- Queue for multiple files

### v0.3 — Transcript Editor
- Text-based editing: delete words in transcript → removes from video
- Strikethrough for auto-detected cuts (silence=gray, filler=orange)
- Click-to-restore any cut segment
- Video preview synced to transcript cursor
- Format presets: Instagram Reel (9:16), Feed (1:1), YouTube (16:9)

### v0.4 — Branding
- Auto captions burned into video (style presets: bold, minimal, pop)
- Logo watermark overlay
- Aspect ratio conversion (16:9 → 9:16 smart crop)
- Background music with ducking

### v1.0 — Ship
- Code signing + notarization
- DMG distribution
- App icon + polish
- Model auto-download on first run

### Future
- Bad take detection (repeated phrases, keep cleanest)
- AI clip extraction (long-form → best short clips)
- Dynamic zoom on emphasis points
- Multi-file batch processing

---

## 11. Risks & Mitigations

| Risk | Impact | Likelihood | Mitigation |
|------|--------|------------|------------|
| A/V sync drift with select/aselect | Medium | Medium | Filter out keep-segments < 0.1s; fallback to segment-based concat if drift > 200ms |
| iPhone VFR HEVC edge cases | Medium | Medium | Detect VFR with `vfrdet` filter; pre-convert to CFR if needed |
| whisper-rs Metal build complexity | Medium | Low | Metal is simpler than CoreML; fall back to CPU-only (still fast on M4) |
| FFmpeg sidecar codesigning | Medium | Medium | Pre-sign with `--options runtime --timestamp` before bundling |
| Large app size (~230MB) | Low | High | Acceptable for video app; download model on first run to reduce initial size |
| HDR/Dolby Vision from iPhone | Low | Medium | Tone-map to SDR with zscale filter; Instagram doesn't support HDR playback |
| Expression length limits (100+ segments) | Low | Low | Switch to segment-based approach if segment count > 80 |

---

## 12. Resolved Decisions

- [x] **Platform**: macOS native (Tauri v2)
- [x] **Whisper acceleration**: Metal (not CoreML) — simpler, no model conversion
- [x] **FFmpeg license**: LGPL-only — VideoToolbox replaces libx264
- [x] **VideoToolbox quality**: Bitrate targeting (`-b:v 8M`) not constant quality (`-q:v`)
- [x] **Progress IPC**: Tauri Channels (not events)
- [x] **Target arch**: ARM64 only (Apple Silicon)
- [x] **Model delivery**: Download on first run (~142MB)
- [x] **Multi-file**: Sequential queue (v0.2)
- [x] **UI**: Single screen with 3 states (not multi-page)

---

**Last Updated**: 2026-02-05
**Owner**: Edisson Reinozo
