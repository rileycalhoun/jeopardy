<script lang="ts">
	import type { GameView } from '$lib/api/schemas';

	type Round = GameView['board'][number];

	let {
		round,
		interactive = false,
		locked = false,
		onSelect
	}: {
		round: Round;
		/** Hosts get clickable tiles; players get a read-only board. */
		interactive?: boolean;
		/** Disables tile selection while keeping the board visible. */
		locked?: boolean;
		onSelect?: (categoryIndex: number, clueIndex: number) => void;
	} = $props();

	const columns = $derived(round.categories.length || 1);
</script>

<div class="overflow-x-auto">
	<div
		class="grid min-w-[720px] gap-1.5 rounded-lg border-4 border-black/70 bg-board-edge p-1.5 shadow-2xl shadow-black/60"
		style={`grid-template-columns: repeat(${columns}, minmax(0, 1fr));`}
	>
		{#each round.categories as category, categoryIndex (categoryIndex)}
			<div class="grid gap-1.5">
				<div
					class="flex min-h-24 items-center justify-center border border-black/40 bg-board-dark p-3 text-center font-display text-lg leading-tight font-bold tracking-wide text-white uppercase shadow-inner [text-shadow:1px_1px_2px_rgba(0,0,0,0.8)]"
				>
					{category.title}
				</div>
				{#each category.clues as clue, clueIndex (clueIndex)}
					{#if interactive}
						<button
							class="min-h-20 border border-black/40 bg-board p-3 font-display text-3xl font-bold text-gold transition [text-shadow:2px_2px_3px_rgba(0,0,0,0.85)] hover:bg-board-dark disabled:cursor-not-allowed disabled:bg-board-deep disabled:text-transparent disabled:[text-shadow:none]"
							disabled={clue.answered || locked}
							onclick={() => onSelect?.(categoryIndex, clueIndex)}
						>
							{clue.answered ? '' : clue.label}
						</button>
					{:else}
						<div
							class="flex min-h-20 items-center justify-center border border-black/40 bg-board p-3 font-display text-3xl font-bold text-gold [text-shadow:2px_2px_3px_rgba(0,0,0,0.85)] data-[answered=true]:bg-board-deep data-[answered=true]:text-transparent data-[answered=true]:[text-shadow:none]"
							data-answered={clue.answered}
						>
							{clue.answered ? '' : clue.label}
						</div>
					{/if}
				{/each}
			</div>
		{/each}
	</div>
</div>
