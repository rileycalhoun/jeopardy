<script lang="ts">
	import type { GameView } from '$lib/api/schemas';

	type PlayerScore = GameView['players'][number];

	let {
		players,
		currentSelector = null
	}: {
		players: PlayerScore[];
		/** Highlights the contestant who picks the next clue. */
		currentSelector?: number | null;
	} = $props();

	function formatScore(score: number): string {
		return score < 0 ? `-$${Math.abs(score).toLocaleString()}` : `$${score.toLocaleString()}`;
	}
</script>

<div class="grid grid-cols-2 gap-3 sm:grid-cols-3 lg:grid-cols-2">
	{#each players as player (player.id)}
		<!-- Styled like a contestant podium: score screen above the name plate. -->
		<div
			class="overflow-hidden rounded-md border-2 border-black/60 shadow-lg shadow-black/50 data-[selector=true]:ring-2 data-[selector=true]:ring-gold/70"
			data-selector={currentSelector === player.id}
		>
			<div class="bg-board-deep px-3 py-3 text-center">
				<span
					class="font-display text-2xl font-bold tracking-wide [text-shadow:0_0_8px_rgba(255,255,255,0.35)] {player.score <
					0
						? 'text-red-400'
						: 'text-white'}"
				>
					{formatScore(player.score)}
				</span>
			</div>
			<div
				class="border-t-2 border-black/50 bg-gradient-to-b from-board-dark to-board px-3 py-2 text-center"
			>
				<span class="font-display font-bold tracking-widest text-white uppercase">
					{player.name}
				</span>
			</div>
		</div>
	{/each}
</div>
