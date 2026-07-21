# AutoVideo

Text-based video editor for macOS — cut videos by editing the transcript instead of scrubbing a timeline.

Drop in a video, get a transcript, delete the sentences you don't want, and export the trimmed cut. The edit happens in text; the timeline follows.

## How it works

- **Transcription** — audio is transcribed on-device with [whisper-rs](https://github.com/tazz4843/whisper-rs) (Metal-accelerated); nothing leaves your machine
- **Text editing** — the transcript is the edit surface; removing words removes the corresponding video segments
- **Export** — cuts are applied via an ffmpeg pipeline in Rust and rendered to a new file

## Stack

- [Tauri 2](https://tauri.app) — Rust core (`transcribe`, `pipeline`, `ffmpeg` modules), native macOS shell
- [SvelteKit](https://kit.svelte.dev) + TypeScript + Tailwind — UI
- whisper.cpp (via whisper-rs) + ffmpeg — transcription and rendering

## Development

```sh
npm install
npm run tauri dev
```

Recommended IDE setup: [VS Code](https://code.visualstudio.com/) + [Svelte](https://marketplace.visualstudio.com/items?itemName=svelte.svelte-vscode) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer).

## Status

Working prototype. Built as a personal tool; expect rough edges.
