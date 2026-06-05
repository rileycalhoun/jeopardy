<script lang="ts">
	import { onMount } from 'svelte';

	import {
		getPlayerGameState,
		getPlayerLobby,
		submitPlayerAnswer,
		type GameView,
		type Lobby
	} from '$lib/api/games';
	import LobbyRoster from '$lib/components/LobbyRoster.svelte';
	import { parseJoinCode } from '$lib/lobby';
	import type { FetchError } from '$lib/safe/fetch';

	let { params } = $props();

	let playerCode = $state<number | null>(null);
	let lobby = $state<Lobby | null>(null);
	let game = $state<GameView | null>(null);
	let currentPlayerId = $state<number | null>(null);
	let answerInput = $state('');
	let answerMessage = $state('');
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

	async function refreshLobby(code: number) {
		const result = await getPlayerLobby(code);
		if (result.ok) {
			lobby = result.value;
			if (!game) errorMessage = '';
			return;
		}
		errorMessage = toMessage(result.error);
	}

	async function refreshGame(code: number) {
		const result = await getPlayerGameState(code);
		if (result.ok) {
			game = result.value.game;
			errorMessage = '';
			return;
		}
		if (result.error.kind !== 'HttpError' || result.error.status !== 404) {
			errorMessage = toMessage(result.error);
		}
	}

	onMount(() => {
		const parsedCode = parseJoinCode(params.player_code);
		if (parsedCode === null) {
			errorMessage = 'That player code is not valid.';
			isLoading = false;
			return;
		}
		playerCode = parsedCode;
		const storedPlayerId = localStorage.getItem(`jeopardy-player-id-${parsedCode}`);
		currentPlayerId = storedPlayerId === null ? null : Number(storedPlayerId);

		void Promise.all([refreshLobby(parsedCode), refreshGame(parsedCode)]).finally(() => {
			isLoading = false;
		});

		const interval = window.setInterval(() => {
			if (playerCode === null) return;
			void refreshGame(playerCode);
			if (!game) void refreshLobby(playerCode);
		}, 2500);

		return () => window.clearInterval(interval);
	});

	async function sendAnswer() {
		if (playerCode === null || currentPlayerId === null || answerInput.trim() === '') return;

		const result = await submitPlayerAnswer(playerCode, currentPlayerId, answerInput);
		if (result.ok) {
			game = result.value.game;
			answerMessage = 'Answer submitted.';
			errorMessage = '';
			return;
		}

		answerMessage = '';
		errorMessage = toMessage(result.error);
	}
</script>

<div class="min-h-screen bg-slate-950 px-6 py-8 text-stone-100">
	<div class="mx-auto flex max-w-7xl flex-col gap-6">
		<header class="border-b border-white/10 pb-5">
			<p class="text-sm tracking-[0.35em] text-sky-300 uppercase">Player</p>
			<h1 class="mt-2 text-4xl font-semibold text-white">Game {params.player_code}</h1>
		</header>

		{#if isLoading}
			<p class="rounded-md border border-white/10 bg-white/5 px-4 py-3">Loading...</p>
		{:else if errorMessage}
			<p class="rounded-md border border-rose-400/30 bg-rose-950/40 px-4 py-3 text-rose-100">
				{errorMessage}
			</p>
		{/if}

		{#if game}
			<section class="grid gap-6 lg:grid-cols-[1fr_20rem]">
				<div class="overflow-x-auto">
					<div class="grid min-w-[720px] gap-2" style={`grid-template-columns: repeat(${game.board[game.current_round]?.categories.length ?? 1}, minmax(0, 1fr));`}>
						{#each game.board[game.current_round]?.categories ?? [] as category}
							<div class="grid gap-2">
								<div class="flex min-h-20 items-center justify-center rounded-md bg-blue-900 p-3 text-center font-semibold uppercase">{category.title}</div>
								{#each category.clues as clue}
									<div
										class="flex min-h-20 items-center justify-center rounded-md border border-blue-300/20 bg-blue-800 p-3 text-2xl font-bold text-amber-200 data-[answered=true]:bg-slate-800 data-[answered=true]:text-slate-500"
										data-answered={clue.answered}
									>
										{clue.answered ? '' : clue.label}
									</div>
								{/each}
							</div>
						{/each}
					</div>
				</div>

				<aside class="flex flex-col gap-4">
					<div class="rounded-md border border-white/10 bg-white/5 p-4">
						<h2 class="font-semibold">Scoreboard</h2>
						<div class="mt-3 space-y-2">
							{#each game.players as player}
								<div class="flex justify-between gap-3 rounded bg-slate-900 px-3 py-2">
									<span>{player.name}</span>
									<span class="font-mono">{player.score}</span>
								</div>
							{/each}
						</div>
					</div>

					<div class="rounded-md border border-white/10 bg-white/5 p-4">
						<p class="text-sm text-slate-300">Status</p>
						<p class="mt-1 font-semibold">{game.phase === 'RoundSelection' ? 'Waiting for host' : game.phase}</p>
					</div>
				</aside>
			</section>

			{#if game.active_clue}
				<section class="rounded-md border border-sky-300/25 bg-slate-900 p-5">
					<p class="text-sm text-sky-200">{game.active_clue.label} · {game.active_clue.value}</p>
					<h2 class="mt-2 text-2xl font-semibold">{game.active_clue.question}</h2>
					{#if currentPlayerId === null}
						<p class="mt-3 text-slate-300">Rejoin from the home page to submit answers from this browser.</p>
					{:else}
						<div class="mt-4 flex flex-col gap-3 sm:flex-row">
							<input
								bind:value={answerInput}
								class="min-w-0 flex-1 rounded-md border border-white/10 bg-slate-950 px-3 py-2 text-white"
								placeholder="Your answer"
							/>
							<button class="rounded-md bg-sky-300 px-4 py-2 font-semibold text-slate-950" onclick={sendAnswer}>
								Submit
							</button>
						</div>
						{#if answerMessage}
							<p class="mt-3 text-sm text-sky-200">{answerMessage}</p>
						{/if}
					{/if}
				</section>
			{/if}
		{:else if lobby}
			<LobbyRoster
				eyebrow="Players"
				heading={`${lobby.players.length} joined`}
				message="Wait for the host to choose a pack and open the board."
				statusLabel="Waiting for host"
				players={lobby.players}
			/>
		{/if}
	</div>
</div>
