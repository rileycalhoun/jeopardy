import { safeFetchJson, type FetchError } from '$lib/safe/fetch';
import { type Result } from 'ripthrow';

import { env } from '$env/dynamic/public';

import {
	CreateGameSchema,
	FinishGameSchema,
	GameStateSchema,
	LobbySchema,
	PacksSchema,
	type CreateGameResponse,
	type GameView,
	type Lobby,
	type QuestionPack
} from '$lib/api/schemas';

export type { CreateGameResponse, GameView, Lobby, QuestionPack };

export type JoinPlayerRequest = {
	player_code: number;
	display_name: string;
};

export type JoinAdminRequest = {
	admin_code: number;
};

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

export function listQuestionPacks(): Promise<Result<{ packs: QuestionPack[] }, FetchError>> {
	return safeFetchJson(`${env.PUBLIC_API_URL}/games/packs`, PacksSchema);
}

export function startGame(
	adminCode: number,
	token: string,
	questionPackId: string
): Promise<Result<{ game: GameView }, FetchError>> {
	return safeFetchJson(`${env.PUBLIC_API_URL}/games/admin/${adminCode}/start`, GameStateSchema, {
		method: 'POST',
		headers: authJsonHeaders(token),
		body: JSON.stringify({ question_pack_id: questionPackId })
	});
}

export function getAdminGameState(
	adminCode: number
): Promise<Result<{ game: GameView }, FetchError>> {
	return safeFetchJson(`${env.PUBLIC_API_URL}/games/admin/${adminCode}/state`, GameStateSchema);
}

export function getPlayerGameState(
	playerCode: number
): Promise<Result<{ game: GameView }, FetchError>> {
	return safeFetchJson(`${env.PUBLIC_API_URL}/games/player/${playerCode}/state`, GameStateSchema);
}

export function selectClue(
	adminCode: number,
	token: string,
	categoryIndex: number,
	clueIndex: number
): Promise<Result<{ game: GameView }, FetchError>> {
	return safeFetchJson(
		`${env.PUBLIC_API_URL}/games/admin/${adminCode}/select-clue`,
		GameStateSchema,
		{
			method: 'POST',
			headers: authJsonHeaders(token),
			body: JSON.stringify({ category_index: categoryIndex, clue_index: clueIndex })
		}
	);
}

export function resolveAnswer(
	adminCode: number,
	token: string,
	playerId: number,
	correct: boolean
): Promise<Result<{ game: GameView }, FetchError>> {
	return safeFetchJson(`${env.PUBLIC_API_URL}/games/admin/${adminCode}/answer`, GameStateSchema, {
		method: 'POST',
		headers: authJsonHeaders(token),
		body: JSON.stringify({ player_id: playerId, correct })
	});
}

export function submitPlayerAnswer(
	playerCode: number,
	playerId: number,
	answer: string
): Promise<Result<{ game: GameView }, FetchError>> {
	return safeFetchJson(`${env.PUBLIC_API_URL}/games/player/${playerCode}/answer`, GameStateSchema, {
		method: 'POST',
		headers: {
			'Content-Type': 'application/json'
		},
		body: JSON.stringify({ player_id: playerId, answer })
	});
}

export function finishGame(
	adminCode: number,
	token: string
): Promise<Result<{ completed: boolean }, FetchError>> {
	return safeFetchJson(`${env.PUBLIC_API_URL}/games/admin/${adminCode}/finish`, FinishGameSchema, {
		method: 'POST',
		headers: {
			Authorization: `Bearer ${token}`
		}
	});
}

function authJsonHeaders(token: string): HeadersInit {
	return {
		'Content-Type': 'application/json',
		Authorization: `Bearer ${token}`
	};
}
