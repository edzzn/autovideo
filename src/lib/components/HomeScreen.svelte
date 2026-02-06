<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { open } from '@tauri-apps/plugin-dialog';
  import { listen, type UnlistenFn } from '@tauri-apps/api/event';
  import { Button } from "@/components/ui/button";
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
  import { Alert, AlertDescription } from "@/components/ui/alert";
  import FileDropZone from "$lib/components/ui/file-drop-zone/file-drop-zone.svelte";
  import FileDropZoneTrigger from "$lib/components/ui/file-drop-zone/file-drop-zone-trigger.svelte";
  import ConfigPanel from "./ConfigPanel.svelte";
  import { pipelineStore } from "@/lib/stores/pipeline";
  import { transcribeVideo } from "@/lib/utils/tauri";
  import { setTranscript } from "$lib/stores/editor";
  import { FileVideo, X, Loader2 } from "lucide-svelte";

  let { selectedFile, config } = $derived($pipelineStore);
  let error = $state<string | null>(null);
  let isTranscribing = $state(false);
  let llmApiKey = $state<string>('');
  let unlistenDrop: UnlistenFn | null = null;

  const videoExtensions = ['mp4', 'mov', 'avi', 'mkv', 'hevc', 'webm', 'm4v'];
  // Accept all video files
  const acceptedTypes = 'video/*';

  onMount(async () => {
    // Listen for Tauri file drop events (these provide actual file paths)
    unlistenDrop = await listen<string[]>('tauri://drag-drop', (event) => {
      if (event.payload && event.payload.length > 0) {
        const filePath = event.payload[0];
        const ext = filePath.split('.').pop()?.toLowerCase();

        if (ext && videoExtensions.includes(ext)) {
          pipelineStore.setFile(filePath);
          error = null;
        } else {
          error = `Invalid file type. Please select a video file: ${videoExtensions.join(', ')}`;
        }
      }
    });
  });

  onDestroy(() => {
    if (unlistenDrop) {
      unlistenDrop();
    }
  });

  // Handle clicks on the drop zone (opens file dialog)
  async function handleUpload(files: File[]) {
    // When user clicks (not drags), open the file dialog to get actual path
    try {
      const selected = await open({
        multiple: false,
        filters: [{
          name: 'Video',
          extensions: videoExtensions
        }]
      });

      if (selected && typeof selected === 'string') {
        pipelineStore.setFile(selected);
        error = null;
      }
    } catch (err) {
      error = err instanceof Error ? err.message : 'Failed to select file';
    }
  }

  function handleFileRejected(opts: { reason: string; file: File }) {
    error = `File "${opts.file.name}" rejected. ${opts.reason}`;
  }

  function clearFile() {
    pipelineStore.setFile(null);
    error = null;
  }

  async function startProcessing() {
    if (!selectedFile) {
      console.log('❌ No file selected');
      return;
    }

    console.log('🚀 Starting transcription...');
    console.log('📁 File:', selectedFile);
    console.log('⚙️ Language:', config.language ?? 'auto-detect');

    try {
      error = null;
      isTranscribing = true;

      console.log('🎤 Invoking transcribe_video command...');
      const result = await transcribeVideo(
        selectedFile,
        config.language,
        llmApiKey.trim() || null
      );
      console.log('✅ Transcription completed');
      console.log('   Segments:', result.segments.length);
      console.log('   Words:', result.words.length);

      // Set the transcript in the editor store
      setTranscript(result);

      // Navigate to the editor screen
      pipelineStore.setScreen('editor');
    } catch (err) {
      console.error('❌ Error during transcription:', err);
      error = err instanceof Error ? err.message : 'Failed to transcribe video';
    } finally {
      isTranscribing = false;
    }
  }
</script>

<div class="container mx-auto px-4 py-8 max-w-4xl">
  <div class="space-y-8">
    <!-- Header -->
    <div class="text-center space-y-2">
      <h1 class="text-4xl font-bold tracking-tight">AutoVideo</h1>
      <p class="text-muted-foreground">
        Automated vlog editing powered by AI
      </p>
    </div>

    {#if error}
      <Alert variant="destructive">
        <AlertDescription>{error}</AlertDescription>
      </Alert>
    {/if}

    <!-- File Selection -->
    <Card>
      <CardHeader>
        <CardTitle>Select Video</CardTitle>
        <CardDescription>Choose a video file to process</CardDescription>
      </CardHeader>
      <CardContent class="space-y-4">
        {#if selectedFile}
          <div class="flex items-center gap-3 p-4 rounded-lg border bg-muted/50">
            <FileVideo class="h-8 w-8 text-primary" />
            <div class="flex-1 min-w-0">
              <p class="font-medium truncate">{selectedFile.split('/').pop()}</p>
              <p class="text-sm text-muted-foreground truncate">{selectedFile}</p>
            </div>
            <Button
              onclick={clearFile}
              variant="ghost"
              size="icon"
              class="h-8 w-8"
            >
              <X class="h-4 w-4" />
            </Button>
          </div>

          <Button
            onclick={startProcessing}
            class="w-full"
            size="lg"
            disabled={isTranscribing}
          >
            {#if isTranscribing}
              <Loader2 class="h-4 w-4 mr-2 animate-spin" />
              Transcribing...
            {:else}
              Start Processing
            {/if}
          </Button>
        {:else}
          <FileDropZone
            onUpload={handleUpload}
            onFileRejected={handleFileRejected}
            maxFiles={1}
            accept={acceptedTypes}
          >
            <FileDropZoneTrigger />
          </FileDropZone>
        {/if}
      </CardContent>
    </Card>

    <!-- Configuration -->
    <ConfigPanel />

    <!-- LLM Cleanup (Optional) -->
    <Card>
      <CardHeader>
        <CardTitle>LLM Post-Processing (Optional)</CardTitle>
        <CardDescription>Clean up word fragments using Z.ai GLM-4.7-Flash</CardDescription>
      </CardHeader>
      <CardContent class="space-y-4">
        <div class="space-y-2">
          <label for="llm-api-key" class="text-sm font-medium">
            Z.ai API Key
          </label>
          <input
            id="llm-api-key"
            type="password"
            bind:value={llmApiKey}
            placeholder="Enter your Z.ai API key (optional)"
            class="flex h-9 w-full rounded-md border border-input bg-transparent px-3 py-1 text-sm shadow-sm transition-colors file:border-0 file:bg-transparent file:text-sm file:font-medium placeholder:text-muted-foreground focus-visible:outline-none focus-visible:ring-1 focus-visible:ring-ring disabled:cursor-not-allowed disabled:opacity-50"
          />
          <p class="text-xs text-muted-foreground">
            If provided, fixes word fragments like "dis av ivo" → "dispositivo"
          </p>
        </div>
      </CardContent>
    </Card>
  </div>
</div>
