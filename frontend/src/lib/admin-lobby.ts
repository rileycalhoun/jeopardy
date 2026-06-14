export function shouldRefreshAdminLobby(game: object | null): boolean {
	return game === null;
}

export function canAdminSelectClue(game: {
	phase: string;
	current_selector: number | null;
}): boolean {
	return game.phase === 'RoundSelection' && game.current_selector === null;
}
