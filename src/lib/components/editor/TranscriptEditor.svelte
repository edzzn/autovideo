<script lang="ts">
  import { Button } from '@/components/ui/button';
  import Word from '$lib/components/editor/Word.svelte';
  import {
    transcript,
    deletedWordIds,
    currentTime,
    keepRanges,
    toggleWord,
    restoreAll
  } from '$lib/stores/editor';
  import type { Action } from 'svelte/action';

  interface Props {
    onSeek?: (time: number) => void;
  }

  let { onSeek }: Props = $props();

  // Reference for the scrollable container
  let scrollContainer: HTMLDivElement | undefined = $state();
  let wordRefs = new Map<string, HTMLElement>();

  // Compute stats
  let originalDuration = $derived($transcript?.duration_seconds ?? 0);

  let editedDuration = $derived(
    $keepRanges.reduce((sum, [start, end]) => sum + (end - start), 0)
  );

  let deletedCount = $derived($deletedWordIds.size);

  let savedTime = $derived(originalDuration - editedDuration);

  let savedPercentage = $derived(
    originalDuration > 0 ? Math.round((savedTime / originalDuration) * 100) : 0
  );

  // Find active word based on current time
  let activeWordId = $derived.by(() => {
    if (!$transcript?.words) return null;
    const time = $currentTime;
    const activeWord = $transcript.words.find(
      (word) => time >= word.start && time <= word.end
    );
    return activeWord?.id ?? null;
  });

  // Auto-scroll to active word
  $effect(() => {
    if (activeWordId && scrollContainer) {
      const wordElement = wordRefs.get(activeWordId);
      if (wordElement) {
        wordElement.scrollIntoView({
          behavior: 'smooth',
          block: 'center',
          inline: 'nearest'
        });
      }
    }
  });

  function formatDuration(seconds: number): string {
    const mins = Math.floor(seconds / 60);
    const secs = Math.floor(seconds % 60);
    return `${mins}:${secs.toString().padStart(2, '0')}`;
  }

  function handleRestoreAll() {
    restoreAll();
  }

  // Action to register word element references for auto-scroll
  const trackWordRef: Action<HTMLElement, string> = (node, wordId) => {
    wordRefs.set(wordId, node);
    return {
      destroy() {
        wordRefs.delete(wordId);
      }
    };
  };
</script>

<div class="flex flex-col h-full">
  <!-- Toolbar -->
  <div class="flex items-center justify-between gap-4 p-3 border-b bg-muted/30">
    <div class="flex items-center gap-6 text-sm">
      <div class="flex flex-col">
        <span class="text-muted-foreground text-xs">Original</span>
        <span class="font-mono font-medium">{formatDuration(originalDuration)}</span>
      </div>
      <div class="flex flex-col">
        <span class="text-muted-foreground text-xs">Edited</span>
        <span class="font-mono font-medium text-primary">{formatDuration(editedDuration)}</span>
      </div>
      <div class="flex flex-col">
        <span class="text-muted-foreground text-xs">Saved</span>
        <span class="font-mono font-medium text-green-600">
          {formatDuration(savedTime)} ({savedPercentage}%)
        </span>
      </div>
      {#if deletedCount > 0}
        <div class="flex flex-col">
          <span class="text-muted-foreground text-xs">Deleted words</span>
          <span class="font-mono font-medium text-destructive">{deletedCount}</span>
        </div>
      {/if}
    </div>
    <div class="flex items-center gap-2">
      {#if deletedCount > 0}
        <Button variant="outline" size="sm" onclick={handleRestoreAll}>
          Restore All
        </Button>
      {/if}
    </div>
  </div>

  <!-- Scrollable word container -->
  <div
    bind:this={scrollContainer}
    class="flex-1 overflow-y-auto p-4"
  >
    {#if $transcript?.words && $transcript.words.length > 0}
      <div class="leading-relaxed">
        {#each $transcript.words as word (word.id)}
          {@const isDeleted = $deletedWordIds.has(word.id)}
          {@const isActive = activeWordId === word.id}
          <span
            class="inline"
            use:trackWordRef={word.id}
          >
            <Word
              {word}
              {isDeleted}
              {isActive}
              onSeek={(time) => onSeek?.(time)}
              onToggle={(wordId) => toggleWord(wordId)}
            />{' '}
          </span>
        {/each}
      </div>
    {:else}
      <div class="flex items-center justify-center h-full text-muted-foreground">
        <p>No transcript available</p>
      </div>
    {/if}
  </div>

  <!-- Help text -->
  <div class="p-2 border-t bg-muted/30">
    <p class="text-xs text-muted-foreground text-center">
      Click a word to seek, Shift+click to toggle deletion
    </p>
  </div>
</div>
