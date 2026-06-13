<script lang="ts">
	import type { GameView } from '$lib/api/schemas';

	type PlayerScore = GameView['players'][number];

	let { players }: { players: PlayerScore[] } = $props();

	// Top three contestants, highest score first. A stable sort keeps roster
	// order for ties so the layout is deterministic.
	const ranked = $derived([...players].sort((a, b) => b.score - a.score).slice(0, 3));

	const champion = $derived(ranked[0] ?? null);

	// Visual left-to-right order places the winner on the tallest centre step:
	// 2nd · 1st · 3rd. Missing places (fewer than three players) drop out.
	const podiumOrder = $derived.by(() => {
		const [first, second, third] = ranked;
		const slots: { player: PlayerScore; place: 1 | 2 | 3 }[] = [];
		if (second) slots.push({ player: second, place: 2 });
		if (first) slots.push({ player: first, place: 1 });
		if (third) slots.push({ player: third, place: 3 });
		return slots;
	});

	function formatScore(score: number): string {
		return score < 0 ? `-$${Math.abs(score).toLocaleString()}` : `$${score.toLocaleString()}`;
	}

	const placeStyle: Record<1 | 2 | 3, { height: string; medal: string; pedestal: string }> = {
		1: {
			height: 'h-40 sm:h-56',
			medal: '🏆',
			pedestal: 'from-gold to-amber-600 text-board-deep'
		},
		2: {
			height: 'h-28 sm:h-40',
			medal: '🥈',
			pedestal: 'from-slate-300 to-slate-500 text-board-deep'
		},
		3: {
			height: 'h-20 sm:h-32',
			medal: '🥉',
			pedestal: 'from-amber-700 to-amber-900 text-white'
		}
	};
</script>

<section
	class="relative overflow-hidden rounded-lg border-4 border-gold/40 bg-board-edge p-6 shadow-2xl shadow-black/60 sm:p-10"
	aria-label="Final results"
>
	<!-- Celebratory glow behind the champion. -->
	<div
		class="pointer-events-none absolute inset-x-0 top-0 h-48 bg-gradient-to-b from-gold/20 to-transparent blur-2xl"
		aria-hidden="true"
	></div>

	<div class="relative flex flex-col items-center gap-2 text-center">
		<p class="show-eyebrow">Game Over</p>
		<h2 class="font-display text-3xl font-bold tracking-wide text-gold uppercase sm:text-5xl">
			Final Results
		</h2>
		{#if champion}
			<p class="mt-1 text-sm text-white/70 sm:text-base">
				<span class="font-display font-bold text-white">{champion.name}</span>
				takes the crown with {formatScore(champion.score)}
			</p>
		{/if}
	</div>

	{#if podiumOrder.length > 0}
		<div class="relative mt-10 flex items-end justify-center gap-3 sm:gap-6">
			{#each podiumOrder as slot (slot.player.id)}
				<div class="flex w-1/3 max-w-[12rem] flex-col items-center">
					<!-- Contestant card above the step. -->
					<div
						class="mb-3 flex flex-col items-center gap-1 text-center"
						data-champion={slot.place === 1}
					>
						<span class="text-3xl sm:text-4xl" aria-hidden="true"
							>{placeStyle[slot.place].medal}</span
						>
						<span
							class="max-w-full truncate font-display text-sm font-bold tracking-wide text-white uppercase sm:text-lg"
							title={slot.player.name}
						>
							{slot.player.name}
						</span>
						<span
							class="font-display text-lg font-bold {slot.player.score < 0
								? 'text-red-400'
								: 'text-gold-soft'} [text-shadow:0_0_8px_rgba(255,215,0,0.35)] sm:text-2xl"
						>
							{formatScore(slot.player.score)}
						</span>
					</div>

					<!-- The pedestal step. Height encodes the placement. -->
					<div
						class="flex w-full items-start justify-center rounded-t-md border-x-2 border-t-2 border-black/50 bg-gradient-to-b {placeStyle[
							slot.place
						].pedestal} {placeStyle[slot.place].height} shadow-lg shadow-black/50"
					>
						<span class="mt-3 font-display text-3xl font-black sm:mt-4 sm:text-5xl">
							{slot.place}
						</span>
					</div>
				</div>
			{/each}
		</div>
	{:else}
		<p class="mt-10 text-center text-white/70">No contestants to rank.</p>
	{/if}
</section>
