<script lang="ts">
	import { convertFileSrc } from '@tauri-apps/api/core';
	import { setCurrentTime, transcript, deletedWordIds } from '$lib/stores/editor';

	let { inputPath, seekTime }: { inputPath: string; seekTime: number | null } = $props();

	let videoRef: HTMLVideoElement | null = $state(null);
	let error = $state<string | null>(null);
	let lastSeekTime = $state(0);

	// Use convertFileSrc with the asset protocol explicitly
	let videoSrc = $derived.by(() => {
		try {
			// Convert file path to asset protocol URL
			const assetUrl = convertFileSrc(inputPath, 'asset');
			console.log('📹 Converted source:', assetUrl);
			console.log('📁 Input path:', inputPath);
			return assetUrl;
		} catch (err) {
			console.error('❌ Error converting file src:', err);
			error = `Failed to convert file path: ${err}`;
			return '';
		}
	});

	// Effect to handle seeking when seekTime changes
	$effect(() => {
		if (seekTime !== null && videoRef) {
			videoRef.currentTime = seekTime;
		}
	});

	// Log deleted words changes for debugging
	$effect(() => {
		console.log('🗑️ Deleted words changed. Count:', $deletedWordIds.size);
		if ($transcript && $deletedWordIds.size > 0) {
			const deletedWords = $transcript.words.filter(w => $deletedWordIds.has(w.id));
			console.log('   First 5 deleted:', deletedWords.slice(0, 5).map(w => `"${w.word}" (${w.start.toFixed(2)}s-${w.end.toFixed(2)}s)`).join(', '));
		}
	});

	function handleTimeUpdate(e: Event) {
		const video = e.target as HTMLVideoElement;
		const currentTime = video.currentTime;

		setCurrentTime(currentTime);

		// Debug logging
		if (!$transcript) {
			console.log('⚠️ No transcript loaded');
			return;
		}

		if (video.paused) {
			// Don't skip when paused
			return;
		}

		// Check if current time is in a deleted word range
		const deletedWords = $transcript.words.filter(w => $deletedWordIds.has(w.id));

		if (deletedWords.length === 0) {
			return;
		}

		for (const word of deletedWords) {
			// If we're playing through a deleted section
			if (currentTime >= word.start && currentTime <= word.end) {
				// Prevent infinite loop by checking if we just sought
				if (Math.abs(currentTime - lastSeekTime) > 0.1) {
					// Find the next non-deleted section
					const nextTime = findNextNonDeletedTime(word.end);
					if (nextTime !== null) {
						console.log(`⏭️ Skipping deleted word "${word.word}" (${word.start.toFixed(2)}s - ${word.end.toFixed(2)}s)`);
						console.log(`   Deleted words count: ${$deletedWordIds.size}`);
						console.log(`   Jumping to ${nextTime.toFixed(2)}s`);
						lastSeekTime = nextTime;
						video.currentTime = nextTime;
					}
				} else {
					console.log('⏸️ Skipping skip (too close to last seek)');
				}
				break;
			}
		}
	}

	function findNextNonDeletedTime(afterTime: number): number | null {
		if (!$transcript) return null;

		// Find all deleted ranges
		const deletedRanges = $transcript.words
			.filter(w => $deletedWordIds.has(w.id))
			.sort((a, b) => a.start - b.start);

		// Find the first non-deleted time after afterTime
		let checkTime = afterTime + 0.01; // Small buffer

		for (const word of deletedRanges) {
			if (checkTime >= word.start && checkTime <= word.end) {
				// Still in deleted range, jump past it
				checkTime = word.end + 0.01;
			}
		}

		return checkTime;
	}

	function handleError(e: Event) {
		const video = e.target as HTMLVideoElement;
		console.error('❌ Video load error:', video.error);
		console.error('   Error code:', video.error?.code);
		console.error('   Error message:', video.error?.message);
		error = `Failed to load video: ${video.error?.message || 'Unknown error'}`;
	}

	function handleLoadedMetadata() {
		console.log('✅ Video loaded successfully');
		error = null;
	}
</script>

<div class="video-preview">
	{#if error}
		<div class="p-4 bg-destructive/10 border border-destructive rounded-lg">
			<p class="text-destructive text-sm">{error}</p>
		</div>
	{/if}
	<video
		bind:this={videoRef}
		src={videoSrc}
		ontimeupdate={handleTimeUpdate}
		onerror={handleError}
		onloadedmetadata={handleLoadedMetadata}
		controls
		class="w-full rounded-lg bg-black"
	>
		<track kind="captions" />
	</video>
</div>

<style>
	.video-preview {
		width: 100%;
	}

	video {
		max-height: 70vh;
		object-fit: contain;
	}
</style>
