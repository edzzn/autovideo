<script lang="ts">
  import { openUrl } from '@tauri-apps/plugin-opener';
  import { Button } from "@/components/ui/button";
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
  import { Separator } from "@/components/ui/separator";
  import { Badge } from "@/components/ui/badge";
  import TranscriptViewer from "./TranscriptViewer.svelte";
  import { pipelineStore } from "@/lib/stores/pipeline";
  import { stopListening } from "@/lib/utils/tauri";
  import { CheckCircle, FolderOpen, Play } from "lucide-svelte";

  let { result } = $derived($pipelineStore);

  function formatDuration(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}m ${secs}s`;
  }

  function formatFileSize(bytes: number): string {
    const mb = bytes / (1024 * 1024);
    return `${mb.toFixed(1)} MB`;
  }

  async function openVideo() {
    if (result?.output_path) {
      try {
        await openUrl(result.output_path);
      } catch (err) {
        console.error('Failed to open video:', err);
      }
    }
  }

  async function openFolder() {
    if (result?.output_path) {
      try {
        const folder = result.output_path.substring(0, result.output_path.lastIndexOf('/'));
        await openUrl(folder);
      } catch (err) {
        console.error('Failed to open folder:', err);
      }
    }
  }

  function processAnother() {
    stopListening();
    pipelineStore.reset();
  }
</script>

<div class="container mx-auto px-4 py-8 max-w-4xl">
  {#if result}
    <div class="space-y-8">
      <!-- Success Header -->
      <div class="text-center space-y-4">
        <div class="flex justify-center">
          <div class="rounded-full bg-green-500/10 p-3">
            <CheckCircle class="h-12 w-12 text-green-500" />
          </div>
        </div>
        <div class="space-y-2">
          <h1 class="text-4xl font-bold tracking-tight">Processing Complete!</h1>
          <p class="text-muted-foreground">
            Your video has been successfully processed
          </p>
        </div>
      </div>

      <!-- Stats -->
      <Card>
        <CardHeader>
          <CardTitle>Processing Statistics</CardTitle>
          <CardDescription>Summary of changes made to your video</CardDescription>
        </CardHeader>
        <CardContent class="space-y-4">
          <div class="grid grid-cols-2 gap-4">
            <div class="space-y-1">
              <p class="text-sm text-muted-foreground">Original Duration</p>
              <p class="text-2xl font-bold">{formatDuration(result.stats.original_duration)}</p>
            </div>
            <div class="space-y-1">
              <p class="text-sm text-muted-foreground">Final Duration</p>
              <p class="text-2xl font-bold">{formatDuration(result.stats.processed_duration)}</p>
            </div>
            <div class="space-y-1">
              <p class="text-sm text-muted-foreground">Silence Removed</p>
              <p class="text-2xl font-bold">{formatDuration(result.stats.removed_silence_duration)}</p>
            </div>
            <div class="space-y-1">
              <p class="text-sm text-muted-foreground">Time Saved</p>
              <p class="text-2xl font-bold">
                <Badge variant="default" class="text-lg px-3 py-1">
                  {result.stats.silence_percentage.toFixed(1)}%
                </Badge>
              </p>
            </div>
          </div>

          <Separator />

          <div class="space-y-2">
            <p class="text-sm text-muted-foreground">Output File</p>
            <p class="text-sm font-mono truncate">{result.output_path}</p>
            <p class="text-sm text-muted-foreground">
              Original size: {formatFileSize(result.stats.original_size_bytes)}
            </p>
          </div>
        </CardContent>
      </Card>

      <!-- Actions -->
      <Card>
        <CardHeader>
          <CardTitle>Next Steps</CardTitle>
        </CardHeader>
        <CardContent class="space-y-3">
          <Button onclick={openVideo} class="w-full" size="lg">
            <Play class="mr-2 h-4 w-4" />
            Open Video
          </Button>
          <Button onclick={openFolder} variant="outline" class="w-full">
            <FolderOpen class="mr-2 h-4 w-4" />
            Open Folder
          </Button>
          <Separator />
          <Button onclick={processAnother} variant="secondary" class="w-full">
            Process Another Video
          </Button>
        </CardContent>
      </Card>

      <!-- Transcript -->
      <TranscriptViewer transcript={result.transcript} />
    </div>
  {:else}
    <div class="text-center py-12">
      <p class="text-muted-foreground">No results available</p>
    </div>
  {/if}
</div>
