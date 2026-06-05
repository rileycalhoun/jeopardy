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

<section class="mt-8 rounded-[2rem] border bg-slate-950/50 p-6 shadow-2xl shadow-slate-950/30">
	<div class="flex items-end justify-between gap-4">
		<div>
			<p class="text-sm tracking-[0.3em] uppercase">{eyebrow}</p>
			<h2 class="mt-2 text-3xl font-semibold text-white">{heading}</h2>
			<p class="mt-3 max-w-2xl text-slate-300">{message}</p>
		</div>
		<p class="rounded-full border px-4 py-2 text-sm">{statusLabel}</p>
	</div>

	{#if players.length === 0}
		<p
			class="mt-6 rounded-3xl border border-dashed border-white/15 bg-slate-900/40 px-4 py-5 text-slate-300"
		>
			No players have joined yet.
		</p>
	{:else}
		<ul class="mt-6 grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
			{#each players as player (player.id)}
				<li class="rounded-3xl border border-white/10 bg-slate-900/70 px-4 py-4">
					<p class="text-xs tracking-[0.3em] text-slate-400 uppercase">Player</p>
					<p class="mt-2 text-xl font-medium break-words text-white">{player.display_name}</p>
				</li>
			{/each}
		</ul>
	{/if}
</section>
