# Jeopardy Clone

A work-in-progress Jeopardy-style game with a Rust backend and SvelteKit frontend.

The current application supports creating a game, joining a lobby as a player, viewing a lobby as a host/admin, and listing joined players. The backend also contains the first pure Jeopardy rules engine and randomized regression coverage, but the full gameplay HTTP flow is still being built.

## Current State

Implemented now:

- Game creation with separate admin and player join codes.
- Player lobby joins with display-name persistence.
- Admin and player lobby read endpoints.
- Frontend screens for creating games, joining as a player, joining as an admin, and viewing lobby rosters.
- Postgres migrations for games, players, game status metadata, and admin tokens.
- A pure Rust Jeopardy rules engine covering clue selection, scoring, Daily Doubles, Final Jeopardy, and completion.
- Deterministic randomized backend tests and saved seed regressions for the rules engine.
- Design and implementation notes under `docs/`.

Not implemented yet:

- Question pack loading from checked-in JSON files.
- Starting a full game session from the lobby.
- Gameplay HTTP commands and state views.
- Admin token enforcement for gameplay writes.
- Realtime lobby or gameplay updates.

## Project Layout

```text
backend/   Rust, Axum, SQLx, Postgres, Jeopardy domain engine
frontend/  SvelteKit, TypeScript, Tailwind, Vitest
docs/      Backend gameplay design, implementation plan, and testing notes
```

## Running Locally

The Docker Compose setup starts Postgres, Adminer, the backend, and the frontend:

```bash
docker compose up
```

Default local services:

- Frontend: `http://localhost:5173`
- Backend: `http://localhost:8080`
- Adminer: `http://localhost:8000`
- Postgres: `localhost:5432`

The frontend expects `PUBLIC_API_URL=http://localhost:8080`. The backend defaults to:

```text
DATABASE_URL=postgres://postgres:password@database:5432/docker
BIND_ADDRESS=0.0.0.0
BIND_PORT=8080
FRONTEND_ORIGIN=http://localhost:5173
```

## Backend

The backend is organized around small module boundaries:

- `app` builds the Axum router and CORS layer.
- `config` reads runtime configuration from the environment.
- `games` contains game/lobby API handlers, service logic, repositories, DTOs, and code generation.
- `players` stores and reads lobby player data.
- `domain` contains the pure Jeopardy rules engine and randomized test harness.
- `sessions` and `moderation` are currently scaffolding for the next gameplay slice.

Current lobby endpoints:

```text
POST /games/new
POST /games/join/player
POST /games/join/admin
GET  /games/player/{player_code}
GET  /games/admin/{admin_code}
```

## Frontend

The frontend currently provides:

- A home page for creating games and joining lobbies.
- `/lobby/player/[player_code]` for player lobby views.
- `/lobby/admin/[admin_code]` for host/admin lobby views.
- Shared lobby roster rendering and lobby-code parsing helpers.

## Verification

Backend library tests:

```bash
cd backend
cargo test --lib
```

Frontend tests:

```bash
cd frontend
npm run test
```

Frontend type and lint checks:

```bash
cd frontend
npm run check
npm run lint
```

Randomized backend engine replay details are documented in `docs/testing/randomized-testing.md`.
