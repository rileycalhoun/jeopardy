import { safeFetchJson, type FetchError } from '$lib/safe/fetch';
import { type Result } from 'ripthrow';
import { z } from 'zod';

import { PUBLIC_API_URL } from '$env/static/public';

const GameSchema = z.object({
	game_id: z.string(),
	player_code: z.int32(),
	admin_code: z.int32()
});

export type Game = z.infer<typeof GameSchema>;

export function newGame(): Promise<Result<Game, FetchError>> {
	return safeFetchJson(`${PUBLIC_API_URL}/games/new`, GameSchema, {
		method: 'POST'
	});
}
