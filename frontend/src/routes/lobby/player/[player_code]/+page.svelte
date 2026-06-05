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
	let lastActiveClueKey = $state<string | null>(null);
	let errorMessage = $state('');
	let isLoading = $state(true);
	let activeClueKey = $derived(
		game?.active_clue
			? `${game.active_clue.round_index}:${game.active_clue.category_index}:${game.active_clue.clue_index}`
			: null
	);

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
			if (result.value.current_player_id !== undefined) {
				currentPlayerId = result.value.current_player_id;
				localStorage.setItem(`jeopardy-player-id-${code}`, `${result.value.current_player_id}`);
			}
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

	$effect(() => {
		if (activeClueKey !== lastActiveClueKey) {
			lastActiveClueKey = activeClueKey;
			answerInput = '';
			answerMessage = '';
		}
	});

	async function sendAnswer() {
		if (playerCode === null || currentPlayerId === null) {
			answerMessage = 'Choose your player name before submitting.';
			return;
		}

		if (answerInput.trim() === '') {
			answerMessage = 'Enter an answer before submitting.';
			return;
		}

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

	function choosePlayer(playerId: number) {
		currentPlayerId = playerId;
		if (playerCode !== null) {
			localStorage.setItem(`jeopardy-player-id-${playerCode}`, `${playerId}`);
		}
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

					<div class="rounded-md border border-sky-300/25 bg-white/5 p-4">
						<h2 class="font-semibold">Submit Answer</h2>
						{#if currentPlayerId === null}
							<label class="mt-3 block text-sm text-slate-300" for="player-identity">Player</label>
							<select
								id="player-identity"
								class="mt-1 w-full rounded-md border border-white/10 bg-slate-950 px-3 py-2 text-white"
								onchange={(event) => choosePlayer(Number(event.currentTarget.value))}
							>
								<option value="">Choose your name</option>
								{#each game.players as player}
									<option value={player.id}>{player.name}</option>
								{/each}
							</select>
						{:else}
							<p class="mt-2 text-sm text-slate-300">
								Playing as {game.players.find((player) => player.id === currentPlayerId)?.name ?? 'selected player'}
							</p>
						{/if}

						<textarea
							bind:value={answerInput}
							class="mt-3 min-h-24 w-full resize-y rounded-md border border-white/10 bg-slate-950 px-3 py-2 text-white disabled:opacity-50"
							disabled={!game.active_clue}
							placeholder={game.active_clue ? 'Type your response' : 'Waiting for the host to select a clue'}
						></textarea>
						<button
							class="mt-3 w-full rounded-md bg-sky-300 px-4 py-2 font-semibold text-slate-950 disabled:opacity-50"
							disabled={!game.active_clue}
							onclick={sendAnswer}
						>
							Submit Answer
						</button>
						{#if answerMessage}
							<p class="mt-3 text-sm text-sky-200">{answerMessage}</p>
						{/if}
					</div>
				</aside>
			</section>

			{#if game.active_clue}
				<section class="rounded-md border border-sky-300/25 bg-slate-900 p-5">
					<p class="text-sm text-sky-200">{game.active_clue.label} · {game.active_clue.value}</p>
					<h2 class="mt-2 text-2xl font-semibold">{game.active_clue.question}</h2>
					<p class="mt-3 text-slate-300">Use the answer box beside the scoreboard to submit your response.</p>
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
