<script lang="ts">
	import { onMount } from 'svelte';

	import {
		finishGame,
		getAdminLobby,
		getAdminGameState,
		joinAdminGame,
		listCategories,
		resolveAnswer,
		selectClue,
		skipClue,
		startGame,
		type CategorySummary,
		type GameView,
		type Lobby
	} from '$lib/api/games';
	import { connectAdminGameSocket } from '$lib/api/realtime';
	import { canAdminSelectClue, shouldRefreshAdminLobby } from '$lib/admin-lobby';
	import JeopardyBoard from '$lib/components/JeopardyBoard.svelte';
	import Podium from '$lib/components/Podium.svelte';
	import QuestionModal from '$lib/components/QuestionModal.svelte';
	import LobbyRoster from '$lib/components/LobbyRoster.svelte';
	import Scoreboard from '$lib/components/Scoreboard.svelte';
	import { parseJoinCode } from '$lib/lobby';
	import type { FetchError } from '$lib/safe/fetch';
	import type { SafeWebSocket } from '$lib/safe/websocket';

	let { params } = $props();

	const FALLBACK_POLL_INTERVAL_MS = 5000;

	let adminCode = $state<number | null>(null);
	let lobby = $state<Lobby | null>(null);
	let categories = $state<CategorySummary[]>([]);
	let selectedCategoryIds = $state<string[]>([]);
	let game = $state<GameView | null>(null);
	let adminToken = $state('');
	let errorMessage = $state('');
	let infoMessage = $state('');
	let isLoading = $state(true);
	let isBusy = $state(false);
	let selectedPlayerIds = $state<number[]>([]);
	let connectionLabel = $state('Connecting');

	let socket: SafeWebSocket | null = null;
	let fallbackInterval: number | null = null;

	const currentRound = $derived(game?.board[game.current_round] ?? null);
	const isFinished = $derived(game?.phase === 'Completed');
	const adminCanSelectClue = $derived(game !== null && canAdminSelectClue(game));
	// Who controls the next clue pick, shown so the host knows whose turn it is.
	const selectorLabel = $derived.by(() => {
		if (!game || game.current_selector === null) return 'Moderator';
		const selectorId = game.current_selector;
		return game.players.find((player) => player.id === selectorId)?.name ?? 'Moderator';
	});
	const activeCategoryTitle = $derived(
		game?.active_clue
			? (game.board[game.active_clue.round_index]?.categories[game.active_clue.category_index]
					?.title ?? '')
			: ''
	);

	function tokenKey(code: number): string {
		return `jeopardy-admin-token-${code}`;
	}

	function toMessage(error: FetchError): string {
		switch (error.kind) {
			case 'NetworkError':
				return 'Could not reach the backend.';
			case 'HttpError':
				if (error.status === 404) return 'No active game state is available yet.';
				if (error.status === 401) return 'Host authorization is missing or expired.';
				if (error.status === 403) return "It's the player's turn to pick a clue.";
				if (error.status === 409) return 'This game has already started.';
				return `Server returned ${error.status}.`;
			case 'JsonParseError':
				return 'Server returned invalid JSON.';
			case 'ValidationError':
				return 'Server returned unexpected data.';
		}
	}

	function toggleCategory(categoryId: string, selected: boolean) {
		if (selected) {
			if (!selectedCategoryIds.includes(categoryId)) {
				selectedCategoryIds = [...selectedCategoryIds, categoryId];
			}
			return;
		}

		selectedCategoryIds = selectedCategoryIds.filter((id) => id !== categoryId);
	}

	function applyGameUpdate(view: GameView) {
		game = view;
		selectedPlayerIds = selectedPlayerIds.filter((playerId) =>
			view.active_clue?.submissions.some((submission) => submission.player_id === playerId)
		);
	}

	async function refreshState(code: number) {
		const result = await getAdminGameState(code);
		if (result.ok) {
			applyGameUpdate(result.value.game);
			errorMessage = '';
			return;
		}

		if (result.error.kind !== 'HttpError' || result.error.status !== 404) {
			errorMessage = toMessage(result.error);
		}
	}

	async function refreshLobby(code: number) {
		const result = await getAdminLobby(code);
		if (result.ok) {
			lobby = result.value;
			if (game === null) errorMessage = '';
			return;
		}

		errorMessage = toMessage(result.error);
	}

	function stopFallbackPolling() {
		if (fallbackInterval !== null) {
			window.clearInterval(fallbackInterval);
			fallbackInterval = null;
		}
	}

	// If the socket cannot recover, fall back to slow polling so the host
	// console never goes dead.
	function startFallbackPolling() {
		if (fallbackInterval !== null) return;
		fallbackInterval = window.setInterval(() => {
			if (adminCode === null) return;
			void refreshState(adminCode);
			if (shouldRefreshAdminLobby(game)) void refreshLobby(adminCode);
		}, FALLBACK_POLL_INTERVAL_MS);
	}

	function openSocket(code: number, token: string) {
		socket = connectAdminGameSocket(code, token, {
			onLobby: (players) => {
				lobby = { players };
			},
			onGameState: (view) => {
				applyGameUpdate(view);
				errorMessage = '';
			},
			onGameFinished: () => {
				game = null;
				infoMessage = 'Game completed.';
			},
			onServerError: (message) => {
				errorMessage =
					message === 'invalid_admin_token' || message === 'missing_admin_token'
						? 'Host authorization is missing or expired.'
						: `Live connection rejected: ${message}.`;
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

	async function load() {
		const parsedCode = parseJoinCode(params.admin_code);
		if (parsedCode === null) {
			errorMessage = 'That admin code is not valid.';
			isLoading = false;
			return;
		}
		adminCode = parsedCode;
		adminToken = localStorage.getItem(tokenKey(parsedCode)) ?? '';

		const [lobbyResult, categoriesResult] = await Promise.all([
			joinAdminGame({ admin_code: parsedCode }),
			listCategories()
		]);

		if (lobbyResult.ok) {
			lobby = lobbyResult.value;
			if (lobbyResult.value.admin_token) {
				adminToken = lobbyResult.value.admin_token;
				localStorage.setItem(tokenKey(parsedCode), adminToken);
			}
		} else {
			errorMessage = toMessage(lobbyResult.error);
		}

		if (categoriesResult.ok) {
			categories = categoriesResult.value.categories;
			selectedCategoryIds = categories
				.slice(0, Math.min(categories.length, 6))
				.map((category) => category.id);
		} else if (!errorMessage) {
			errorMessage = toMessage(categoriesResult.error);
		}

		await refreshState(parsedCode);
		isLoading = false;

		if (adminToken) {
			openSocket(parsedCode, adminToken);
		} else {
			connectionLabel = 'Polling';
			startFallbackPolling();
		}
	}

	async function startSelectedCategories() {
		if (adminCode === null || !adminToken || selectedCategoryIds.length === 0) return;
		isBusy = true;
		const result = await startGame(adminCode, adminToken, selectedCategoryIds);
		isBusy = false;
		if (result.ok) {
			applyGameUpdate(result.value.game);
			errorMessage = '';
			infoMessage = '';
			return;
		}

		errorMessage = toMessage(result.error);
	}

	async function chooseClue(categoryIndex: number, clueIndex: number) {
		if (adminCode === null || !adminToken || !game || !canAdminSelectClue(game)) return;
		const result = await selectClue(adminCode, adminToken, categoryIndex, clueIndex);
		if (result.ok) {
			applyGameUpdate(result.value.game);
			errorMessage = '';
			return;
		}
		errorMessage = toMessage(result.error);
	}

	async function markAnswersCorrect() {
		if (adminCode === null || !adminToken || !game?.active_clue) return;
		const playerIds = game.active_clue.submissions
			.filter((submission) => selectedPlayerIds.includes(submission.player_id))
			.map((submission) => submission.player_id);
		if (playerIds.length === 0) return;
		isBusy = true;
		const result = await resolveAnswer(adminCode, adminToken, playerIds);
		isBusy = false;
		if (result.ok) {
			applyGameUpdate(result.value.game);
			errorMessage = '';
			return;
		}
		errorMessage = toMessage(result.error);
	}

	async function skipActiveClue() {
		if (adminCode === null || !adminToken || !game?.active_clue) return;
		isBusy = true;
		const result = await skipClue(adminCode, adminToken);
		isBusy = false;
		if (result.ok) {
			applyGameUpdate(result.value.game);
			errorMessage = '';
			return;
		}
		errorMessage = toMessage(result.error);
	}

	async function finish() {
		if (adminCode === null || !adminToken) return;
		const result = await finishGame(adminCode, adminToken);
		if (result.ok) {
			game = null;
			infoMessage = 'Game completed.';
			errorMessage = '';
			return;
		}
		errorMessage = toMessage(result.error);
	}

	onMount(() => {
		void load();
		return () => {
			socket?.close();
			stopFallbackPolling();
		};
	});
</script>

<div class="min-h-screen bg-board-deep px-6 py-8 text-white">
	<div class="mx-auto flex max-w-7xl flex-col gap-6">
		<header
			class="flex flex-col gap-4 border-b border-gold/20 pb-5 md:flex-row md:items-end md:justify-between"
		>
			<div>
				<p class="show-eyebrow">Host Console</p>
				<h1 class="mt-2 font-display text-4xl font-bold tracking-wide text-white uppercase">
					Game {params.admin_code}
				</h1>
			</div>
			<div class="flex items-center gap-3">
				<span
					class="rounded-full border border-gold/30 px-3 py-1 font-display text-xs tracking-widest text-gold-soft uppercase"
				>
					{connectionLabel}
				</span>
				<button class="show-button-outline" onclick={finish}>Finish Game</button>
			</div>
		</header>

		{#if isLoading}
			<p class="show-panel">Loading...</p>
		{:else}
			{#if errorMessage}
				<p class="rounded-md border border-red-400/40 bg-red-950/40 px-4 py-3 text-red-100">
					{errorMessage}
				</p>
			{/if}
			{#if infoMessage}
				<p class="rounded-md border border-gold/30 bg-gold/10 px-4 py-3 text-gold-soft">
					{infoMessage}
				</p>
			{/if}

			{#if game === null}
				<section class="grid gap-6 lg:grid-cols-[1fr_24rem]">
					<div class="show-panel">
						<h2 class="font-display text-xl font-bold tracking-wide uppercase">Start Game</h2>
						<p class="mt-2 text-sm text-white/70">
							Pick the categories for this game and open the board once your contestants are in.
						</p>
						<div class="mt-4 grid gap-3 sm:grid-cols-2">
							{#each categories as category (category.id)}
								<label
									class="rounded-md border border-white/10 bg-board-deep/70 p-3 transition data-[selected=true]:border-gold data-[selected=true]:bg-gold/10"
									data-selected={selectedCategoryIds.includes(category.id)}
								>
									<span class="flex items-start gap-3">
										<input
											type="checkbox"
											class="mt-1 h-4 w-4 accent-gold"
											checked={selectedCategoryIds.includes(category.id)}
											onchange={(event) => toggleCategory(category.id, event.currentTarget.checked)}
										/>
										<span>
											<span class="block font-display text-sm font-bold text-gold-soft uppercase">
												{category.title}
											</span>
											{#if category.description}
												<span class="mt-1 block text-sm text-white/60">{category.description}</span>
											{/if}
										</span>
									</span>
								</label>
							{/each}
							{#if categories.length === 0}
								<p
									class="rounded-md border border-white/10 bg-board-deep/70 p-3 text-sm text-white/60"
								>
									No playable categories are available.
								</p>
							{/if}
						</div>
						<div class="mt-4 flex items-center gap-3">
							<button
								class="show-button-gold"
								disabled={isBusy || selectedCategoryIds.length === 0}
								onclick={startSelectedCategories}
							>
								Start
							</button>
							<p class="text-sm text-white/60">
								{selectedCategoryIds.length} selected
							</p>
						</div>
					</div>
					{#if lobby}
						<LobbyRoster
							eyebrow="Lobby"
							heading={`${lobby.players.length} player${lobby.players.length === 1 ? '' : 's'} ready`}
							message="Players can stay on their lobby page; it will switch to the board after start."
							statusLabel="Waiting"
							players={lobby.players}
						/>
					{/if}
				</section>
			{:else if isFinished}
				<Podium players={game.players} />
			{:else}
				<section class="grid gap-6 lg:grid-cols-[1fr_22rem]">
					{#if currentRound}
						<JeopardyBoard
							round={currentRound}
							interactive
							locked={!adminCanSelectClue}
							onSelect={chooseClue}
						/>
					{/if}

					<aside class="flex flex-col gap-4">
						<div class="show-panel">
							<h2 class="show-eyebrow">Scores</h2>
							<div class="mt-3">
								<Scoreboard players={game.players} currentSelector={game.current_selector} />
							</div>
						</div>

						<div class="show-panel">
							<p class="show-eyebrow">Phase</p>
							<p class="mt-1 font-display text-lg font-bold tracking-wide uppercase">
								{game.phase}
							</p>
						</div>

						{#if game.phase === 'RoundSelection'}
							<div class="show-panel">
								<p class="show-eyebrow">Picking Next</p>
								<p
									class="mt-1 font-display text-lg font-bold tracking-wide text-gold-soft uppercase"
								>
									{selectorLabel}
								</p>
								<p class="mt-2 text-xs text-white/50">
									{game.current_selector === null
										? 'Choose the next clue from the board.'
										: 'Only the selected player can choose the next clue.'}
								</p>
							</div>
						{/if}
					</aside>
				</section>

				{#if game.active_clue}
					<QuestionModal
						open
						clue={game.active_clue}
						categoryTitle={activeCategoryTitle}
						showAnswer
					>
						<div class="flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-center">
							<button
								class="show-button-gold bg-emerald-400 hover:bg-emerald-300"
								disabled={isBusy || selectedPlayerIds.length === 0}
								onclick={markAnswersCorrect}
							>
								Correct
							</button>
							<button
								class="show-button-outline"
								disabled={isBusy}
								onclick={skipActiveClue}
							>
								Nobody was Correct
							</button>
						</div>
						{#if game.active_clue.submissions.length > 0}
							<div class="mt-6">
								<h3 class="show-eyebrow">Answer Cards</h3>
								<div class="mt-3 grid gap-3 md:grid-cols-2">
									{#each game.active_clue.submissions as submission (submission.player_id)}
										<button
											class="min-h-28 rounded-md border border-white/10 bg-board-deep/80 p-4 text-left transition hover:border-gold/60 data-[selected=true]:border-gold data-[selected=true]:bg-gold/10"
											data-selected={selectedPlayerIds.includes(submission.player_id)}
											aria-pressed={selectedPlayerIds.includes(submission.player_id)}
											onclick={() =>
												(selectedPlayerIds = selectedPlayerIds.includes(submission.player_id)
													? selectedPlayerIds.filter((id) => id !== submission.player_id)
													: [...selectedPlayerIds, submission.player_id])}
										>
											<span class="block font-display text-sm font-bold text-gold-soft uppercase">
												{submission.player_name}
											</span>
											<span class="mt-2 block text-lg text-white">{submission.answer}</span>
										</button>
									{/each}
								</div>
							</div>
						{:else}
							<div
								class="mt-6 rounded-md border border-white/10 bg-board-deep/60 p-4 text-center text-sm text-white/60"
							>
								Submitted answers will appear here as cards.
							</div>
						{/if}
					</QuestionModal>
				{/if}
			{/if}
		{/if}
	</div>
</div>
