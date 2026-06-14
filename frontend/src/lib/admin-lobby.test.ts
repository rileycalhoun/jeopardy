import { describe, expect, test } from 'vitest';

import { canAdminSelectClue, shouldRefreshAdminLobby } from './admin-lobby';

describe('shouldRefreshAdminLobby', () => {
	test('keeps polling the lobby before the game starts', () => {
		expect(shouldRefreshAdminLobby(null)).toBe(true);
	});

	test('stops polling the lobby after game state is available', () => {
		expect(shouldRefreshAdminLobby({ phase: 'RoundSelection' })).toBe(false);
	});
});

describe('canAdminSelectClue', () => {
	test('allows the moderator to select while the moderator holds control', () => {
		expect(canAdminSelectClue({ phase: 'RoundSelection', current_selector: null })).toBe(true);
	});

	test('prevents the moderator from selecting during a player turn', () => {
		expect(canAdminSelectClue({ phase: 'RoundSelection', current_selector: 7 })).toBe(false);
	});

	test('prevents selection outside the round selection phase', () => {
		expect(canAdminSelectClue({ phase: 'ClueOpen', current_selector: null })).toBe(false);
	});
});
