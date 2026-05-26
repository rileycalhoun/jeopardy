<script lang="ts">
	import { newGame } from '$lib/api/games';

	let game_id: string | undefined = $state(undefined);
	let admin_code: number | undefined = $state(undefined);
	let player_code: number | undefined = $state(undefined);
	let error_message = $state('');

	async function buttonClick() {
		let result = await newGame();
		if (result.ok) {
			game_id = result.value.game_id;
			admin_code = result.value.admin_code;
			player_code = result.value.player_code;
			error_message = '';
			return;
		}

		switch (result.error.kind) {
			case 'NetworkError':
				error_message = 'Could not reach the server.';
				break;
			case 'HttpError':
				error_message = `Server returned ${result.error.status}`;
				break;
			case 'JsonParseError':
				error_message = 'Server returned invalid JSON.';
				break;
			case 'ValidationError':
				error_message = 'Server returned unexpected data.';
				break;
		}
	}
</script>

<button onclick={buttonClick}>Create Game</button>

{#if error_message}
	<p>{error_message}</p>
{/if}

{#if game_id != undefined && admin_code != undefined && player_code != undefined}
	<p>Game ID: {game_id}</p>
	<p>Admin code: {admin_code}</p>
	<p>Player code: {player_code}</p>
{:else}
	<p>Click the button to create a game!</p>
{/if}
