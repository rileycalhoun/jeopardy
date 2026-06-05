<script lang="ts">
	import { onMount } from 'svelte';

	import { getAdminLobby, type Lobby } from '$lib/api/games';
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
					? 'No lobby matched that admin code.'
					: `Server returned ${error.status}.`;
			case 'JsonParseError':
				return 'Server returned invalid JSON.';
			case 'ValidationError':
				return 'Server returned unexpected data.';
		}
	}

	onMount(async () => {
		const adminCode = parseJoinCode(params.admin_code);

		if (adminCode === null) {
			isLoading = false;
			errorMessage = 'That admin code is not valid.';
			return;
		}

		const result = await getAdminLobby(adminCode);

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
	class="min-h-screen bg-[linear-gradient(180deg,_#170b21,_#231439_55%,_#0f1220)] px-6 py-12 text-stone-100"
>
	<div class="mx-auto max-w-5xl">
		<p class="text-sm tracking-[0.35em] text-violet-300 uppercase">Admin Lobby</p>
		<div class="mt-4 flex flex-col gap-4 md:flex-row md:items-end md:justify-between">
			<div>
				<h1 class="text-4xl font-semibold text-white">Admin code {params.admin_code}</h1>
				<p class="mt-3 max-w-2xl text-slate-300">
					This host view tracks everyone who has entered the room and gives you a clean place to
					start moderating the game.
				</p>
			</div>
			<div
				class="rounded-[1.5rem] border border-violet-300/20 bg-slate-950/50 px-5 py-4 text-sm text-violet-100"
			>
				Host controls are staged here for the next gameplay slice.
			</div>
		</div>

		{#if isLoading}
			<p class="mt-8 rounded-3xl border border-white/10 bg-slate-900/40 px-5 py-4 text-slate-300">
				Loading lobby...
			</p>
		{:else if errorMessage}
			<p class="mt-8 rounded-3xl border border-rose-400/30 bg-rose-950/40 px-5 py-4 text-rose-100">
				{errorMessage}
			</p>
		{:else if lobby}
			<div class="text-violet-300">
				<LobbyRoster
					eyebrow="Lobby status"
					heading={`${lobby.players.length} player${lobby.players.length === 1 ? '' : 's'} ready`}
					message="This host view tracks everyone who has entered the room and gives you a clean place to start moderating the game."
					statusLabel="Moderation standby"
					players={lobby.players}
				/>
			</div>
		{/if}
	</div>
</div>
