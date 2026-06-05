<script lang="ts">
	import { onMount } from 'svelte';

	import {
		finishGame,
		getAdminGameState,
		joinAdminGame,
		listQuestionPacks,
		resolveAnswer,
		selectClue,
		startGame,
		type GameView,
		type Lobby,
		type QuestionPack
	} from '$lib/api/games';
	import LobbyRoster from '$lib/components/LobbyRoster.svelte';
	import { parseJoinCode } from '$lib/lobby';
	import type { FetchError } from '$lib/safe/fetch';

	let { params } = $props();

	let adminCode = $state<number | null>(null);
	let lobby = $state<Lobby | null>(null);
	let packs = $state<QuestionPack[]>([]);
	let selectedPackId = $state('');
	let game = $state<GameView | null>(null);
	let adminToken = $state('');
	let errorMessage = $state('');
	let isLoading = $state(true);
	let isBusy = $state(false);
	let selectedPlayerId = $state<number | null>(null);

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
				if (error.status === 409) return 'This game has already started.';
				return `Server returned ${error.status}.`;
			case 'JsonParseError':
				return 'Server returned invalid JSON.';
			case 'ValidationError':
				return 'Server returned unexpected data.';
		}
	}

	async function refreshState(code: number) {
		const result = await getAdminGameState(code);
		if (result.ok) {
			game = result.value.game;
			selectedPlayerId = game.players[0]?.id ?? null;
			errorMessage = '';
			return;
		}

		if (result.error.kind !== 'HttpError' || result.error.status !== 404) {
			errorMessage = toMessage(result.error);
		}
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

		const [lobbyResult, packsResult] = await Promise.all([
			joinAdminGame({ admin_code: parsedCode }),
			listQuestionPacks()
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

		if (packsResult.ok) {
			packs = packsResult.value.packs;
			selectedPackId = packs[0]?.id ?? '';
		} else if (!errorMessage) {
			errorMessage = toMessage(packsResult.error);
		}

		await refreshState(parsedCode);
		isLoading = false;
	}

	async function startSelectedPack() {
		if (adminCode === null || !adminToken || !selectedPackId) return;
		isBusy = true;
		const result = await startGame(adminCode, adminToken, selectedPackId);
		isBusy = false;
		if (result.ok) {
			game = result.value.game;
			errorMessage = '';
			return;
		}
		errorMessage = toMessage(result.error);
	}

	async function chooseClue(categoryIndex: number, clueIndex: number) {
		if (adminCode === null || !adminToken || game?.phase !== 'RoundSelection') return;
		const result = await selectClue(adminCode, adminToken, categoryIndex, clueIndex);
		if (result.ok) {
			game = result.value.game;
			selectedPlayerId = game.players[0]?.id ?? null;
			errorMessage = '';
			return;
		}
		errorMessage = toMessage(result.error);
	}

	async function markAnswer(correct: boolean) {
		if (adminCode === null || !adminToken || selectedPlayerId === null) return;
		const result = await resolveAnswer(adminCode, adminToken, selectedPlayerId, correct);
		if (result.ok) {
			game = result.value.game;
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
			errorMessage = 'Game completed.';
			return;
		}
		errorMessage = toMessage(result.error);
	}

	onMount(() => {
		void load();
		const interval = window.setInterval(() => {
			if (adminCode !== null) void refreshState(adminCode);
		}, 2500);
		return () => window.clearInterval(interval);
	});
</script>

<div class="min-h-screen bg-slate-950 px-6 py-8 text-stone-100">
	<div class="mx-auto flex max-w-7xl flex-col gap-6">
		<header class="flex flex-col gap-4 border-b border-white/10 pb-5 md:flex-row md:items-end md:justify-between">
			<div>
				<p class="text-sm tracking-[0.35em] text-amber-300 uppercase">Host</p>
				<h1 class="mt-2 text-4xl font-semibold text-white">Game {params.admin_code}</h1>
			</div>
			<button class="rounded-md border border-white/20 px-4 py-2 text-sm" onclick={finish}>Finish game</button>
		</header>

		{#if isLoading}
			<p class="rounded-md border border-white/10 bg-white/5 px-4 py-3">Loading...</p>
		{:else}
			{#if errorMessage}
				<p class="rounded-md border border-amber-300/30 bg-amber-950/30 px-4 py-3 text-amber-100">
					{errorMessage}
				</p>
			{/if}

			{#if game === null}
				<section class="grid gap-6 lg:grid-cols-[1fr_22rem]">
					<div class="rounded-md border border-white/10 bg-white/5 p-5">
						<h2 class="text-xl font-semibold">Start game</h2>
						<div class="mt-4 flex flex-col gap-3 sm:flex-row">
							<select bind:value={selectedPackId} class="rounded-md border border-white/10 bg-slate-900 px-3 py-2 text-white">
								{#each packs as pack}
									<option value={pack.id}>{pack.title}</option>
								{/each}
							</select>
							<button
								class="rounded-md bg-amber-300 px-4 py-2 font-semibold text-slate-950 disabled:opacity-50"
								disabled={isBusy || !selectedPackId}
								onclick={startSelectedPack}
							>
								Start
							</button>
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
			{:else}
				<section class="grid gap-6 lg:grid-cols-[1fr_20rem]">
					<div class="overflow-x-auto">
						<div class="grid min-w-[720px] gap-2" style={`grid-template-columns: repeat(${game.board[game.current_round]?.categories.length ?? 1}, minmax(0, 1fr));`}>
							{#each game.board[game.current_round]?.categories ?? [] as category, categoryIndex}
								<div class="grid gap-2">
									<div class="flex min-h-20 items-center justify-center rounded-md bg-blue-900 p-3 text-center font-semibold uppercase">{category.title}</div>
									{#each category.clues as clue, clueIndex}
										<button
											class="min-h-20 rounded-md border border-blue-300/20 bg-blue-800 p-3 text-2xl font-bold text-amber-200 disabled:bg-slate-800 disabled:text-slate-500"
											disabled={clue.answered || game.phase !== 'RoundSelection'}
											onclick={() => chooseClue(categoryIndex, clueIndex)}
										>
											{clue.answered ? '' : clue.label}
										</button>
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
							<p class="text-sm text-slate-300">Phase</p>
							<p class="mt-1 font-semibold">{game.phase}</p>
						</div>
					</aside>
				</section>

				{#if game.active_clue}
					<section class="rounded-md border border-amber-300/25 bg-slate-900 p-5">
						<p class="text-sm text-amber-200">{game.active_clue.label} · {game.active_clue.value}</p>
						<h2 class="mt-2 text-2xl font-semibold">{game.active_clue.question}</h2>
						<p class="mt-3 text-slate-300">Answer: {game.active_clue.answer}</p>
						<div class="mt-5 flex flex-col gap-3 sm:flex-row sm:items-center">
							<select bind:value={selectedPlayerId} class="rounded-md border border-white/10 bg-slate-950 px-3 py-2 text-white">
								{#each game.players as player}
									<option value={player.id}>{player.name}</option>
								{/each}
							</select>
							<button class="rounded-md bg-emerald-300 px-4 py-2 font-semibold text-slate-950" onclick={() => markAnswer(true)}>Correct</button>
							<button class="rounded-md bg-rose-300 px-4 py-2 font-semibold text-slate-950" onclick={() => markAnswer(false)}>Incorrect</button>
						</div>
						{#if game.active_clue.submissions.length > 0}
							<div class="mt-5">
								<h3 class="text-sm font-semibold text-slate-300">Submissions</h3>
								<div class="mt-2 space-y-2">
									{#each game.active_clue.submissions as submission}
										<button
											class="w-full rounded-md border border-white/10 bg-slate-950 px-3 py-2 text-left hover:border-amber-300/60"
											onclick={() => (selectedPlayerId = submission.player_id)}
										>
											<span class="font-semibold">{submission.player_name}</span>
											<span class="ml-2 text-slate-300">{submission.answer}</span>
										</button>
									{/each}
								</div>
							</div>
						{/if}
					</section>
				{/if}
			{/if}
		{/if}
	</div>
</div>
