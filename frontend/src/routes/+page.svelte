<script lang="ts">
	import { PUBLIC_API_URL } from '$env/static/public';

	type GameData = {
		game_id: string;
		admin_code: number;
		player_code: number;
	};

	let game: GameData | null = $state(null);
	let error = $state('');

	async function createGame() {
		error = '';
		game = null;

		const res = await fetch(`${PUBLIC_API_URL}/games/new`);
		if (!res.ok) {
			error = 'Request failed with error code: ' + res.status;
			return;
		}

		game = (await res.json()) as GameData;
	}
</script>

<button onclick={createGame}>Create Game</button>

{#if error}
	<p>{error}</p>
{/if}

{#if game}
	<p>Game ID: {game.game_id}</p>
	<p>Admin code: {game.admin_code}</p>
	<p>Player code: {game.player_code}</p>
{/if}
