# Jeopardy

A work-in-progress host-controlled Jeopardy game with a Rust/Axum backend, SvelteKit frontend, Postgres via SQLx, and Redis for runtime game state.

## Current State

Implemented now:

- Game creation with separate admin and player join codes.
- Player lobby joins with display-name persistence.
- Admin and player lobby pages.
- File-backed JSON category packs in `backend/categories/`.
- Backend-owned runtime gameplay sessions built from lobby players and selected categories.
- Runtime sessions stored in Redis, so active gameplay survives backend restarts and works across multiple backend instances.
- WebSocket live updates for lobby and game state, with Redis pub/sub fan-out between instances and a slow-polling frontend fallback.
- Admin token issuance on admin join; gameplay writes require `Authorization: Bearer <token>`.
- Host-controlled clue selection and correct/incorrect scoring.
- Postgres migrations for games, players, game status metadata, and admin tokens.
- A pure Rust Jeopardy rules engine covering clue selection, scoring, Daily Doubles, Final Jeopardy, and completion.

Still limited:

- Players do not buzz in. The host controls scoring.
- Category authoring is JSON-only.

## Storage Model

PostgreSQL is the durable source of truth for long-lived records: the `games`
table, player records, admin token hashes, game status metadata, the selected
content id, and `started_at`/`completed_at` timestamps.

Redis holds temporary runtime state for active games: the serialized
`RuntimeSession` (engine state, active clue, answer submissions, current
selector) under `jeopardy:game:{game_id}:session` with a refresh-on-write TTL,
plus the `jeopardy:events` pub/sub channel used to broadcast game updates to
websocket clients on every backend instance. Finishing a game removes the
Redis session and marks the Postgres game record completed.

## Project Layout

```text
backend/   Rust, Axum, SQLx, Postgres, category content, Jeopardy engine
frontend/  SvelteKit, TypeScript, Tailwind, Vitest
docs/      Backend gameplay design, implementation plan, and testing notes
```

## Running With Docker

This project has separate Docker setups for development and production.

Development Docker uses `docker-compose.dev.yml`, bind-mounted source directories,
`cargo watch`, and the Vite dev server:

```bash
docker compose -f docker-compose.dev.yml up --build
```

Development services:

- Frontend: `http://localhost:5173`
- Backend: `http://localhost:8080`
- Adminer: `http://localhost:8000`
- Postgres: `localhost:5432`
- Redis: `localhost:6379`

Production Docker uses `docker-compose.yml`, builds optimized backend/frontend
artifacts, runs database migrations, and starts the compiled Rust binary plus
the SvelteKit Node server:

```bash
docker compose up --build
```

Production services:

- Frontend: `http://localhost:3000`
- Backend: `http://localhost:8080`
- Adminer: `http://localhost:8000`
- Postgres: `localhost:5432`
- Redis: `localhost:6379`

The `postgres` Docker volume is used for database persistence.
The production `migrate` service runs `sqlx migrate run` after Postgres becomes
healthy and before the backend starts. The backend also waits for the Redis
healthcheck, since runtime sessions and update events live in Redis.

## Environment Variables

Backend variables:

```text
DATABASE_URL=postgres://postgres:password@database:5432/docker
REDIS_URL=redis://redis:6379
BIND_ADDRESS=0.0.0.0
BIND_PORT=8080
FRONTEND_ORIGIN=http://localhost:3000
CATEGORY_DIR=/app/categories
RUST_LOG=info
LOG_FORMAT=json
```

`RUST_LOG` accepts standard `tracing_subscriber::EnvFilter` directives, including
per-module levels such as `jeopardy_clone=debug,tower_http=info`. `LOG_FORMAT`
accepts `json` for structured production logs or `pretty` for readable local
logs. Missing values default to `info` and `pretty`; invalid values fall back
gracefully. Logs are written to stdout/stderr for collection by Docker, Loki,
or another container log driver.

Frontend variables:

```text
PUBLIC_API_URL=http://localhost:8080
HOST=0.0.0.0
PORT=3000
ORIGIN=http://localhost:3000
```

Database and published port variables:

```text
POSTGRES_USER=postgres
POSTGRES_PASSWORD=<required for production>
POSTGRES_DB=docker
POSTGRES_PORT=5432
REDIS_PORT=6379
BACKEND_PORT=8080
FRONTEND_PORT=3000
ADMINER_PORT=8000
```

For production Docker, `POSTGRES_PASSWORD` must be set in the environment or a
local `.env` file before running `docker compose up --build`. For deployed
production, set `PUBLIC_API_URL` to the browser-visible backend URL and set
`FRONTEND_ORIGIN` to the public frontend origin used for CORS and SvelteKit
origin checks. A reverse proxy should route public traffic to the frontend on
port `3000` and the backend API on port `8080`.

For running the backend directly from `backend/`, use localhost database and
Redis URLs:

```bash
DATABASE_URL=postgres://postgres:password@127.0.0.1:5432/docker \
REDIS_URL=redis://127.0.0.1:6379 \
RUST_LOG=debug \
LOG_FORMAT=pretty cargo run
```

The production frontend container uses `@sveltejs/adapter-node` and starts with
`node build`. The development frontend container still uses `npm run dev`.

## Gameplay Flow

1. Open the frontend and create a host game.
2. Share the player code with players.
3. Players join with a display name.
4. Open the admin lobby with the admin code.
5. Choose categories and start the game.
6. The host selects clues and marks a selected player correct or incorrect.
7. Player and host pages receive the board, active clue, and scoreboard state
   live over WebSockets (with slow polling as a fallback).
8. The host can finish the game.

## Backend API

Lobby:

```text
POST /games/new
POST /games/join/player
POST /games/join/admin
GET  /games/player/{player_code}
GET  /games/admin/{admin_code}
```

Gameplay:

```text
GET  /games/categories
POST /games/admin/{admin_code}/start
GET  /games/admin/{admin_code}/state
GET  /games/player/{player_code}/state
POST /games/player/{player_code}/answer
POST /games/admin/{admin_code}/select-clue
POST /games/admin/{admin_code}/answer
POST /games/admin/{admin_code}/daily-double/wager
POST /games/admin/{admin_code}/daily-double/resolve
POST /games/admin/{admin_code}/final/wager
POST /games/admin/{admin_code}/final/resolve
POST /games/admin/{admin_code}/finish
```

Gameplay write routes require:

```text
Authorization: Bearer <admin_token>
```

The raw admin token is returned once from `POST /games/join/admin`.

WebSockets:

```text
GET /ws/games/admin/{admin_code}?token=<admin_token>
GET /ws/games/player/{player_code}
```

Each socket sends the current lobby (and game state, if a session is active)
on connect, then pushes tagged JSON messages (`lobby`, `game_state`,
`game_finished`, `pong`, `error`) after every gameplay mutation. The admin
socket validates the admin token and admins receive answers and player
submissions; player sockets receive a redacted view. Unauthenticated sockets
are closed with an `error` message and close code `4401`. Gameplay commands
remain REST-only; sockets carry `{"type":"ping"}` heartbeats from clients.

## Category Pack Format

Category packs live in `backend/categories/*.json`. Each selected category
contributes one randomly selected clue for each point value: 200, 400, 600,
800, and 1000. The selected clues are fixed when the game starts.

```json
{
  "id": "video_games",
  "title": "Video Games",
  "description": "Questions about games, consoles, characters, and gaming history.",
  "questions": [
    {
      "points": 200,
      "question": "What Nintendo franchise features Link as its main playable hero?",
      "answer": "The Legend of Zelda"
    }
  ]
}
```

Required category fields are `id`, `title`, and `questions`. `description` is
optional. Each question requires `points`, `question`, and `answer`, and valid
point values are exactly `100`, `200`, `300`, `400`, and `500`.

## Verification

Backend checks:

```bash
cd backend
cargo fmt
cargo check
cargo test --lib
cargo test
```

`cargo test` includes SQLx tests that require a reachable Postgres database and Docker/Postgres running locally.

The Redis session store integration test is `#[ignore]`d by default. Run it
against a live Redis with:

```bash
REDIS_URL=redis://127.0.0.1:6379 cargo test redis_store -- --ignored
```

Frontend checks:

```bash
cd frontend
npm run check
npm run test
```

Randomized backend engine replay details are documented in `docs/testing/randomized-testing.md`.
