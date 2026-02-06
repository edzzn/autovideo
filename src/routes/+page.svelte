<script lang="ts">
  import { onDestroy } from 'svelte';
  import HomeScreen from '@/lib/components/HomeScreen.svelte';
  import ProcessingScreen from '@/lib/components/ProcessingScreen.svelte';
  import DoneScreen from '@/lib/components/DoneScreen.svelte';
  import EditorScreen from '$lib/components/editor/EditorScreen.svelte';
  import { Alert, AlertDescription, AlertTitle } from '@/components/ui/alert';
  import { pipelineStore } from '@/lib/stores/pipeline';
  import { stopListening } from '@/lib/utils/tauri';
  import { AlertCircle } from 'lucide-svelte';

  let { screen, error } = $derived($pipelineStore);

  onDestroy(() => {
    stopListening();
  });
</script>

<main class="min-h-screen bg-background">
  {#if error}
    <div class="container mx-auto px-4 py-4 max-w-4xl">
      <Alert variant="destructive">
        <AlertCircle class="h-4 w-4" />
        <AlertTitle>Error</AlertTitle>
        <AlertDescription>{error}</AlertDescription>
      </Alert>
    </div>
  {/if}

  {#if screen === 'home'}
    <HomeScreen />
  {:else if screen === 'processing'}
    <ProcessingScreen />
  {:else if screen === 'done'}
    <DoneScreen />
  {:else if screen === 'editor'}
    <EditorScreen />
  {/if}
</main>
