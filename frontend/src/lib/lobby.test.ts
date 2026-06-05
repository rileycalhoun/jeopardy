import { describe, expect, test } from 'vitest';

import { adminLobbyPath, parseJoinCode, playerLobbyPath } from './lobby';

describe('parseJoinCode', () => {
	test('returns an integer for a clean numeric code', () => {
		expect(parseJoinCode('654321')).toBe(654321);
	});

	test('trims surrounding whitespace before parsing', () => {
		expect(parseJoinCode(' 123456 ')).toBe(123456);
	});

	test('rejects non-integer input', () => {
		expect(parseJoinCode('12a456')).toBeNull();
		expect(parseJoinCode('')).toBeNull();
		expect(parseJoinCode('12.5')).toBeNull();
	});
});

describe('lobby paths', () => {
	test('builds the player lobby path from a player code', () => {
		expect(playerLobbyPath(654321)).toBe('/lobby/player/654321');
	});

	test('builds the admin lobby path from an admin code', () => {
		expect(adminLobbyPath(123456)).toBe('/lobby/admin/123456');
	});
});
