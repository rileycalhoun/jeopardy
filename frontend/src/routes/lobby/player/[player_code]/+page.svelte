<script lang="ts">
	import { onMount } from 'svelte';

	import {
		getPlayerGameState,
		getPlayerLobby,
		submitPlayerAnswer,
		type GameView,
		type Lobby
	} from '$lib/api/games';
	import { connectPlayerGameSocket } from '$lib/api/realtime';
	import ActiveCluePanel from '$lib/components/ActiveCluePanel.svelte';
	import JeopardyBoard from '$lib/components/JeopardyBoard.svelte';
	import LobbyRoster from '$lib/components/LobbyRoster.svelte';
	import Scoreboard from '$lib/components/Scoreboard.svelte';
	import { parseJoinCode } from '$lib/lobby';
	import type { FetchError } from '$lib/safe/fetch';
	import type { SafeWebSocket } from '$lib/safe/websocket';

	let { params } = $props();

	const FALLBACK_POLL_INTERVAL_MS = 5000;

	let playerCode = $state<number | null>(null);
	let lobby = $state<Lobby | null>(null);
	let game = $state<GameView | null>(null);
	let currentPlayerId = $state<number | null>(null);
	let answerInput = $state('');
	let answerMessage = $state('');
	let infoMessage = $state('');
	let lastActiveClueKey = $state<string | null>(null);
	let errorMessage = $state('');
	let isLoading = $state(true);
	let connectionLabel = $state('Connecting');

	let socket: SafeWebSocket | null = null;
	let fallbackInterval: number | null = null;

	let activeClueKey = $derived(
		game?.active_clue
			? `${game.active_clue.round_index}:${game.active_clue.category_index}:${game.active_clue.clue_index}`
			: null
	);
	const currentRound = $derived(game?.board[game.current_round] ?? null);
	const activeCategoryTitle = $derived(
		game?.active_clue
			? (game.board[game.active_clue.round_index]?.categories[game.active_clue.category_index]
					?.title ?? '')
			: ''
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

	function stopFallbackPolling() {
		if (fallbackInterval !== null) {
			window.clearInterval(fallbackInterval);
			fallbackInterval = null;
		}
	}

	// If the socket cannot recover, fall back to slow polling so the player
	// screen never goes dead.
	function startFallbackPolling() {
		if (fallbackInterval !== null) return;
		fallbackInterval = window.setInterval(() => {
			if (playerCode === null) return;
			void refreshGame(playerCode);
			if (!game) void refreshLobby(playerCode);
		}, FALLBACK_POLL_INTERVAL_MS);
	}

	function openSocket(code: number) {
		socket = connectPlayerGameSocket(code, {
			onLobby: (players) => {
				lobby = { players };
			},
			onGameState: (view) => {
				game = view;
				errorMessage = '';
			},
			onGameFinished: () => {
				game = null;
				infoMessage = 'The host finished the game. Thanks for playing!';
			},
			onServerError: (message) => {
				errorMessage = `Live connection rejected: ${message}.`;
			},
			onStateChange: (state) => {
				switch (state) {
					case 'open':
						connectionLabel = 'Live';
						stopFallbackPolling();
						break;
					case 'connecting':
					case 'reconnecting':
						connectionLabel = 'Reconnecting';
						break;
					case 'failed':
						connectionLabel = 'Polling';
						startFallbackPolling();
						break;
					case 'closed':
						connectionLabel = 'Offline';
						break;
				}
			}
		});
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

		openSocket(parsedCode);

		return () => {
			socket?.close();
			stopFallbackPolling();
		};
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

<div class="min-h-screen bg-board-deep px-6 py-8 text-white">
	<div class="mx-auto flex max-w-7xl flex-col gap-6">
		<header class="flex items-end justify-between gap-4 border-b border-gold/20 pb-5">
			<div>
				<p class="show-eyebrow">Contestant</p>
				<h1 class="mt-2 font-display text-4xl font-bold tracking-wide text-white uppercase">
					Game {params.player_code}
				</h1>
			</div>
			<span
				class="rounded-full border border-gold/30 px-3 py-1 font-display text-xs tracking-widest text-gold-soft uppercase"
			>
				{connectionLabel}
			</span>
		</header>

		{#if isLoading}
			<p class="show-panel">Loading...</p>
		{:else if errorMessage}
			<p class="rounded-md border border-red-400/40 bg-red-950/40 px-4 py-3 text-red-100">
				{errorMessage}
			</p>
		{/if}
		{#if infoMessage}
			<p class="rounded-md border border-gold/30 bg-gold/10 px-4 py-3 text-gold-soft">
				{infoMessage}
			</p>
		{/if}

		{#if game}
			<section class="grid gap-6 lg:grid-cols-[1fr_22rem]">
				{#if currentRound}
					<JeopardyBoard round={currentRound} />
				{/if}

				<aside class="flex flex-col gap-4">
					<div class="show-panel">
						<h2 class="show-eyebrow">Scores</h2>
						<div class="mt-3">
							<Scoreboard players={game.players} currentSelector={game.current_selector} />
						</div>
					</div>

					<div class="show-panel">
						<p class="show-eyebrow">Status</p>
						<p class="mt-1 font-display text-lg font-bold tracking-wide uppercase">
							{game.phase === 'RoundSelection' ? 'Waiting for host' : game.phase}
						</p>
					</div>

					<div class="show-panel">
						<h2 class="show-eyebrow">Submit Answer</h2>
						{#if currentPlayerId === null}
							<label class="mt-3 block text-sm text-white/70" for="player-identity">Player</label>
							<select
								id="player-identity"
								class="show-input mt-1 w-full"
								onchange={(event) => choosePlayer(Number(event.currentTarget.value))}
							>
								<option value="">Choose your name</option>
								{#each game.players as player (player.id)}
									<option value={player.id}>{player.name}</option>
								{/each}
							</select>
						{:else}
							<p class="mt-2 text-sm text-white/70">
								Playing as {game.players.find((player) => player.id === currentPlayerId)?.name ??
									'selected player'}
							</p>
						{/if}

						<textarea
							bind:value={answerInput}
							class="show-input mt-3 min-h-24 w-full resize-y disabled:opacity-50"
							disabled={!game.active_clue}
							placeholder={game.active_clue
								? 'Type your response'
								: 'Waiting for the host to select a clue'}
						></textarea>
						<button
							class="show-button-gold mt-3 w-full"
							disabled={!game.active_clue}
							onclick={sendAnswer}
						>
							Submit Answer
						</button>
						{#if answerMessage}
							<p class="mt-3 text-sm text-gold-soft">{answerMessage}</p>
						{/if}
					</div>
				</aside>
			</section>

			{#if game.active_clue}
				<ActiveCluePanel clue={game.active_clue} categoryTitle={activeCategoryTitle}>
					<p class="text-center text-sm text-white/70">
						Use the answer box beside the scoreboard to submit your response.
					</p>
				</ActiveCluePanel>
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
