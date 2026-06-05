# Backend Gameplay Design

## Scope

This design covers the first end-to-end backend gameplay slice for the Jeopardy project.

The slice starts after the existing lobby flow:

- host creates a game
- players join with the player code
- admins join with the admin code

The slice ends when the backend can:

- load editable question packs from JSON
- issue admin tokens for authenticated write actions
- start a game from a chosen pack
- expose complete backend game state for admins and players
- drive the full Jeopardy lifecycle through Final Jeopardy and completion

Frontend gameplay screens, realtime broadcasting, and richer authoring tools are intentionally out of scope for this first spec.

## Goals

- Keep the backend as the source of truth for Jeopardy rules.
- Reuse the existing pure engine in `backend/src/domain/jeopardy.rs`.
- Add as little architectural churn as possible.
- Support multiple checked-in JSON packs now without closing off a later database-backed content model.
- Persist enough metadata to preserve session ownership and pack selection while keeping detailed runtime state in memory for now.

## Non-Goals

- WebSocket gameplay transport
- buzzer-style player answer input
- long-term persistent replay/event sourcing
- pack editing UI
- restart recovery of in-progress turn-by-turn engine state

## Chosen Approach

Use a session wrapper around the existing engine.

The rules engine remains pure and transport-agnostic. A new application layer will:

- load and validate packs
- create runtime sessions from packs and lobby rosters
- persist game metadata and admin tokens
- expose HTTP command/query APIs

This approach is preferred over a full event log or fully persisted mutable engine state because it preserves the backend testing work already in place and keeps the first complete gameplay slice focused.

## Architecture

### Domain

`backend/src/domain/jeopardy.rs` remains the source of truth for:

- phases
- clue selection
- score changes
- Daily Double wager and resolution
- Final Jeopardy wager and resolution
- invalid action handling
- completion

The engine should stay free of:

- Axum types
- SQLx queries
- file loading
- auth concerns

### Content

Add a `content` boundary responsible for loading question packs from JSON files in:

`backend/question-packs/*.json`

Responsibilities:

- enumerate available packs
- parse JSON
- validate pack structure and IDs
- convert pack data into engine-ready scenario input

This boundary should be written so the storage mechanism can later change from file-backed JSON to database-backed content without forcing a rules rewrite.

### Games

The `games` module remains the application entry point for gameplay commands and reads.

New responsibilities:

- admin token issuance
- starting a game with a chosen pack
- loading game state for admins and players
- applying admin-driven gameplay commands
- persisting status transitions

### Sessions

The `sessions` module becomes the runtime owner of active games.

Responsibilities:

- hold in-memory `JeopardyGame` instances
- key active runtime by the persisted game identity
- provide safe lookup for read/write operations
- later serve as the natural home for realtime fanout

For this slice, sessions are in-memory only. On restart, lobby metadata remains in Postgres but active in-progress gameplay runtime is not yet reconstructed.

### Persistence

Postgres stores metadata, not full live board state.

Persisted concerns:

- game status
- chosen pack ID
- start/completion timestamps
- admin token hashes
- existing lobby/player data

Live detailed engine state remains in memory during active play.

## Data Model

### Persisted Tables

#### `games`

Extend the existing table with:

- `status` enum or text field with values: `lobby`, `in_progress`, `completed`
- `question_pack_id` nullable until start
- `started_at`
- `completed_at`

The existing `game_id`, `admin_code`, and `player_code` remain as the lobby/game identity fields already in use.

#### `admin_tokens`

New table:

- `id`
- `game_id`
- `token_hash`
- `label`
- `created_at`
- `revoked_at`

The raw token is only returned once at issuance time. Stored values are hashed before persistence.

### File-Backed Pack Model

Packs live in `backend/question-packs/`.

Each JSON pack should include:

- `id`
- `title`
- regular round boards
- categories
- clues
- clue values
- Daily Double flags
- Final Jeopardy category and clue

The pack format should stay editor-friendly and human-readable because JSON is the authoring mechanism for now.

### Runtime Session Shape

Each active session needs:

- persisted game ID
- pack ID
- `JeopardyGame`
- current player roster snapshot
- timestamp metadata

This stays internal to the backend and is not exposed directly over HTTP.

### Read Models

#### `LobbyView`

- game status
- joined players
- available packs or chosen pack

#### `GameView`

- phase
- current round
- scoreboard
- board state
- active clue state
- Final Jeopardy state if applicable
- caller role metadata where useful

The API read models intentionally wrap the engine state so transport shape can evolve without exposing engine internals directly.

## Authentication Model

Admin join by admin code should issue a backend token.

Write actions require that token:

- start game
- select clue
- resolve clue outcome
- submit Daily Double wager
- resolve Daily Double
- submit Final Jeopardy wager
- resolve Final Jeopardy

The initial slice treats the host/admin as the game operator. Players do not submit authoritative gameplay actions yet.

This keeps the backend complete and deterministic without introducing buzzer concurrency in the same slice.

## HTTP API

### Reads

- `GET /packs`
  - list available JSON packs
- `GET /games/player/{player_code}`
  - existing lobby view before start
- `GET /games/admin/{admin_code}`
  - existing lobby view before start
- `GET /games/player/{player_code}/state`
  - return `GameView`
- `GET /games/admin/{admin_code}/state`
  - return `GameView`

### Auth / Join

- `POST /games/join/player`
  - existing player join flow
- `POST /games/join/admin`
  - existing admin join flow
  - extended to return an issued admin token

### Writes

- `POST /games/start`
  - admin token required
  - body includes chosen `pack_id`
- `POST /games/actions/select-clue`
- `POST /games/actions/attempt-answer`
- `POST /games/actions/daily-double-wager`
- `POST /games/actions/resolve-daily-double`
- `POST /games/actions/final-wager`
- `POST /games/actions/resolve-final`

All write commands are admin-token protected in this slice.

## Lifecycle

1. Host creates a game.
2. Players join with the player code.
3. Admin joins with the admin code and receives an admin token.
4. Admin selects a pack and starts the game.
5. Backend loads the chosen pack, builds a scenario from the roster, and creates a runtime `JeopardyGame`.
6. Admin drives the game through command endpoints.
7. Player and admin state endpoints reflect the evolving engine state.
8. When the engine completes, the backend marks the game as completed in Postgres.

## Error Handling

Application errors should stay centralized in `AppError`.

Additions likely needed:

- invalid pack ID
- game already started
- game not started
- admin token missing/invalid/revoked
- runtime session not loaded
- invalid gameplay command for current phase

Engine-level validation errors should be translated into stable API errors without duplicating rules in handlers.

## Testing

### Keep Existing Engine Tests

The existing deterministic randomized backend tests remain the primary rules regression layer.

### Add Session/Application Tests

Add backend tests for:

- pack loading and validation
- admin token issuance and validation
- start game lifecycle
- state query behavior before and after start
- full game command flow through completion
- persistence status updates

### Pack Fixture Tests

Include tests that validate checked-in JSON packs can be loaded into valid scenarios.

## Open Assumptions

This spec intentionally carries the current engine assumptions forward for the first slice:

- zero-score players may enter Final Jeopardy and wager zero
- negative-score players are excluded from Final Jeopardy
- the admin/operator resolves clue outcomes

If gameplay semantics change later, update the engine, randomized tests, and API expectations together.

## Implementation Order

1. Add content loader and JSON pack fixtures.
2. Add persistence changes for game status and admin tokens.
3. Add session runtime manager for active `JeopardyGame` instances.
4. Extend admin join to issue tokens.
5. Add start-game flow.
6. Add gameplay state read endpoints.
7. Add gameplay command endpoints through completion.
8. Add integration tests around the new backend slice.

