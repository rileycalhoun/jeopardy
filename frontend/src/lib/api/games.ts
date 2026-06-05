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
	players: z.array(PlayerSummarySchema),
	admin_token: z.string().optional(),
	current_player_id: z.number().int().optional()
});

const QuestionPackSchema = z.object({
	id: z.string(),
	title: z.string()
});

const PacksSchema = z.object({
	packs: z.array(QuestionPackSchema)
});

const PlayerScoreSchema = z.object({
	id: z.number().int(),
	name: z.string(),
	score: z.number().int()
});

const ClueTileSchema = z.object({
	label: z.string(),
	value: z.number().int(),
	answered: z.boolean(),
	daily_double: z.boolean()
});

const CategorySchema = z.object({
	title: z.string(),
	clues: z.array(ClueTileSchema)
});

const RoundSchema = z.object({
	name: z.string(),
	categories: z.array(CategorySchema)
});

const ActiveClueSchema = z.object({
	round_index: z.number().int(),
	category_index: z.number().int(),
	clue_index: z.number().int(),
	label: z.string(),
	value: z.number().int(),
	question: z.string(),
	answer: z.string(),
	attempted_player_ids: z.array(z.number().int()),
	submissions: z.array(
		z.object({
			player_id: z.number().int(),
			player_name: z.string(),
			answer: z.string()
		})
	)
});

const FinalJeopardySchema = z.object({
	category: z.string(),
	question: z.string(),
	answer: z.string()
});

const GameViewSchema = z.object({
	phase: z.string(),
	current_round: z.number().int(),
	current_selector: z.number().int(),
	players: z.array(PlayerScoreSchema),
	board: z.array(RoundSchema),
	active_clue: ActiveClueSchema.nullable(),
	final_jeopardy: FinalJeopardySchema.nullable()
});

const GameStateSchema = z.object({
	game: GameViewSchema
});

const FinishGameSchema = z.object({
	completed: z.boolean()
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
export type QuestionPack = z.infer<typeof QuestionPackSchema>;
export type GameView = z.infer<typeof GameViewSchema>;

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

export function getAdminGameState(adminCode: number): Promise<Result<{ game: GameView }, FetchError>> {
	return safeFetchJson(`${env.PUBLIC_API_URL}/games/admin/${adminCode}/state`, GameStateSchema);
}

export function getPlayerGameState(playerCode: number): Promise<Result<{ game: GameView }, FetchError>> {
	return safeFetchJson(`${env.PUBLIC_API_URL}/games/player/${playerCode}/state`, GameStateSchema);
}

export function selectClue(
	adminCode: number,
	token: string,
	categoryIndex: number,
	clueIndex: number
): Promise<Result<{ game: GameView }, FetchError>> {
	return safeFetchJson(`${env.PUBLIC_API_URL}/games/admin/${adminCode}/select-clue`, GameStateSchema, {
		method: 'POST',
		headers: authJsonHeaders(token),
		body: JSON.stringify({ category_index: categoryIndex, clue_index: clueIndex })
	});
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
