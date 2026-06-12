import { describe, expect, test } from 'vitest';

import { ServerMessageSchema } from './realtime';

const minimalGameView = {
	phase: 'RoundSelection',
	current_round: 0,
	current_selector: 7,
	players: [{ id: 7, name: 'Ada', score: 0 }],
	board: [
		{
			name: 'Jeopardy',
			categories: [
				{
					title: 'Rust',
					clues: [{ label: '$200', value: 200, answered: false, daily_double: false }]
				}
			]
		}
	],
	active_clue: null,
	final_jeopardy: null
};

describe('ServerMessageSchema', () => {
	test('accepts lobby updates', () => {
		const parsed = ServerMessageSchema.parse({
			type: 'lobby',
			players: [{ id: 1, display_name: 'Ada' }]
		});
		expect(parsed.type).toBe('lobby');
	});

	test('accepts game state updates', () => {
		const parsed = ServerMessageSchema.parse({ type: 'game_state', game: minimalGameView });
		expect(parsed.type).toBe('game_state');
		if (parsed.type === 'game_state') {
			expect(parsed.game.players[0].name).toBe('Ada');
		}
	});

	test('accepts finish, pong, and error messages', () => {
		expect(ServerMessageSchema.parse({ type: 'game_finished' }).type).toBe('game_finished');
		expect(ServerMessageSchema.parse({ type: 'pong' }).type).toBe('pong');

		const parsed = ServerMessageSchema.parse({ type: 'error', message: 'invalid_admin_token' });
		expect(parsed.type).toBe('error');
	});

	test('rejects unknown message types', () => {
		const result = ServerMessageSchema.safeParse({ type: 'mystery' });
		expect(result.success).toBe(false);
	});

	test('rejects malformed game state payloads', () => {
		const result = ServerMessageSchema.safeParse({ type: 'game_state', game: { phase: 1 } });
		expect(result.success).toBe(false);
	});
});
