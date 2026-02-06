import { writable, derived, type Writable, type Readable } from 'svelte/store';
import type { TranscriptResult, Word } from '$lib/types/pipeline';

// Individual stores for editor state
export const transcript: Writable<TranscriptResult | null> = writable(null);
export const deletedWordIds: Writable<Set<string>> = writable(new Set());
export const currentTime: Writable<number> = writable(0);
export const inputPath: Writable<string | null> = writable(null);

// Derived store: compute keep ranges from non-deleted words
export const keepRanges: Readable<[number, number][]> = derived(
  [transcript, deletedWordIds],
  ([$transcript, $deletedWordIds]) => {
    if (!$transcript || !$transcript.words || $transcript.words.length === 0) {
      return [];
    }

    // Filter to get non-deleted words
    const keptWords = $transcript.words.filter(
      (word: Word) => !$deletedWordIds.has(word.id)
    );

    if (keptWords.length === 0) {
      return [];
    }

    // Sort words by start time
    const sortedWords = [...keptWords].sort((a, b) => a.start - b.start);

    // Create initial ranges from words
    const ranges: { start: number; end: number }[] = sortedWords.map((word: Word) => ({
      start: word.start,
      end: word.end
    }));

    // Merge adjacent ranges where gap is less than 100ms (0.1 seconds)
    const mergedRanges: { start: number; end: number }[] = [];
    const GAP_THRESHOLD = 0.1; // 100ms

    for (const range of ranges) {
      if (mergedRanges.length === 0) {
        mergedRanges.push({ ...range });
      } else {
        const lastRange = mergedRanges[mergedRanges.length - 1];
        const gap = range.start - lastRange.end;

        if (gap < GAP_THRESHOLD) {
          // Merge: extend the last range
          lastRange.end = Math.max(lastRange.end, range.end);
        } else {
          // Gap too large: start a new range
          mergedRanges.push({ ...range });
        }
      }
    }

    // Convert to [start, end] tuples
    return mergedRanges.map((range): [number, number] => [range.start, range.end]);
  }
);

// Actions
export function setTranscript(result: TranscriptResult): void {
  transcript.set(result);
  inputPath.set(result.input_path);
  deletedWordIds.set(new Set());
  currentTime.set(0);
}

export function toggleWord(wordId: string): void {
  deletedWordIds.update((ids) => {
    const newIds = new Set(ids);
    if (newIds.has(wordId)) {
      console.log('✅ Restoring word:', wordId);
      newIds.delete(wordId);
    } else {
      console.log('❌ Deleting word:', wordId);
      newIds.add(wordId);
    }
    console.log('📊 Total deleted words:', newIds.size);
    return newIds;
  });
}

export function deleteWord(wordId: string): void {
  deletedWordIds.update((ids) => {
    const newIds = new Set(ids);
    newIds.add(wordId);
    return newIds;
  });
}

export function restoreWord(wordId: string): void {
  deletedWordIds.update((ids) => {
    const newIds = new Set(ids);
    newIds.delete(wordId);
    return newIds;
  });
}

export function restoreAll(): void {
  deletedWordIds.set(new Set());
}

export function setCurrentTime(time: number): void {
  currentTime.set(time);
}

export function reset(): void {
  transcript.set(null);
  deletedWordIds.set(new Set());
  currentTime.set(0);
  inputPath.set(null);
}
