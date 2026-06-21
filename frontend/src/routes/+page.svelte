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
			if (result.value.current_player_id !== undefined) {
				localStorage.setItem(
					`jeopardy-player-id-${playerCode}`,
					`${result.value.current_player_id}`
				);
			}
			if (result.value.current_player_token !== undefined) {
				localStorage.setItem(
					`jeopardy-player-token-${playerCode}`,
					result.value.current_player_token
				);
			}
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
	<title>Jeopardy Lobby</title>
</svelte:head>

<div class="min-h-screen bg-board-deep px-6 py-12 text-white">
	<div class="mx-auto flex max-w-6xl flex-col gap-10">
		<section class="max-w-3xl">
			<p class="show-eyebrow">Quiz Night</p>
			<h1
				class="mt-4 font-display text-5xl leading-tight font-bold tracking-wide text-white uppercase [text-shadow:3px_3px_6px_rgba(0,0,0,0.8)] md:text-6xl"
			>
				This is... <span class="text-gold">the big board.</span>
			</h1>
			<p class="mt-4 max-w-2xl text-lg text-white/70">
				Create a room, join as a contestant, or moderate as host. Players enter with the player
				code; hosts moderate with the admin code.
			</p>
		</section>

		<section class="grid gap-6 lg:grid-cols-3">
			<div class="show-panel">
				<p class="show-eyebrow">Host</p>
				<h2 class="mt-3 font-display text-2xl font-bold text-white uppercase">Create a new game</h2>
				<p class="mt-2 text-sm text-white/70">
					Generate a fresh player code and admin code for the next lobby.
				</p>
				<button class="show-button-gold mt-6 w-full" onclick={createHostGame}>
					Create host game
				</button>

				{#if createError}
					<p
						class="mt-4 rounded-md border border-red-400/40 bg-red-950/40 px-4 py-3 text-sm text-red-100"
					>
						{createError}
					</p>
				{/if}

				{#if createdGame}
					<div class="mt-6 space-y-3 rounded-lg border border-gold/25 bg-board-deep/70 p-4">
						<div class="flex items-center justify-between gap-4">
							<span class="text-sm text-white/70">Admin code</span>
							<span class="font-display text-2xl font-bold tracking-[0.18em] text-gold">
								{createdGame.admin_code}
							</span>
						</div>
						<div class="flex items-center justify-between gap-4">
							<span class="text-sm text-white/70">Player code</span>
							<span class="font-display text-2xl font-bold tracking-[0.18em] text-white">
								{createdGame.player_code}
							</span>
						</div>
						<button class="show-button-outline mt-2 w-full" onclick={openHostLobby}>
							Open host lobby
						</button>
					</div>
				{/if}
			</div>

			<div class="show-panel">
				<p class="show-eyebrow">Contestant</p>
				<h2 class="mt-3 font-display text-2xl font-bold text-white uppercase">Join a lobby</h2>
				<p class="mt-2 text-sm text-white/70">
					Enter the player code and choose the name that will appear on your podium.
				</p>
				<div class="mt-6 space-y-3">
					<input bind:value={playerCodeInput} class="show-input w-full" placeholder="Player code" />
					<input
						bind:value={playerNameInput}
						class="show-input w-full"
						placeholder="Display name"
					/>
					<button class="show-button-gold w-full" onclick={joinAsPlayer}> Join as player </button>
				</div>

				{#if playerJoinError}
					<p
						class="mt-4 rounded-md border border-red-400/40 bg-red-950/40 px-4 py-3 text-sm text-red-100"
					>
						{playerJoinError}
					</p>
				{/if}
			</div>

			<div class="show-panel">
				<p class="show-eyebrow">Admin</p>
				<h2 class="mt-3 font-display text-2xl font-bold text-white uppercase">Moderate a game</h2>
				<p class="mt-2 text-sm text-white/70">
					Use the admin code to open the host lobby and watch players arrive.
				</p>
				<div class="mt-6 space-y-3">
					<input bind:value={adminCodeInput} class="show-input w-full" placeholder="Admin code" />
					<button class="show-button-outline w-full" onclick={joinAsAdmin}> Join as admin </button>
				</div>

				{#if adminJoinError}
					<p
						class="mt-4 rounded-md border border-red-400/40 bg-red-950/40 px-4 py-3 text-sm text-red-100"
					>
						{adminJoinError}
					</p>
				{/if}
			</div>
		</section>
	</div>
</div>
