<script lang="ts">
  import type { Word } from '$lib/types/pipeline';

  interface Props {
    word: Word;
    isDeleted: boolean;
    isActive: boolean;
    onSeek?: (time: number) => void;
    onToggle?: (wordId: string) => void;
  }

  let { word, isDeleted, isActive, onSeek, onToggle }: Props = $props();

  function handleClick(event: MouseEvent) {
    if (event.shiftKey) {
      console.log('🔀 Toggling word:', word.id);
      onToggle?.(word.id);
    } else {
      console.log('⏩ Seeking to:', word.start);
      onSeek?.(word.start);
    }
  }

  let classes = $derived(
    [
      'inline px-0.5 py-0.5 rounded text-sm transition-colors cursor-pointer',
      'hover:bg-muted',
      'focus:outline-none focus:ring-2 focus:ring-ring focus:ring-offset-1',
      isDeleted && 'line-through opacity-50',
      isActive && 'bg-primary/20'
    ]
      .filter(Boolean)
      .join(' ')
  );
</script>

<button
  type="button"
  class={classes}
  onclick={handleClick}
  title="Click to seek, Shift+click to toggle deletion"
>
  {word.word}
</button>
