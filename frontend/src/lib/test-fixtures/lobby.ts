type FixturePlayer = {
	id: number;
	display_name: string;
};

const NAME_PARTS = [
	'Ada',
	'Grace',
	'Ken',
	'Buzzy',
	'Renée',
	'Zoë',
	'Player-One',
	'Δelta',
	'Quiz Wizard',
	'Very Long Contestant Name'
];

function createSeededRng(seed: number) {
	let state = seed >>> 0;

	return () => {
		state = (1664525 * state + 1013904223) >>> 0;
		return state / 0x1_0000_0000;
	};
}

function pick<T>(values: T[], next: () => number): T {
	const index = Math.floor(next() * values.length);
	return values[index] ?? values[0];
}

export function generateLobbyFixture(seed: number, playerCount?: number): FixturePlayer[] {
	const next = createSeededRng(seed);
	const count = playerCount ?? Math.floor(next() * 9);

	return Array.from({ length: count }, (_, index) => {
		let name = `${pick(NAME_PARTS, next)} ${index + 1}`;

		if (next() > 0.7) {
			name += ' !!!';
		}

		if (next() > 0.82) {
			name += ` ${'X'.repeat(18)}`;
		}

		return {
			id: index + 1,
			display_name: name
		};
	});
}
