# Jeopardy Clone

A work-in-progress host-controlled Jeopardy game with a Rust/Axum backend, SvelteKit frontend, and Postgres via SQLx.

## Current State

Implemented now:

- Game creation with separate admin and player join codes.
- Player lobby joins with display-name persistence.
- Admin and player lobby pages.
- File-backed JSON question packs in `backend/question-packs/`.
- Backend-owned runtime gameplay sessions built from lobby players and a selected pack.
- Admin token issuance on admin join; gameplay writes require `Authorization: Bearer <token>`.
- Host-controlled clue selection and correct/incorrect scoring.
- Player and host game-state screens using polling.
- Postgres migrations for games, players, game status metadata, and admin tokens.
- A pure Rust Jeopardy rules engine covering clue selection, scoring, Daily Doubles, Final Jeopardy, and completion.

Still limited:

- Runtime engine state is in memory only. Restarting the backend loses active gameplay sessions.
- WebSockets are not implemented yet; frontend gameplay state uses polling.
- Players do not buzz in or submit answers. The host controls scoring.
- Pack authoring is JSON-only.

## Project Layout

```text
backend/   Rust, Axum, SQLx, Postgres, content packs, Jeopardy engine
frontend/  SvelteKit, TypeScript, Tailwind, Vitest
docs/      Backend gameplay design, implementation plan, and testing notes
```

## Running Locally

Start the full stack:

```bash
docker compose up
```

Default services:

- Frontend: `http://localhost:5173`
- Backend: `http://localhost:8080`
- Adminer: `http://localhost:8000`
- Postgres: `localhost:5432`

Backend env vars:

```text
DATABASE_URL=postgres://postgres:password@database:5432/docker
BIND_ADDRESS=0.0.0.0
BIND_PORT=8080
FRONTEND_ORIGIN=http://localhost:5173
QUESTION_PACK_DIR=./question-packs
```

For running the backend directly from `backend/`, use a localhost database URL:

```bash
DATABASE_URL=postgres://postgres:password@127.0.0.1:5432/docker cargo run
```

## Gameplay Flow

1. Open the frontend and create a host game.
2. Share the player code with players.
3. Players join with a display name.
4. Open the admin lobby with the admin code.
5. Choose a question pack and start the game.
6. The host selects clues and marks a selected player correct or incorrect.
7. Player pages poll for the same board, active clue, and scoreboard state.
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
GET  /games/packs
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

## Question Pack Format

Packs live in `backend/question-packs/*.json`.

```json
{
  "id": "classic",
  "title": "Classic Starter Pack",
  "rounds": [
    {
      "name": "Jeopardy",
      "categories": [
        {
          "title": "Rust",
          "clues": [
            {
              "label": "$200",
              "question": "This keyword creates a binding.",
              "answer": "What is let?",
              "value": 200,
              "daily_double": false
            }
          ]
        }
      ]
    }
  ],
  "final_jeopardy": {
    "category": "Computer History",
    "question": "This early programmer is often associated with the Analytical Engine.",
    "answer": "Who is Ada Lovelace?"
  }
}
```

`daily_double` and `final_jeopardy` are optional.

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

Frontend checks:

```bash
cd frontend
npm run check
npm run test
```

Randomized backend engine replay details are documented in `docs/testing/randomized-testing.md`.
