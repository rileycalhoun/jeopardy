import { afterEach, beforeEach, describe, expect, test, vi } from 'vitest';
import { z } from 'zod';

import { createSafeWebSocket, type WebSocketError, type WebSocketState } from './websocket';

const CONNECTING = 0;
const OPEN = 1;
const CLOSED = 3;

class FakeWebSocket {
	static instances: FakeWebSocket[] = [];

	url: string;
	readyState = CONNECTING;
	sent: string[] = [];
	onopen: (() => void) | null = null;
	onmessage: ((event: MessageEvent) => void) | null = null;
	onerror: (() => void) | null = null;
	onclose: (() => void) | null = null;

	constructor(url: string) {
		this.url = url;
		FakeWebSocket.instances.push(this);
	}

	send(data: string) {
		if (this.readyState !== OPEN) throw new Error('socket is not open');
		this.sent.push(data);
	}

	close() {
		if (this.readyState === CLOSED) return;
		this.readyState = CLOSED;
		this.onclose?.();
	}

	// Test helpers to drive the lifecycle.
	simulateOpen() {
		this.readyState = OPEN;
		this.onopen?.();
	}

	simulateMessage(data: string) {
		this.onmessage?.({ data } as MessageEvent);
	}

	simulateDrop() {
		this.readyState = CLOSED;
		this.onclose?.();
	}

	static latest(): FakeWebSocket {
		const socket = FakeWebSocket.instances.at(-1);
		if (!socket) throw new Error('no fake socket was created');
		return socket;
	}
}

const MessageSchema = z.object({ type: z.string(), value: z.number().optional() });

type Harness = {
	messages: z.infer<typeof MessageSchema>[];
	states: WebSocketState[];
	errors: WebSocketError[];
};

function connect(overrides: Record<string, unknown> = {}) {
	const harness: Harness = { messages: [], states: [], errors: [] };
	const socket = createSafeWebSocket({
		url: 'ws://test.invalid/ws',
		schema: MessageSchema,
		onMessage: (message) => harness.messages.push(message),
		onStateChange: (state) => harness.states.push(state),
		onError: (error) => harness.errors.push(error),
		createWebSocket: (url: string) => new FakeWebSocket(url) as unknown as WebSocket,
		baseReconnectDelayMs: 100,
		maxReconnectDelayMs: 1000,
		maxReconnectAttempts: 2,
		heartbeatIntervalMs: 0,
		...overrides
	});
	return { socket, harness };
}

beforeEach(() => {
	vi.useFakeTimers();
	FakeWebSocket.instances = [];
});

afterEach(() => {
	vi.useRealTimers();
});

describe('createSafeWebSocket', () => {
	test('delivers schema-valid messages to onMessage', () => {
		const { harness } = connect();
		FakeWebSocket.latest().simulateOpen();
		FakeWebSocket.latest().simulateMessage(JSON.stringify({ type: 'score', value: 200 }));

		expect(harness.states).toContain('open');
		expect(harness.messages).toEqual([{ type: 'score', value: 200 }]);
		expect(harness.errors).toEqual([]);
	});

	test('reports invalid JSON without throwing', () => {
		const { harness } = connect();
		FakeWebSocket.latest().simulateOpen();
		FakeWebSocket.latest().simulateMessage('this is not json');

		expect(harness.messages).toEqual([]);
		expect(harness.errors[0]?.kind).toBe('JsonParseError');
	});

	test('reports schema mismatches without throwing', () => {
		const { harness } = connect();
		FakeWebSocket.latest().simulateOpen();
		FakeWebSocket.latest().simulateMessage(JSON.stringify({ value: 'wrong-shape' }));

		expect(harness.messages).toEqual([]);
		expect(harness.errors[0]?.kind).toBe('ValidationError');
	});

	test('reconnects with backoff and fails after max attempts', () => {
		const { harness } = connect();
		FakeWebSocket.latest().simulateOpen();
		expect(FakeWebSocket.instances).toHaveLength(1);

		FakeWebSocket.latest().simulateDrop();
		expect(harness.states).toContain('reconnecting');

		// First retry after the base delay.
		vi.advanceTimersByTime(100);
		expect(FakeWebSocket.instances).toHaveLength(2);
		FakeWebSocket.latest().simulateDrop();

		// Second retry after doubled delay.
		vi.advanceTimersByTime(200);
		expect(FakeWebSocket.instances).toHaveLength(3);
		FakeWebSocket.latest().simulateDrop();

		// Retries exhausted: terminal failure, no further sockets.
		expect(harness.states).toContain('failed');
		vi.advanceTimersByTime(60_000);
		expect(FakeWebSocket.instances).toHaveLength(3);
	});

	test('a successful open resets the retry budget', () => {
		const { harness } = connect();
		FakeWebSocket.latest().simulateOpen();
		FakeWebSocket.latest().simulateDrop();
		vi.advanceTimersByTime(100);

		// Recovered; the attempt counter resets.
		FakeWebSocket.latest().simulateOpen();
		FakeWebSocket.latest().simulateDrop();
		vi.advanceTimersByTime(100);
		FakeWebSocket.latest().simulateOpen();

		expect(harness.states.filter((state) => state === 'open')).toHaveLength(3);
		expect(harness.states).not.toContain('failed');
	});

	test('manual close stops reconnecting', () => {
		const { socket, harness } = connect();
		FakeWebSocket.latest().simulateOpen();

		socket.close();

		expect(socket.getState()).toBe('closed');
		vi.advanceTimersByTime(60_000);
		expect(FakeWebSocket.instances).toHaveLength(1);
		expect(harness.states.at(-1)).toBe('closed');
	});

	test('send fails safely when the socket is not open', () => {
		const { socket, harness } = connect();

		expect(socket.send({ type: 'ping' })).toBe(false);
		expect(harness.errors[0]?.kind).toBe('SendError');

		FakeWebSocket.latest().simulateOpen();
		expect(socket.send({ type: 'ping' })).toBe(true);
		expect(FakeWebSocket.latest().sent).toEqual([JSON.stringify({ type: 'ping' })]);
	});

	test('heartbeats are sent on the configured interval', () => {
		connect({ heartbeatIntervalMs: 1000 });
		const fake = FakeWebSocket.latest();
		fake.simulateOpen();

		vi.advanceTimersByTime(1000);
		expect(fake.sent).toContain(JSON.stringify({ type: 'ping' }));
	});

	test('a silent server recycles the connection', () => {
		const { harness } = connect({ heartbeatIntervalMs: 1000 });
		const fake = FakeWebSocket.latest();
		fake.simulateOpen();

		// No messages for over two heartbeat intervals.
		vi.advanceTimersByTime(3000);

		expect(harness.errors.some((error) => error.kind === 'ConnectionError')).toBe(true);
		expect(harness.states).toContain('reconnecting');
	});
});
