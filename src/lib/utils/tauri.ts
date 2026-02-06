import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import type { PipelineConfig, PipelineEvent, TranscriptResult } from '$lib/types/pipeline';
import { pipelineStore } from '$lib/stores/pipeline';

let unlistenFn: UnlistenFn | null = null;

export async function startListening(): Promise<void> {
  if (unlistenFn) {
    console.log('⚠️ Already listening to pipeline-progress events');
    return;
  }

  console.log('👂 Setting up listener for pipeline-progress events...');
  unlistenFn = await listen<PipelineEvent>('pipeline-progress', (event) => {
    console.log('📨 Received pipeline event:', event.payload);
    pipelineStore.handleEvent(event.payload);
  });
  console.log('✅ Listener setup complete');
}

export function stopListening(): void {
  if (unlistenFn) {
    console.log('🛑 Stopping pipeline-progress listener');
    unlistenFn();
    unlistenFn = null;
  }
}

export async function processVideo(
  inputPath: string,
  config: PipelineConfig
): Promise<void> {
  console.log('📞 Invoking process_video command');
  console.log('   Input path:', inputPath);
  console.log('   Config:', JSON.stringify(config, null, 2));

  try {
    await invoke('process_video', { inputPath, config });
    console.log('✅ process_video command invoked successfully');
  } catch (err) {
    console.error('❌ process_video command failed:', err);
    throw err;
  }
}

export async function getFFmpegVersion(): Promise<string> {
  console.log('📞 Invoking get_ffmpeg_version command');
  try {
    const version = await invoke<string>('get_ffmpeg_version');
    console.log('✅ FFmpeg version:', version);
    return version;
  } catch (err) {
    console.error('❌ get_ffmpeg_version command failed:', err);
    throw err;
  }
}

export async function transcribeVideo(
  inputPath: string,
  language: string | null = null,
  llmApiKey: string | null = null
): Promise<TranscriptResult> {
  console.log('📞 Invoking transcribe_video command');
  console.log('   Input path:', inputPath);
  console.log('   Language:', language ?? 'auto-detect');
  console.log('   LLM cleanup:', llmApiKey ? 'enabled' : 'disabled');

  try {
    const result = await invoke<TranscriptResult>('transcribe_video', {
      inputPath,
      language,
      llmApiKey
    });
    console.log('✅ transcribe_video completed');
    console.log('   Segments:', result.segments.length);
    console.log('   Words:', result.words.length);
    console.log('   Duration:', result.duration_seconds, 'seconds');
    return result;
  } catch (err) {
    console.error('❌ transcribe_video command failed:', err);
    throw err;
  }
}

export async function exportEditedVideo(
  inputPath: string,
  keepRanges: [number, number][],
  enhanceAudio: boolean
): Promise<string> {
  console.log('📞 Invoking export_edited_video command');
  console.log('   Input path:', inputPath);
  console.log('   Keep ranges:', keepRanges.length);
  console.log('   Enhance audio:', enhanceAudio);

  try {
    const outputPath = await invoke<string>('export_edited_video', {
      inputPath,
      keepRanges,
      enhanceAudio
    });
    console.log('✅ export_edited_video completed');
    console.log('   Output path:', outputPath);
    return outputPath;
  } catch (err) {
    console.error('❌ export_edited_video command failed:', err);
    throw err;
  }
}
