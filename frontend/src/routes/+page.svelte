<script lang="ts">
	import { goto } from '$app/navigation';
	import { resolve } from '$app/paths';
	import { joinAdminGame, joinPlayerGame, newGame, type CreateGameResponse } from '$lib/api/games';
	import { parseJoinCode } from '$lib/lobby';
	import type { FetchError } from '$lib/safe/fetch';

	let createdGame = $state<CreateGameResponse | null>(null);
	let createError = $state('');
	let playerCodeInput = $state('');
	let playerNameInput = $state('');
	let playerJoinError = $state('');
	let adminCodeInput = $state('');
	let adminJoinError = $state('');

	function errorMessage(error: FetchError): string {
		switch (error.kind) {
			case 'NetworkError':
				return 'Could not reach the server.';
			case 'HttpError':
				if (error.status === 404) {
					return 'No game matched that code.';
				}

				if (error.status === 409) {
					return 'That player name is already taken in this lobby.';
				}

				return `Server returned ${error.status}.`;
			case 'JsonParseError':
				return 'Server returned invalid JSON.';
			case 'ValidationError':
				return 'Server returned unexpected data.';
		}
	}

	async function createHostGame() {
		const result = await newGame();

		if (result.ok) {
			createdGame = result.value;
			createError = '';
			return;
		}

		createError = errorMessage(result.error);
	}

	async function joinAsPlayer() {
		const playerCode = parseJoinCode(playerCodeInput);

		if (playerCode === null) {
			playerJoinError = 'Player code must be a whole number.';
			return;
		}

		const displayName = playerNameInput.trim();

		if (displayName === '') {
			playerJoinError = 'Display name is required.';
			return;
		}

		const result = await joinPlayerGame({
			player_code: playerCode,
			display_name: displayName
		});

		if (result.ok) {
			playerJoinError = '';
			await goto(resolve('/lobby/player/[player_code]', { player_code: `${playerCode}` }));
			return;
		}

		playerJoinError = errorMessage(result.error);
	}

	async function joinAsAdmin() {
		const adminCode = parseJoinCode(adminCodeInput);

		if (adminCode === null) {
			adminJoinError = 'Admin code must be a whole number.';
			return;
		}

		const result = await joinAdminGame({
			admin_code: adminCode
		});

		if (result.ok) {
			adminJoinError = '';
			await goto(resolve('/lobby/admin/[admin_code]', { admin_code: `${adminCode}` }));
			return;
		}

		adminJoinError = errorMessage(result.error);
	}

	function openHostLobby() {
		if (createdGame === null) {
			return;
		}

		void goto(resolve('/lobby/admin/[admin_code]', { admin_code: `${createdGame.admin_code}` }));
	}
</script>

<svelte:head>
	<title>Jeopardy Clone Lobby</title>
</svelte:head>

<div
	class="min-h-screen bg-[radial-gradient(circle_at_top,_#ffe39a,_transparent_35%),linear-gradient(160deg,_#10141f,_#1b2740_45%,_#0d1118)] px-6 py-12 text-stone-100"
>
	<div class="mx-auto flex max-w-6xl flex-col gap-10">
		<section class="max-w-3xl">
			<p class="text-sm tracking-[0.35em] text-amber-300 uppercase">Jeopardy Clone</p>
			<h1 class="mt-4 font-serif text-5xl leading-tight text-white md:text-6xl">
				Create a room, join as a player, or moderate as host.
			</h1>
			<p class="mt-4 max-w-2xl text-lg text-slate-300">
				Players enter with the player code. Hosts moderate with the admin code. The internal game
				record stays off the public surface.
			</p>
		</section>

		<section class="grid gap-6 lg:grid-cols-3">
			<div
				class="rounded-[2rem] border border-amber-200/20 bg-slate-950/50 p-6 shadow-2xl shadow-slate-950/30 backdrop-blur"
			>
				<p class="text-xs tracking-[0.35em] text-amber-300 uppercase">Host</p>
				<h2 class="mt-3 text-2xl font-semibold text-white">Create a new game</h2>
				<p class="mt-2 text-sm text-slate-300">
					Generate a fresh player code and admin code for the next lobby.
				</p>
				<button
					class="mt-6 w-full rounded-full bg-amber-300 px-5 py-3 text-sm font-semibold text-slate-950 transition hover:bg-amber-200"
					onclick={createHostGame}
				>
					Create host game
				</button>

				{#if createError}
					<p
						class="mt-4 rounded-2xl border border-rose-400/30 bg-rose-950/40 px-4 py-3 text-sm text-rose-100"
					>
						{createError}
					</p>
				{/if}

				{#if createdGame}
					<div
						class="mt-6 space-y-3 rounded-[1.5rem] border border-amber-300/20 bg-slate-900/70 p-4"
					>
						<div class="flex items-center justify-between gap-4">
							<span class="text-sm text-slate-300">Admin code</span>
							<span class="font-mono text-2xl tracking-[0.18em] text-amber-200">
								{createdGame.admin_code}
							</span>
						</div>
						<div class="flex items-center justify-between gap-4">
							<span class="text-sm text-slate-300">Player code</span>
							<span class="font-mono text-2xl tracking-[0.18em] text-sky-200">
								{createdGame.player_code}
							</span>
						</div>
						<button
							class="mt-2 w-full rounded-full border border-amber-200/25 px-5 py-3 text-sm font-semibold text-white transition hover:border-amber-200/60 hover:bg-white/5"
							onclick={openHostLobby}
						>
							Open host lobby
						</button>
					</div>
				{/if}
			</div>

			<div
				class="rounded-[2rem] border border-sky-200/20 bg-slate-950/50 p-6 shadow-2xl shadow-slate-950/30 backdrop-blur"
			>
				<p class="text-xs tracking-[0.35em] text-sky-300 uppercase">Player</p>
				<h2 class="mt-3 text-2xl font-semibold text-white">Join a lobby</h2>
				<p class="mt-2 text-sm text-slate-300">
					Enter the player code and choose the name that will appear in the lobby.
				</p>
				<div class="mt-6 space-y-3">
					<input
						bind:value={playerCodeInput}
						class="w-full rounded-2xl border border-white/10 bg-slate-900/70 px-4 py-3 text-white transition outline-none placeholder:text-slate-500 focus:border-sky-300"
						placeholder="Player code"
					/>
					<input
						bind:value={playerNameInput}
						class="w-full rounded-2xl border border-white/10 bg-slate-900/70 px-4 py-3 text-white transition outline-none placeholder:text-slate-500 focus:border-sky-300"
						placeholder="Display name"
					/>
					<button
						class="w-full rounded-full bg-sky-300 px-5 py-3 text-sm font-semibold text-slate-950 transition hover:bg-sky-200"
						onclick={joinAsPlayer}
					>
						Join as player
					</button>
				</div>

				{#if playerJoinError}
					<p
						class="mt-4 rounded-2xl border border-rose-400/30 bg-rose-950/40 px-4 py-3 text-sm text-rose-100"
					>
						{playerJoinError}
					</p>
				{/if}
			</div>

			<div
				class="rounded-[2rem] border border-violet-200/20 bg-slate-950/50 p-6 shadow-2xl shadow-slate-950/30 backdrop-blur"
			>
				<p class="text-xs tracking-[0.35em] text-violet-300 uppercase">Admin</p>
				<h2 class="mt-3 text-2xl font-semibold text-white">Moderate a game</h2>
				<p class="mt-2 text-sm text-slate-300">
					Use the admin code to open the host lobby and watch players arrive.
				</p>
				<div class="mt-6 space-y-3">
					<input
						bind:value={adminCodeInput}
						class="w-full rounded-2xl border border-white/10 bg-slate-900/70 px-4 py-3 text-white transition outline-none placeholder:text-slate-500 focus:border-violet-300"
						placeholder="Admin code"
					/>
					<button
						class="w-full rounded-full bg-violet-300 px-5 py-3 text-sm font-semibold text-slate-950 transition hover:bg-violet-200"
						onclick={joinAsAdmin}
					>
						Join as admin
					</button>
				</div>

				{#if adminJoinError}
					<p
						class="mt-4 rounded-2xl border border-rose-400/30 bg-rose-950/40 px-4 py-3 text-sm text-rose-100"
					>
						{adminJoinError}
					</p>
				{/if}
			</div>
		</section>
	</div>
</div>
