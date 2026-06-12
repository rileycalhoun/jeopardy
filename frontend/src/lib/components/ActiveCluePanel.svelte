<script lang="ts">
	import type { Snippet } from 'svelte';

	import type { GameView } from '$lib/api/schemas';

	type ActiveClue = NonNullable<GameView['active_clue']>;

	let {
		clue,
		categoryTitle = '',
		showAnswer = false,
		children
	}: {
		clue: ActiveClue;
		categoryTitle?: string;
		/** Hosts see the expected answer; players never do. */
		showAnswer?: boolean;
		/** Extra content under the clue (host controls, player hints). */
		children?: Snippet;
	} = $props();
</script>

<!-- The active clue takes over like a zoomed-in board tile on TV. -->
<section
	class="rounded-lg border-4 border-black/70 bg-gradient-to-b from-board-dark to-board p-8 text-center shadow-2xl shadow-black/60"
>
	<p class="show-eyebrow">
		{#if categoryTitle}{categoryTitle} ·{/if}
		{clue.label}
	</p>
	<h2
		class="mx-auto mt-6 max-w-4xl font-display text-3xl leading-snug font-bold text-white uppercase [text-shadow:2px_2px_4px_rgba(0,0,0,0.8)] md:text-4xl"
	>
		{clue.question}
	</h2>
	{#if showAnswer && clue.answer}
		<p class="mt-6 text-lg text-gold-soft">
			Answer: <span class="font-semibold">{clue.answer}</span>
		</p>
	{/if}
	{#if children}
		<div class="mt-8 text-left">
			{@render children()}
		</div>
	{/if}
</section>
