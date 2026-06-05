import { render } from 'vitest-browser-svelte';
import { expect, test } from 'vitest';

import LobbyRoster from './LobbyRoster.svelte';
import { generateLobbyFixture } from '$lib/test-fixtures/lobby';

test('renders empty lobby safely', async () => {
	const screen = await render(LobbyRoster, {
		eyebrow: 'Player Lobby',
		heading: 'Nobody Joined',
		message: 'Still waiting for players.',
		statusLabel: 'Waiting',
		players: []
	});

	await expect.element(screen.getByText('Nobody Joined')).toBeVisible();
	await expect.element(screen.getByText('No players have joined yet.')).toBeVisible();
});

for (const seed of [1, 7, 42, 99, 1337]) {
	test(`renders randomized lobby fixture for seed ${seed}`, async () => {
		const screen = await render(LobbyRoster, {
			eyebrow: 'Admin Lobby',
			heading: `Seed ${seed}`,
			message: 'Randomized render fixture.',
			statusLabel: 'Standby',
			players: generateLobbyFixture(seed)
		});

		await expect.element(screen.getByText(`Seed ${seed}`)).toBeVisible();
		await expect.element(screen.getByText('Standby')).toBeVisible();
	});
}
