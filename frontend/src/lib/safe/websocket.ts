import type { z } from 'zod';

// Mirrors the `$lib/safe/fetch` philosophy: no raw exceptions escape to the
// caller, every inbound message is validated with zod, and failures surface
// as typed values through callbacks.

export type WebSocketState =
	| 'connecting' // first connection attempt in flight
	| 'open' // connected and validated messages are flowing
	| 'reconnecting' // connection lost; retrying with bounded backoff
	| 'failed' // retries exhausted; the socket will not recover
	| 'closed'; // closed deliberately via close()

export type WebSocketError =
	| {
			kind: 'ConnectionError';
			message: string;
	  }
	| {
			kind: 'JsonParseError';
			message: string;
			cause: unknown;
	  }
	| {
			kind: 'ValidationError';
			message: string;
			issues: z.ZodIssue[];
	  }
	| {
			kind: 'SendError';
			message: string;
	  };

export type SafeWebSocketOptions<T> = {
	url: string;
	/** Every inbound message must match this schema or it is reported and dropped. */
	schema: z.ZodType<T>;
	onMessage: (message: T) => void;
	onStateChange?: (state: WebSocketState) => void;
	onError?: (error: WebSocketError) => void;
	/** Reconnect attempts before giving up. Resets after a successful open. */
	maxReconnectAttempts?: number;
	baseReconnectDelayMs?: number;
	maxReconnectDelayMs?: number;
	/** How often to send a JSON heartbeat; 0 disables. The connection is
	 * considered dead (and recycled) after two silent intervals. */
	heartbeatIntervalMs?: number;
	/** Injectable for tests. */
	createWebSocket?: (url: string) => WebSocket;
};

export type SafeWebSocket = {
	close: () => void;
	send: (payload: unknown) => boolean;
	getState: () => WebSocketState;
};

const DEFAULT_MAX_RECONNECT_ATTEMPTS = 5;
const DEFAULT_BASE_RECONNECT_DELAY_MS = 500;
const DEFAULT_MAX_RECONNECT_DELAY_MS = 10_000;
const DEFAULT_HEARTBEAT_INTERVAL_MS = 25_000;

export function createSafeWebSocket<T>(options: SafeWebSocketOptions<T>): SafeWebSocket {
	const maxReconnectAttempts = options.maxReconnectAttempts ?? DEFAULT_MAX_RECONNECT_ATTEMPTS;
	const baseReconnectDelayMs = options.baseReconnectDelayMs ?? DEFAULT_BASE_RECONNECT_DELAY_MS;
	const maxReconnectDelayMs = options.maxReconnectDelayMs ?? DEFAULT_MAX_RECONNECT_DELAY_MS;
	const heartbeatIntervalMs = options.heartbeatIntervalMs ?? DEFAULT_HEARTBEAT_INTERVAL_MS;
	const createWebSocket = options.createWebSocket ?? ((url: string) => new WebSocket(url));

	let socket: WebSocket | null = null;
	let state: WebSocketState = 'connecting';
	let reconnectAttempts = 0;
	let reconnectTimer: ReturnType<typeof setTimeout> | null = null;
	let heartbeatTimer: ReturnType<typeof setInterval> | null = null;
	let lastMessageAt = Date.now();
	let closedByCaller = false;

	function setState(next: WebSocketState) {
		if (state === next) return;
		state = next;
		options.onStateChange?.(next);
	}

	function reportError(error: WebSocketError) {
		options.onError?.(error);
	}

	function stopHeartbeat() {
		if (heartbeatTimer !== null) {
			clearInterval(heartbeatTimer);
			heartbeatTimer = null;
		}
	}

	function startHeartbeat() {
		if (heartbeatIntervalMs <= 0) return;
		lastMessageAt = Date.now();
		heartbeatTimer = setInterval(() => {
			if (Date.now() - lastMessageAt > heartbeatIntervalMs * 2) {
				// Server went silent; recycle the connection.
				reportError({
					kind: 'ConnectionError',
					message: 'No heartbeat response from the server.'
				});
				socket?.close();
				return;
			}
			trySend({ type: 'ping' });
		}, heartbeatIntervalMs);
	}

	function trySend(payload: unknown): boolean {
		if (socket === null || socket.readyState !== WebSocket.OPEN) {
			reportError({
				kind: 'SendError',
				message: 'The socket is not open.'
			});
			return false;
		}
		try {
			socket.send(JSON.stringify(payload));
			return true;
		} catch (cause) {
			reportError({
				kind: 'SendError',
				message: `Could not send over the socket: ${String(cause)}`
			});
			return false;
		}
	}

	function handleMessage(event: MessageEvent) {
		lastMessageAt = Date.now();

		if (typeof event.data !== 'string') {
			return;
		}

		let json: unknown;
		try {
			json = JSON.parse(event.data);
		} catch (cause) {
			reportError({
				kind: 'JsonParseError',
				message: 'The socket message was not valid JSON.',
				cause
			});
			return;
		}

		const parsed = options.schema.safeParse(json);
		if (!parsed.success) {
			reportError({
				kind: 'ValidationError',
				message: 'The socket message did not match the expected schema.',
				issues: parsed.error.issues
			});
			return;
		}

		options.onMessage(parsed.data);
	}

	function scheduleReconnect() {
		if (closedByCaller) {
			setState('closed');
			return;
		}
		if (reconnectAttempts >= maxReconnectAttempts) {
			setState('failed');
			return;
		}

		setState('reconnecting');
		const delay = Math.min(baseReconnectDelayMs * 2 ** reconnectAttempts, maxReconnectDelayMs);
		reconnectAttempts += 1;
		reconnectTimer = setTimeout(connect, delay);
	}

	function connect() {
		reconnectTimer = null;

		let next: WebSocket;
		try {
			next = createWebSocket(options.url);
		} catch (cause) {
			reportError({
				kind: 'ConnectionError',
				message: `Could not open the socket: ${String(cause)}`
			});
			scheduleReconnect();
			return;
		}

		socket = next;

		next.onopen = () => {
			reconnectAttempts = 0;
			setState('open');
			startHeartbeat();
		};

		next.onmessage = handleMessage;

		next.onerror = () => {
			reportError({
				kind: 'ConnectionError',
				message: 'The socket reported an error.'
			});
		};

		next.onclose = () => {
			stopHeartbeat();
			socket = null;
			scheduleReconnect();
		};
	}

	connect();

	return {
		close: () => {
			closedByCaller = true;
			if (reconnectTimer !== null) {
				clearTimeout(reconnectTimer);
				reconnectTimer = null;
			}
			stopHeartbeat();
			socket?.close();
			socket = null;
			setState('closed');
		},
		send: trySend,
		getState: () => state
	};
}
