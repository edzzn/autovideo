<script lang="ts">
  import { Button } from '@/components/ui/button';
  import { Alert, AlertDescription } from '@/components/ui/alert';
  import VideoPreview from './VideoPreview.svelte';
  import TranscriptEditor from './TranscriptEditor.svelte';
  import { transcript, inputPath, keepRanges, reset as resetEditor } from '$lib/stores/editor';
  import { pipelineStore } from '$lib/stores/pipeline';
  import { exportEditedVideo } from '$lib/utils/tauri';
  import { ArrowLeft, Download, Loader2 } from 'lucide-svelte';

  // State
  let seekTime = $state<number | null>(null);
  let isExporting = $state(false);
  let error = $state<string | null>(null);

  // Get values from editor store
  let currentTranscript = $derived($transcript);
  let currentInputPath = $derived($inputPath);
  let currentKeepRanges = $derived($keepRanges);

  function handleSeek(time: number) {
    seekTime = time;
  }

  function handleBack() {
    resetEditor();
    pipelineStore.setScreen('home');
  }

  async function handleExport() {
    if (!currentInputPath) {
      error = 'No input file selected';
      return;
    }

    if (currentKeepRanges.length === 0) {
      error = 'No content to export. Please restore some words.';
      return;
    }

    isExporting = true;
    error = null;

    try {
      // Get enhance_audio setting from pipeline config
      const { config } = pipelineStore.get();
      const outputPath = await exportEditedVideo(
        currentInputPath,
        currentKeepRanges,
        config.enhance_audio
      );

      // Create a minimal result for the done screen
      const originalDuration = currentTranscript?.duration_seconds ?? 0;
      const editedDuration = currentKeepRanges.reduce(
        (sum, [start, end]) => sum + (end - start),
        0
      );

      // Reset editor state before navigating away
      resetEditor();

      pipelineStore.handleEvent({
        PipelineCompleted: {
          result: {
            output_path: outputPath,
            transcript: {
              segments: currentTranscript?.segments ?? [],
              language: null
            },
            stats: {
              original_duration: originalDuration,
              original_size_bytes: 0,
              processed_duration: editedDuration,
              removed_silence_duration: originalDuration - editedDuration,
              silence_percentage:
                originalDuration > 0
                  ? ((originalDuration - editedDuration) / originalDuration) * 100
                  : 0
            }
          }
        }
      });
    } catch (err) {
      error = err instanceof Error ? err.message : String(err);
      isExporting = false;
    }
  }
</script>

<div class="flex flex-col h-screen">
  <!-- Header -->
  <header class="flex items-center justify-between px-4 py-3 border-b bg-background">
    <Button variant="ghost" size="sm" onclick={handleBack} disabled={isExporting}>
      <ArrowLeft class="h-4 w-4 mr-2" />
      Back
    </Button>

    <h1 class="text-lg font-semibold">Edit Transcript</h1>

    <Button
      variant="default"
      size="sm"
      onclick={handleExport}
      disabled={isExporting || currentKeepRanges.length === 0}
    >
      {#if isExporting}
        <Loader2 class="h-4 w-4 mr-2 animate-spin" />
        Exporting...
      {:else}
        <Download class="h-4 w-4 mr-2" />
        Export
      {/if}
    </Button>
  </header>

  <!-- Error Alert -->
  {#if error}
    <div class="px-4 pt-4">
      <Alert variant="destructive">
        <AlertDescription>{error}</AlertDescription>
      </Alert>
    </div>
  {/if}

  <!-- Main Content -->
  <main class="flex-1 flex overflow-hidden">
    {#if currentInputPath && currentTranscript}
      <!-- Left Column: Video Preview (40%) -->
      <div class="w-2/5 p-4 border-r overflow-y-auto">
        <VideoPreview inputPath={currentInputPath} {seekTime} />
      </div>

      <!-- Right Column: Transcript Editor (60%) -->
      <div class="w-3/5 overflow-hidden">
        <TranscriptEditor onSeek={handleSeek} />
      </div>
    {:else}
      <div class="flex-1 flex items-center justify-center">
        <p class="text-muted-foreground">No transcript loaded. Please go back and process a video first.</p>
      </div>
    {/if}
  </main>
</div>
