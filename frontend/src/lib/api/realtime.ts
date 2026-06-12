import { z } from 'zod';

import { env } from '$env/dynamic/public';

import {
	GameViewSchema,
	PlayerSummarySchema,
	type GameView,
	type PlayerSummary
} from '$lib/api/schemas';
import {
	createSafeWebSocket,
	type SafeWebSocket,
	type WebSocketError,
	type WebSocketState
} from '$lib/safe/websocket';

// Messages pushed by the backend realtime module (`realtime/messages.rs`).
export const ServerMessageSchema = z.discriminatedUnion('type', [
	z.object({ type: z.literal('lobby'), players: z.array(PlayerSummarySchema) }),
	z.object({ type: z.literal('game_state'), game: GameViewSchema }),
	z.object({ type: z.literal('game_finished') }),
	z.object({ type: z.literal('pong') }),
	z.object({ type: z.literal('error'), message: z.string() })
]);

export type ServerMessage = z.infer<typeof ServerMessageSchema>;

export type GameSocketHandlers = {
	onLobby?: (players: PlayerSummary[]) => void;
	onGameState?: (game: GameView) => void;
	onGameFinished?: () => void;
	/** Application-level error sent by the server (for example bad auth). */
	onServerError?: (message: string) => void;
	onStateChange?: (state: WebSocketState) => void;
	onError?: (error: WebSocketError) => void;
};

export function connectAdminGameSocket(
	adminCode: number,
	token: string,
	handlers: GameSocketHandlers
): SafeWebSocket {
	const url = `${webSocketBaseUrl()}/ws/games/admin/${adminCode}?token=${encodeURIComponent(token)}`;
	return connectGameSocket(url, handlers);
}

export function connectPlayerGameSocket(
	playerCode: number,
	handlers: GameSocketHandlers
): SafeWebSocket {
	const url = `${webSocketBaseUrl()}/ws/games/player/${playerCode}`;
	return connectGameSocket(url, handlers);
}

function connectGameSocket(url: string, handlers: GameSocketHandlers): SafeWebSocket {
	return createSafeWebSocket({
		url,
		schema: ServerMessageSchema,
		onStateChange: handlers.onStateChange,
		onError: handlers.onError,
		onMessage: (message) => {
			switch (message.type) {
				case 'lobby':
					handlers.onLobby?.(message.players);
					break;
				case 'game_state':
					handlers.onGameState?.(message.game);
					break;
				case 'game_finished':
					handlers.onGameFinished?.();
					break;
				case 'error':
					handlers.onServerError?.(message.message);
					break;
				case 'pong':
					// Heartbeat reply; nothing to surface.
					break;
			}
		}
	});
}

function webSocketBaseUrl(): string {
	const api = env.PUBLIC_API_URL ?? '';
	return api.replace(/^http/, 'ws');
}
