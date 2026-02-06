<script lang="ts">
  import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
  import { Separator } from "@/components/ui/separator";
  import type { Transcript } from "@/lib/types/pipeline";

  interface Props {
    transcript: Transcript;
  }

  let { transcript }: Props = $props();

  function formatTime(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }
</script>

<Card>
  <CardHeader>
    <CardTitle>Transcript</CardTitle>
    {#if transcript.language}
      <p class="text-sm text-muted-foreground">
        Detected language: {transcript.language}
      </p>
    {/if}
  </CardHeader>
  <CardContent>
    <div class="max-h-96 overflow-y-auto space-y-4">
      {#each transcript.segments as segment, index (segment.id)}
        {#if index > 0}
          <Separator />
        {/if}
        <div class="space-y-1">
          <div class="flex items-center gap-2">
            <span class="text-xs font-mono text-muted-foreground">
              {formatTime(segment.start)} - {formatTime(segment.end)}
            </span>
          </div>
          <p class="text-sm">{segment.text}</p>
        </div>
      {/each}

      {#if transcript.segments.length === 0}
        <p class="text-sm text-muted-foreground text-center py-4">
          No transcript segments found
        </p>
      {/if}
    </div>
  </CardContent>
</Card>
