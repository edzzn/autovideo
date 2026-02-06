# AutoVideo v0.1 Implementation Plan

## Current State
- Only `project.md` exists (detailed spec)
- No code scaffolded yet (0% implementation)

## Goal
Phase 1: "pnpm tauri dev shows SvelteKit app, FFmpeg runs from Rust"

---

## Phase 1: Scaffold + Sidecar

### Step 1: Scaffold Tauri v2 + SvelteKit
```bash
cd /Users/edzzn/dev
rm -rf autovideo  # Remove spec-only folder
pnpm dlx sv create autovideo  # Select: skeleton, TypeScript
cd autovideo
pnpm add -D @tauri-apps/cli
pnpm tauri init  # App: AutoVideo, URL: localhost:5173, dist: ../build
```

### Step 2: Configure adapter-static (SPA mode)
- Install: `pnpm add -D @sveltejs/adapter-static`
- **svelte.config.js**: `adapter({ fallback: 'index.html' })`
- **src/routes/+layout.ts**: `export const ssr = false;`

### Step 3: Setup Tailwind + shadcn-svelte
```bash
pnpm dlx shadcn-svelte@next init
npx jsrepo init @ieedan/shadcn-svelte-extras
npx jsrepo add @ieedan/shadcn-svelte-extras/file-drop-zone
npx jsrepo add @ieedan/shadcn-svelte-extras/stepper
npx jsrepo add @ieedan/shadcn-svelte-extras/meter
```

### Step 4: Download FFmpeg sidecar
```bash
mkdir -p src-tauri/binaries
curl -L "https://ffmpeg.martin-riedl.de/redirect/latest/macos/arm64/release/ffmpeg.zip" -o /tmp/ffmpeg.zip
unzip /tmp/ffmpeg.zip -d /tmp/ffmpeg
mv /tmp/ffmpeg/ffmpeg src-tauri/binaries/ffmpeg-aarch64-apple-darwin
chmod +x src-tauri/binaries/ffmpeg-aarch64-apple-darwin
xattr -d com.apple.quarantine src-tauri/binaries/ffmpeg-aarch64-apple-darwin
```

### Step 5: Configure sidecar in tauri.conf.json
```json
{
  "bundle": {
    "externalBin": ["binaries/ffmpeg"]
  }
}
```

### Step 6: Add shell plugin + permissions
```bash
pnpm tauri add shell
pnpm add @tauri-apps/plugin-shell
```

**src-tauri/capabilities/default.json**:
```json
{
  "identifier": "main-capability",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "shell:allow-spawn",
    "shell:allow-execute",
    {
      "identifier": "shell:allow-execute",
      "allow": [{ "name": "binaries/ffmpeg", "sidecar": true, "args": true }]
    }
  ]
}
```

### Step 7: FFmpeg verification command
**src-tauri/src/lib.rs**:
```rust
use tauri_plugin_shell::ShellExt;

#[tauri::command]
async fn get_ffmpeg_version(app: tauri::AppHandle) -> Result<String, String> {
    let output = app.shell()
        .sidecar("ffmpeg").map_err(|e| e.to_string())?
        .args(["-version"])
        .output().await.map_err(|e| e.to_string())?;

    if output.status.success() {
        Ok(String::from_utf8_lossy(&output.stdout).to_string())
    } else {
        Err(String::from_utf8_lossy(&output.stderr).to_string())
    }
}

pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .invoke_handler(tauri::generate_handler![get_ffmpeg_version])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

### Step 8: Test page
**src/routes/+page.svelte**: Button that calls `invoke('get_ffmpeg_version')` and displays result.

---

## Phase 1 Verification
```bash
pnpm tauri dev
```
- [ ] Window opens with SvelteKit app
- [ ] Tailwind styles work
- [ ] Click button shows FFmpeg version with `videotoolbox` encoder

---

## Phase 2: Pipeline Commands (after Phase 1)
- `ffmpeg.rs` — sidecar wrapper, stderr parser
- `transcribe.rs` — whisper-rs with Metal
- `pipeline.rs` — orchestrator
- `models.rs` — types (Transcript, Segment, PipelineEvent)
- Wire Tauri Channels for progress

## Phase 3: Frontend UI (after Phase 2)
- Drop zone → Processing (stepper) → Done states
- pipeline.ts store
- Channel → store updates

## Phase 4: Integration (after Phase 3)
- E2E test with iPhone MOV
- Error handling
- macOS notification
- Edge cases

---

## Key Files to Create/Modify
| File | Purpose |
|------|---------|
| `svelte.config.js` | adapter-static, fallback: 'index.html' |
| `src/routes/+layout.ts` | ssr: false |
| `src-tauri/tauri.conf.json` | externalBin: ["binaries/ffmpeg"] |
| `src-tauri/capabilities/default.json` | shell permissions |
| `src-tauri/src/lib.rs` | get_ffmpeg_version command |
| `src-tauri/binaries/ffmpeg-aarch64-apple-darwin` | FFmpeg binary |

## Potential Issues
| Issue | Mitigation |
|-------|------------|
| Gatekeeper blocks FFmpeg | `xattr -d com.apple.quarantine` |
| Sidecar permission denied | Verify capabilities/default.json |
| adapter-static fails | Ensure +layout.ts has ssr=false |
