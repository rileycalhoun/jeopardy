import { z } from 'zod';

// Shared zod schemas for backend payloads. Used by both the REST client
// (`$lib/api/games`) and the websocket client (`$lib/api/realtime`).

export const PlayerSummarySchema = z.object({
	id: z.number().int(),
	display_name: z.string()
});

export const LobbySchema = z.object({
	players: z.array(PlayerSummarySchema),
	admin_token: z.string().optional(),
	current_player_id: z.number().int().optional()
});

export const QuestionPackSchema = z.object({
	id: z.string(),
	title: z.string()
});

export const PacksSchema = z.object({
	packs: z.array(QuestionPackSchema)
});

export const PlayerScoreSchema = z.object({
	id: z.number().int(),
	name: z.string(),
	score: z.number().int()
});

export const ClueTileSchema = z.object({
	label: z.string(),
	value: z.number().int(),
	answered: z.boolean(),
	daily_double: z.boolean()
});

export const CategorySchema = z.object({
	title: z.string(),
	clues: z.array(ClueTileSchema)
});

export const RoundSchema = z.object({
	name: z.string(),
	categories: z.array(CategorySchema)
});

export const AnswerSubmissionSchema = z.object({
	player_id: z.number().int(),
	player_name: z.string(),
	answer: z.string()
});

export const ActiveClueSchema = z.object({
	round_index: z.number().int(),
	category_index: z.number().int(),
	clue_index: z.number().int(),
	label: z.string(),
	value: z.number().int(),
	question: z.string(),
	answer: z.string(),
	attempted_player_ids: z.array(z.number().int()),
	submissions: z.array(AnswerSubmissionSchema)
});

export const FinalJeopardySchema = z.object({
	category: z.string(),
	question: z.string(),
	answer: z.string()
});

export const GameViewSchema = z.object({
	phase: z.string(),
	current_round: z.number().int(),
	current_selector: z.number().int(),
	players: z.array(PlayerScoreSchema),
	board: z.array(RoundSchema),
	active_clue: ActiveClueSchema.nullable(),
	final_jeopardy: FinalJeopardySchema.nullable()
});

export const GameStateSchema = z.object({
	game: GameViewSchema
});

export const CreateGameSchema = z.object({
	player_code: z.int32(),
	admin_code: z.int32()
});

export const FinishGameSchema = z.object({
	completed: z.boolean()
});

export type PlayerSummary = z.infer<typeof PlayerSummarySchema>;
export type Lobby = z.infer<typeof LobbySchema>;
export type QuestionPack = z.infer<typeof QuestionPackSchema>;
export type GameView = z.infer<typeof GameViewSchema>;
export type CreateGameResponse = z.infer<typeof CreateGameSchema>;
