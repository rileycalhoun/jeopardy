import { safeFetchJson, type FetchError } from '$lib/safe/fetch';
import { type Result } from 'ripthrow';
import { z } from 'zod';

import { env } from '$env/dynamic/public';

const CreateGameSchema = z.object({
	player_code: z.int32(),
	admin_code: z.int32()
});

const PlayerSummarySchema = z.object({
	id: z.number().int(),
	display_name: z.string()
});

const LobbySchema = z.object({
	players: z.array(PlayerSummarySchema)
});

export type JoinPlayerRequest = {
	player_code: number;
	display_name: string;
};

export type JoinAdminRequest = {
	admin_code: number;
};

export type Lobby = z.infer<typeof LobbySchema>;
export type CreateGameResponse = z.infer<typeof CreateGameSchema>;

export function newGame(): Promise<Result<CreateGameResponse, FetchError>> {
	return safeFetchJson(`${env.PUBLIC_API_URL}/games/new`, CreateGameSchema, {
		method: 'POST'
	});
}

export function joinPlayerGame(payload: JoinPlayerRequest): Promise<Result<Lobby, FetchError>> {
	return safeFetchJson(`${env.PUBLIC_API_URL}/games/join/player`, LobbySchema, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify(payload)
	});
}

export function joinAdminGame(payload: JoinAdminRequest): Promise<Result<Lobby, FetchError>> {
	return safeFetchJson(`${env.PUBLIC_API_URL}/games/join/admin`, LobbySchema, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify(payload)
	});
}

export function getPlayerLobby(playerCode: number): Promise<Result<Lobby, FetchError>> {
	return safeFetchJson(`${env.PUBLIC_API_URL}/games/player/${playerCode}`, LobbySchema);
}

export function getAdminLobby(adminCode: number): Promise<Result<Lobby, FetchError>> {
	return safeFetchJson(`${env.PUBLIC_API_URL}/games/admin/${adminCode}`, LobbySchema);
}
