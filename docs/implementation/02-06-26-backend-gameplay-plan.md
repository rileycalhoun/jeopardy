# Backend Gameplay Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build the first complete backend gameplay slice: load editable JSON question packs, issue admin tokens, start a game, expose backend game state, and drive a full Jeopardy session through completion.

**Architecture:** Keep `backend/src/domain/jeopardy.rs` as the pure rules engine. Add a file-backed content loader, a session runtime manager, and thin application/API layers that persist only session metadata and admin auth state in Postgres while holding live `JeopardyGame` instances in memory.

**Tech Stack:** Rust, Axum, SQLx, Postgres, Serde, Tokio, existing randomized backend engine tests

---

## File Structure

### Existing Files To Modify

- `backend/src/config.rs`
  - add a configurable question-pack directory
- `backend/src/state.rs`
  - extend shared app state with content/session/auth services
- `backend/src/error.rs`
  - add gameplay/content/auth/session error variants
- `backend/src/app.rs`
  - register gameplay read/write routes
- `backend/src/games/models.rs`
  - add gameplay request/response DTOs
- `backend/src/games/repository.rs`
  - persist game status, selected pack, timestamps, and admin tokens
- `backend/src/games/service.rs`
  - add start-game, token issuance, state read, and command application flows
- `backend/src/games/api.rs`
  - add pack listing, start, state, and gameplay action handlers
- `backend/src/sessions/mod.rs`
  - export real runtime manager types
- `backend/src/sessions/manager.rs`
  - replace placeholder with active runtime session manager
- `backend/src/lib.rs`
  - export new backend modules for tests
- `backend/src/domain/jeopardy.rs`
  - add minimal helpers required to serialize/inspect state cleanly for HTTP read models if tests prove they are missing

### New Backend Files

- `backend/migrations/<timestamp>_gameplay_session_metadata.sql`
  - extend `games`, add `admin_tokens`
- `backend/src/content/mod.rs`
  - module exports
- `backend/src/content/models.rs`
  - JSON pack structs and validation-facing models
- `backend/src/content/loader.rs`
  - enumerate and load packs from disk
- `backend/src/content/service.rs`
  - convert loaded packs + lobby roster into engine scenarios
- `backend/src/sessions/runtime.rs`
  - runtime session struct and conversion helpers
- `backend/src/games/auth.rs`
  - admin token generation, hashing, lookup helpers
- `backend/src/games/state.rs`
  - API-facing game/lobby read-model assembly
- `backend/src/games/tests.rs`
  - focused service/repository tests for gameplay lifecycle

### New Fixtures / Content

- `backend/question-packs/classic.json`
  - baseline playable pack
- `backend/question-packs/science-mix.json`
  - second checked-in pack

### New Docs

- `docs/testing/backend-gameplay.md`
  - commands and lifecycle notes for the new gameplay backend slice

---

### Task 1: Add Database Metadata For Started Games

**Files:**
- Create: `backend/migrations/<timestamp>_gameplay_session_metadata.sql`
- Modify: `backend/src/games/repository.rs`
- Test: `backend/src/games/tests.rs`

- [ ] **Step 1: Write the failing repository test for game status persistence**

Add a test in `backend/src/games/tests.rs` that expects a created game to transition from lobby to in-progress with a stored `question_pack_id` and `started_at`.

- [ ] **Step 2: Run the targeted backend test to verify it fails**

Run: `cd backend && cargo test games::tests::persists_started_game_metadata -- --exact`
Expected: FAIL because no persistence API or schema exists yet.

- [ ] **Step 3: Add the SQL migration**

Create a migration that:
- adds `status TEXT NOT NULL DEFAULT 'lobby'` to `games`
- adds nullable `question_pack_id TEXT`
- adds nullable `started_at TIMESTAMPTZ`
- adds nullable `completed_at TIMESTAMPTZ`
- creates `admin_tokens` with `game_id`, `token_hash`, `label`, `created_at`, `revoked_at`

- [ ] **Step 4: Add minimal repository methods**

Implement methods in `backend/src/games/repository.rs` for:
- updating game start metadata
- marking game completed
- inserting admin tokens
- looking up active admin tokens by hash

- [ ] **Step 5: Run the targeted test to verify it passes**

Run: `cd backend && cargo test games::tests::persists_started_game_metadata -- --exact`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add backend/migrations backend/src/games/repository.rs backend/src/games/tests.rs
git commit -m "feat: persist gameplay session metadata"
```

---

### Task 2: Add File-Backed Question Pack Loading

**Files:**
- Create: `backend/src/content/mod.rs`
- Create: `backend/src/content/models.rs`
- Create: `backend/src/content/loader.rs`
- Modify: `backend/src/config.rs`
- Modify: `backend/src/lib.rs`
- Create: `backend/question-packs/classic.json`
- Create: `backend/question-packs/science-mix.json`
- Test: `backend/src/games/tests.rs`

- [ ] **Step 1: Write the failing pack loader tests**

Add tests for:
- listing packs from the configured directory
- rejecting duplicate pack IDs
- loading a valid pack into a validated pack model

- [ ] **Step 2: Run the targeted tests to verify they fail**

Run: `cd backend && cargo test games::tests::loads_question_packs_from_disk -- --exact`
Expected: FAIL because the loader and fixtures do not exist.

- [ ] **Step 3: Add the question-pack config**

Extend `backend/src/config.rs` with:
- `question_pack_dir: String`
- env override such as `QUESTION_PACK_DIR`
- default `./question-packs`

- [ ] **Step 4: Add pack models and loader**

Implement:
- serde-friendly JSON structs
- validation for IDs, rounds, categories, clues, Daily Doubles, and Final Jeopardy presence
- directory enumeration and pack listing

- [ ] **Step 5: Add two valid JSON packs**

Create small but complete packs that are easy to understand and use in tests.

- [ ] **Step 6: Run the targeted tests to verify they pass**

Run: `cd backend && cargo test games::tests::loads_question_packs_from_disk -- --exact`
Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add backend/src/content backend/src/config.rs backend/src/lib.rs backend/question-packs backend/src/games/tests.rs
git commit -m "feat: add file-backed question pack loading"
```

---

### Task 3: Convert Packs And Lobby Rosters Into Engine Scenarios

**Files:**
- Create: `backend/src/content/service.rs`
- Modify: `backend/src/content/mod.rs`
- Modify: `backend/src/domain/jeopardy.rs`
- Test: `backend/src/games/tests.rs`

- [ ] **Step 1: Write the failing scenario builder tests**

Add tests that assert:
- a loaded pack plus joined players produces a valid `GameScenario`
- player names and IDs are preserved
- clue/value/Daily Double data is mapped correctly

- [ ] **Step 2: Run the targeted tests to verify they fail**

Run: `cd backend && cargo test games::tests::builds_engine_scenarios_from_packs -- --exact`
Expected: FAIL because the scenario builder does not exist.

- [ ] **Step 3: Implement the scenario builder**

Add a content service that:
- accepts a validated pack
- accepts current joined players
- assigns stable player IDs
- maps rounds/categories/clues into `GameScenario`
- chooses a deterministic starting selector, documented in code comments

- [ ] **Step 4: Add only the minimal domain helper methods the tests require**

If needed, add small read-only helpers in `backend/src/domain/jeopardy.rs`, but do not move transport or persistence logic into the engine.

- [ ] **Step 5: Run the targeted tests to verify they pass**

Run: `cd backend && cargo test games::tests::builds_engine_scenarios_from_packs -- --exact`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add backend/src/content backend/src/domain/jeopardy.rs backend/src/games/tests.rs
git commit -m "feat: build engine scenarios from question packs"
```

---

### Task 4: Add Admin Token Issuance And Validation

**Files:**
- Create: `backend/src/games/auth.rs`
- Modify: `backend/src/games/mod.rs`
- Modify: `backend/src/games/models.rs`
- Modify: `backend/src/games/service.rs`
- Modify: `backend/src/games/repository.rs`
- Modify: `backend/src/error.rs`
- Test: `backend/src/games/tests.rs`

- [ ] **Step 1: Write the failing admin auth tests**

Add tests that expect:
- admin join returns a token
- only hashed tokens are stored
- invalid/revoked tokens are rejected

- [ ] **Step 2: Run the targeted tests to verify they fail**

Run: `cd backend && cargo test games::tests::issues_and_validates_admin_tokens -- --exact`
Expected: FAIL because token issuance does not exist.

- [ ] **Step 3: Add token helpers**

Implement:
- random token generation
- stable hash storage
- validation against persisted active token rows

- [ ] **Step 4: Extend admin join flow**

Change the admin join response model to include the issued token while preserving current lobby information.

- [ ] **Step 5: Add centralized auth errors**

Extend `AppError` with:
- missing/invalid admin token
- revoked token

- [ ] **Step 6: Run the targeted tests to verify they pass**

Run: `cd backend && cargo test games::tests::issues_and_validates_admin_tokens -- --exact`
Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add backend/src/games/auth.rs backend/src/games/mod.rs backend/src/games/models.rs backend/src/games/service.rs backend/src/games/repository.rs backend/src/error.rs backend/src/games/tests.rs
git commit -m "feat: issue admin tokens for gameplay writes"
```

---

### Task 5: Add The In-Memory Runtime Session Manager

**Files:**
- Modify: `backend/src/sessions/mod.rs`
- Modify: `backend/src/sessions/manager.rs`
- Create: `backend/src/sessions/runtime.rs`
- Modify: `backend/src/state.rs`
- Test: `backend/src/games/tests.rs`

- [ ] **Step 1: Write the failing runtime session tests**

Add tests that expect:
- a session can be created for a started game
- sessions are retrievable by persisted game ID
- applying commands mutates only that session

- [ ] **Step 2: Run the targeted tests to verify they fail**

Run: `cd backend && cargo test games::tests::stores_runtime_sessions_in_memory -- --exact`
Expected: FAIL because the manager is still a placeholder.

- [ ] **Step 3: Implement the runtime session manager**

Use an async-safe in-memory map keyed by persisted game ID and store:
- pack ID
- `JeopardyGame`
- roster snapshot
- timestamps

- [ ] **Step 4: Wire the manager into `AppState`**

Extend shared state so services can read/write active runtime sessions.

- [ ] **Step 5: Run the targeted tests to verify they pass**

Run: `cd backend && cargo test games::tests::stores_runtime_sessions_in_memory -- --exact`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add backend/src/sessions backend/src/state.rs backend/src/games/tests.rs
git commit -m "feat: add in-memory gameplay session manager"
```

---

### Task 6: Add Start-Game Service Logic

**Files:**
- Modify: `backend/src/games/models.rs`
- Modify: `backend/src/games/service.rs`
- Modify: `backend/src/games/repository.rs`
- Modify: `backend/src/error.rs`
- Test: `backend/src/games/tests.rs`

- [ ] **Step 1: Write the failing start-game service tests**

Add tests that expect:
- a valid admin token and pack ID can start a lobby game
- starting twice is rejected
- missing pack ID is rejected
- no runtime session is created when validation fails

- [ ] **Step 2: Run the targeted tests to verify they fail**

Run: `cd backend && cargo test games::tests::starts_gameplay_sessions -- --exact`
Expected: FAIL because the start flow does not exist.

- [ ] **Step 3: Add start-game request/response models**

Add DTOs for:
- chosen `pack_id`
- authenticated admin start request
- post-start summary or initial `GameView`

- [ ] **Step 4: Implement start-game orchestration**

Service flow:
- validate admin token
- load joined players
- load chosen pack
- build `GameScenario`
- construct `JeopardyGame`
- persist game status and pack metadata
- register runtime session

- [ ] **Step 5: Run the targeted tests to verify they pass**

Run: `cd backend && cargo test games::tests::starts_gameplay_sessions -- --exact`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add backend/src/games/models.rs backend/src/games/service.rs backend/src/games/repository.rs backend/src/error.rs backend/src/games/tests.rs
git commit -m "feat: start gameplay sessions from selected packs"
```

---

### Task 7: Add Game State Read Models And Query Endpoints

**Files:**
- Create: `backend/src/games/state.rs`
- Modify: `backend/src/games/mod.rs`
- Modify: `backend/src/games/models.rs`
- Modify: `backend/src/games/service.rs`
- Modify: `backend/src/games/api.rs`
- Modify: `backend/src/app.rs`
- Test: `backend/src/games/tests.rs`

- [ ] **Step 1: Write the failing game-state tests**

Add tests that expect:
- player and admin state endpoints return current scoreboard, phase, board, and active clue state
- reading before start returns the right app error

- [ ] **Step 2: Run the targeted tests to verify they fail**

Run: `cd backend && cargo test games::tests::returns_game_views_for_players_and_admins -- --exact`
Expected: FAIL because gameplay state queries do not exist.

- [ ] **Step 3: Build `GameView` assembly helpers**

Convert runtime engine state into stable API shapes:
- scoreboard
- board categories/clues with answered state
- active clue metadata
- phase/round information

- [ ] **Step 4: Add service, API, and route wiring**

Expose:
- `GET /packs`
- `GET /games/player/{player_code}/state`
- `GET /games/admin/{admin_code}/state`

- [ ] **Step 5: Run the targeted tests to verify they pass**

Run: `cd backend && cargo test games::tests::returns_game_views_for_players_and_admins -- --exact`
Expected: PASS

- [ ] **Step 6: Commit**

```bash
git add backend/src/games/state.rs backend/src/games/mod.rs backend/src/games/models.rs backend/src/games/service.rs backend/src/games/api.rs backend/src/app.rs backend/src/games/tests.rs
git commit -m "feat: expose gameplay state queries"
```

---

### Task 8: Add Admin-Driven Gameplay Command Endpoints

**Files:**
- Modify: `backend/src/games/models.rs`
- Modify: `backend/src/games/service.rs`
- Modify: `backend/src/games/api.rs`
- Modify: `backend/src/app.rs`
- Modify: `backend/src/error.rs`
- Test: `backend/src/games/tests.rs`

- [ ] **Step 1: Write the failing gameplay command tests**

Add tests for:
- select clue
- attempt answer
- Daily Double wager and resolution
- Final Jeopardy wager and resolution
- game completion metadata updates

- [ ] **Step 2: Run the targeted tests to verify they fail**

Run: `cd backend && cargo test games::tests::applies_admin_gameplay_commands_through_completion -- --exact`
Expected: FAIL because command endpoints do not exist.

- [ ] **Step 3: Add request DTOs per command**

Keep DTOs explicit instead of multiplexing everything into a loose union body.

- [ ] **Step 4: Implement service command dispatch**

Each command should:
- validate admin token
- fetch runtime session
- translate API request into a `GameAction`
- apply it through the engine
- persist completion timestamp when the game reaches `Completed`

- [ ] **Step 5: Add HTTP handlers and routes**

Register:
- `POST /games/start`
- `POST /games/actions/select-clue`
- `POST /games/actions/attempt-answer`
- `POST /games/actions/daily-double-wager`
- `POST /games/actions/resolve-daily-double`
- `POST /games/actions/final-wager`
- `POST /games/actions/resolve-final`

- [ ] **Step 6: Run the targeted tests to verify they pass**

Run: `cd backend && cargo test games::tests::applies_admin_gameplay_commands_through_completion -- --exact`
Expected: PASS

- [ ] **Step 7: Commit**

```bash
git add backend/src/games/models.rs backend/src/games/service.rs backend/src/games/api.rs backend/src/app.rs backend/src/error.rs backend/src/games/tests.rs
git commit -m "feat: add admin-driven gameplay commands"
```

---

### Task 9: Add Integration Coverage Across The Full Backend Slice

**Files:**
- Modify: `backend/src/games/tests.rs`
- Modify: `backend/src/domain/randomized/mod.rs`
- Modify: `backend/src/domain/randomized/scenario.rs`
- Test: `backend/src/games/tests.rs`

- [ ] **Step 1: Write the failing integration-style backend test**

Add one high-value flow test:
- create game
- join players
- join admin and receive token
- start game from a JSON pack
- drive a short but complete game to completion

- [ ] **Step 2: Run the targeted test to verify it fails**

Run: `cd backend && cargo test games::tests::full_backend_gameplay_flow -- --exact`
Expected: FAIL because at least one required gameplay layer is missing.

- [ ] **Step 3: Extend randomized helpers only if needed**

If the existing randomized engine fixtures can be reused for pack-backed integration tests, do that. Avoid duplicating scenario generation logic.

- [ ] **Step 4: Run the targeted test to verify it passes**

Run: `cd backend && cargo test games::tests::full_backend_gameplay_flow -- --exact`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add backend/src/games/tests.rs backend/src/domain/randomized
git commit -m "test: cover full backend gameplay lifecycle"
```

---

### Task 10: Update Backend Gameplay Documentation

**Files:**
- Create: `docs/testing/backend-gameplay.md`
- Modify: `docs/testing/randomized-testing.md`

- [ ] **Step 1: Draft the new backend gameplay testing doc**

Document:
- required env vars
- migration flow
- pack location
- start-game and command lifecycle
- how gameplay tests relate to randomized engine tests

- [ ] **Step 2: Update the randomized testing doc**

Add a short section explaining that the randomized engine remains the rules source of truth under the new gameplay API layer.

- [ ] **Step 3: Review the docs for consistency with the spec and plan**

Check path names, env vars, and route names exactly.

- [ ] **Step 4: Commit**

```bash
git add docs/testing
git commit -m "docs: add backend gameplay testing guide"
```

---

### Task 11: Run Final Verification

**Files:**
- No new files

- [ ] **Step 1: Run backend formatting**

Run: `cd backend && cargo fmt`
Expected: no errors

- [ ] **Step 2: Run the backend targeted gameplay suite**

Run: `cd backend && cargo test games::tests`
Expected: PASS

- [ ] **Step 3: Run the full backend test suite**

Run: `cd backend && DATABASE_URL=postgres://postgres:password@127.0.0.1:5432/docker cargo test`
Expected: PASS

- [ ] **Step 4: Run the randomized backend engine suite**

Run: `cd backend && cargo test --lib randomized_backend_engine_cases_are_deterministic_and_replayable -- --exact --nocapture`
Expected: PASS

- [ ] **Step 5: Check git diff for unrelated changes**

Run: `git status --short`
Expected: only planned gameplay/doc changes remain

- [ ] **Step 6: Commit the final verification pass**

```bash
git add -A
git commit -m "chore: verify backend gameplay slice"
```

