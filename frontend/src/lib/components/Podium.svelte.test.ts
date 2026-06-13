import { render } from 'vitest-browser-svelte';
import { expect, test } from 'vitest';

import Podium from './Podium.svelte';

type PlayerScore = { id: number; name: string; score: number };

function players(...entries: PlayerScore[]): PlayerScore[] {
	return entries;
}

test('ranks the top three contestants by score', async () => {
	const screen = await render(Podium, {
		players: players(
			{ id: 1, name: 'Bronwyn', score: 300 },
			{ id: 2, name: 'Aldous', score: 900 },
			{ id: 3, name: 'Celeste', score: 600 }
		)
	});

	await expect.element(screen.getByText('Final Results')).toBeVisible();
	// Winner is called out in the celebratory subtitle.
	await expect.element(screen.getByText(/Aldous takes the crown/)).toBeVisible();
	// Runner-up and third place still render with their scores.
	await expect.element(screen.getByText('Celeste')).toBeVisible();
	await expect.element(screen.getByText('Bronwyn')).toBeVisible();
	await expect.element(screen.getByText('$600')).toBeVisible();
	await expect.element(screen.getByText('$300')).toBeVisible();
});

test('shows only the top three when more contestants exist', async () => {
	const screen = await render(Podium, {
		players: players(
			{ id: 1, name: 'First', score: 1000 },
			{ id: 2, name: 'Second', score: 800 },
			{ id: 3, name: 'Third', score: 500 },
			{ id: 4, name: 'FourthPlace', score: 100 }
		)
	});

	await expect.element(screen.getByText('Third')).toBeVisible();
	// The fourth-place contestant is not part of the podium.
	await expect.element(screen.getByText('FourthPlace')).not.toBeInTheDocument();
});

test('renders fewer than three contestants without crashing', async () => {
	const screen = await render(Podium, {
		players: players({ id: 1, name: 'Solo', score: 400 }, { id: 2, name: 'Duo', score: 200 })
	});

	// Champion is announced in the subtitle; the runner-up renders on the podium.
	await expect.element(screen.getByText(/Solo takes the crown/)).toBeVisible();
	await expect.element(screen.getByText('Duo')).toBeVisible();
	await expect.element(screen.getByText('$200')).toBeVisible();
});

test('formats negative final scores', async () => {
	const screen = await render(Podium, {
		players: players(
			{ id: 1, name: 'Leader', score: 400 },
			{ id: 2, name: 'Middle', score: 100 },
			{ id: 3, name: 'Underwater', score: -500 }
		)
	});

	// The third-place negative score appears once, on its podium card.
	await expect.element(screen.getByText('-$500')).toBeVisible();
});
