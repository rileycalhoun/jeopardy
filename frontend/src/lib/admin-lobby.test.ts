import { describe, expect, test } from 'vitest';

import { shouldRefreshAdminLobby } from './admin-lobby';

describe('shouldRefreshAdminLobby', () => {
	test('keeps polling the lobby before the game starts', () => {
		expect(shouldRefreshAdminLobby(null)).toBe(true);
	});

	test('stops polling the lobby after game state is available', () => {
		expect(shouldRefreshAdminLobby({ phase: 'RoundSelection' })).toBe(false);
	});
});
