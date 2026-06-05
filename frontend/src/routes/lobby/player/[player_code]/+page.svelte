<script lang="ts">
	import { onMount } from 'svelte';

	import { getPlayerLobby, type Lobby } from '$lib/api/games';
	import LobbyRoster from '$lib/components/LobbyRoster.svelte';
	import { parseJoinCode } from '$lib/lobby';
	import type { FetchError } from '$lib/safe/fetch';

	let { params } = $props();

	let lobby = $state<Lobby | null>(null);
	let errorMessage = $state('');
	let isLoading = $state(true);

	function toMessage(error: FetchError): string {
		switch (error.kind) {
			case 'NetworkError':
				return 'Could not reach the backend.';
			case 'HttpError':
				return error.status === 404
					? 'No lobby matched that player code.'
					: `Server returned ${error.status}.`;
			case 'JsonParseError':
				return 'Server returned invalid JSON.';
			case 'ValidationError':
				return 'Server returned unexpected data.';
		}
	}

	onMount(async () => {
		const playerCode = parseJoinCode(params.player_code);

		if (playerCode === null) {
			isLoading = false;
			errorMessage = 'That player code is not valid.';
			return;
		}

		const result = await getPlayerLobby(playerCode);

		isLoading = false;

		if (result.ok) {
			lobby = result.value;
			errorMessage = '';
			return;
		}

		errorMessage = toMessage(result.error);
	});
</script>

<div
	class="min-h-screen bg-[linear-gradient(180deg,_#08121f,_#10243c_55%,_#0c1320)] px-6 py-12 text-stone-100"
>
	<div class="mx-auto max-w-4xl">
		<p class="text-sm tracking-[0.35em] text-sky-300 uppercase">Player Lobby</p>
		<h1 class="mt-4 text-4xl font-semibold text-white">Player code {params.player_code}</h1>
		<p class="mt-3 max-w-2xl text-slate-300">
			You are in the staging lobby. Wait for the host to finish gathering players and open the
			board.
		</p>

		{#if isLoading}
			<p class="mt-8 rounded-3xl border border-white/10 bg-slate-900/40 px-5 py-4 text-slate-300">
				Loading lobby...
			</p>
		{:else if errorMessage}
			<p class="mt-8 rounded-3xl border border-rose-400/30 bg-rose-950/40 px-5 py-4 text-rose-100">
				{errorMessage}
			</p>
		{:else if lobby}
			<div class="text-sky-300">
				<LobbyRoster
					eyebrow="Players"
					heading={`${lobby.players.length} joined`}
					message="You are in the staging lobby. Wait for the host to finish gathering players and open the board."
					statusLabel="Waiting for host"
					players={lobby.players}
				/>
			</div>
		{/if}
	</div>
</div>
