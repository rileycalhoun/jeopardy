<script lang="ts">
	export type LobbyPlayer = {
		id: number;
		display_name: string;
	};

	let {
		eyebrow,
		heading,
		message,
		statusLabel,
		players
	}: {
		eyebrow: string;
		heading: string;
		message: string;
		statusLabel: string;
		players: LobbyPlayer[];
	} = $props();
</script>

<section class="show-panel">
	<div class="flex items-end justify-between gap-4">
		<div>
			<p class="show-eyebrow">{eyebrow}</p>
			<h2 class="mt-2 font-display text-3xl font-bold text-white uppercase">{heading}</h2>
			<p class="mt-3 max-w-2xl text-white/70">{message}</p>
		</div>
		<p
			class="rounded-full border border-gold/40 px-4 py-2 font-display text-sm tracking-wider text-gold-soft uppercase"
		>
			{statusLabel}
		</p>
	</div>

	{#if players.length === 0}
		<p
			class="mt-6 rounded-lg border border-dashed border-gold/20 bg-board-deep/60 px-4 py-5 text-white/70"
		>
			No players have joined yet.
		</p>
	{:else}
		<ul class="mt-6 grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
			{#each players as player (player.id)}
				<!-- Mini contestant podium. -->
				<li
					class="overflow-hidden rounded-md border-2 border-black/60 bg-gradient-to-b from-board-dark to-board px-4 py-4 shadow-md shadow-black/40"
				>
					<p class="font-display text-xs tracking-[0.3em] text-gold-soft uppercase">Contestant</p>
					<p class="mt-2 font-display text-xl font-bold tracking-wide break-words text-white">
						{player.display_name}
					</p>
				</li>
			{/each}
		</ul>
	{/if}
</section>
