<script lang="ts">
  import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
  import { Progress } from "@/components/ui/progress";
  import StageCard from "./StageCard.svelte";
  import { pipelineStore } from "@/lib/stores/pipeline";

  let { stages, selectedFile } = $derived($pipelineStore);

  let completedCount = $derived(stages.filter(s => s.status === 'completed').length);
  let totalStages = $derived(stages.length);
  let overallProgress = $derived((completedCount / totalStages) * 100);
  let activeStage = $derived(stages.find(s => s.status === 'active'));
</script>

<div class="container mx-auto px-4 py-8 max-w-4xl">
  <div class="space-y-8">
    <!-- Header -->
    <div class="text-center space-y-2">
      <h1 class="text-4xl font-bold tracking-tight">Processing Video</h1>
      <p class="text-muted-foreground">
        {selectedFile?.split('/').pop() || 'Video'}
      </p>
    </div>

    <!-- Overall Progress -->
    <Card>
      <CardHeader>
        <CardTitle>Overall Progress</CardTitle>
        <CardDescription>
          {#if activeStage}
            Currently {activeStage.label.toLowerCase()}...
          {:else if completedCount === totalStages}
            All stages completed!
          {:else}
            Preparing to process...
          {/if}
        </CardDescription>
      </CardHeader>
      <CardContent class="space-y-2">
        <Progress value={overallProgress} class="h-2" />
        <p class="text-sm text-muted-foreground text-right">
          {completedCount} of {totalStages} stages completed
        </p>
      </CardContent>
    </Card>

    <!-- Stage Cards -->
    <div class="space-y-4">
      {#each stages as stage (stage.id)}
        <StageCard {stage} />
      {/each}
    </div>
  </div>
</div>
