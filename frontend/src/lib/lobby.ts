export function parseJoinCode(input: string): number | null {
	const trimmed = input.trim();

	if (!/^\d+$/.test(trimmed)) {
		return null;
	}

	const parsed = Number(trimmed);

	return Number.isSafeInteger(parsed) ? parsed : null;
}

export function playerLobbyPath(playerCode: number): string {
	return `/lobby/player/${playerCode}`;
}

export function adminLobbyPath(adminCode: number): string {
	return `/lobby/admin/${adminCode}`;
}
