<script lang="ts">
	import QuestionModal from '$lib/components/QuestionModal.svelte';
	import type { GameView } from '$lib/api/schemas';

	type ActiveClue = NonNullable<GameView['active_clue']>;

	// A fixed clue plus a minimal answer form, used to test the modal's slot,
	// autofocus, and submit behavior without standing up a whole game page.
	let {
		open = true,
		dismissable = false,
		onClose,
		onSubmit
	}: {
		open?: boolean;
		dismissable?: boolean;
		onClose?: () => void;
		onSubmit?: (answer: string) => void;
	} = $props();

	const clue: ActiveClue = {
		round_index: 0,
		category_index: 0,
		clue_index: 0,
		label: '$400',
		value: 400,
		question: 'This planet is known as the Red Planet',
		answer: 'Mars',
		attempted_player_ids: [],
		submissions: []
	};

	let answer = $state('');
</script>

<QuestionModal {open} {clue} categoryTitle="SCIENCE" {dismissable} {onClose}>
	<form
		onsubmit={(event) => {
			event.preventDefault();
			onSubmit?.(answer);
		}}
	>
		<input data-autofocus aria-label="Your Response" bind:value={answer} />
		<button type="submit">Submit Answer</button>
	</form>
</QuestionModal>
