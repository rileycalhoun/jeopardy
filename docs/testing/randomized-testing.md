# Randomized Testing

This repository now has a deterministic randomized testing layer centered on the backend Jeopardy engine.

## Why it exists

The backend owns Jeopardy rules:

- phase transitions
- clue selection
- score updates
- Daily Double wager handling
- Final Jeopardy wager handling
- invalid action rejection
- game completion

Randomized backend tests generate many valid and intentionally-invalid action sequences, apply them to the engine, and check invariants after every step. When a failure appears, the seed and case index are printed so the run can be replayed exactly.

The frontend randomized tests are smaller and render-focused. They only verify that unusual but valid lobby-style display states render safely. They do not duplicate backend rule logic.

## Backend commands

Run the backend randomized suite:

```bash
cd backend
cargo test --lib randomized_backend_engine_cases_are_deterministic_and_replayable -- --exact --nocapture
```

Run the full backend library test suite:

```bash
cd backend
cargo test --lib
```

Control deterministic replay with environment variables:

```bash
cd backend
TEST_SEED=123456 cargo test --lib randomized_backend_engine_cases_are_deterministic_and_replayable -- --exact --nocapture
```

Increase randomized coverage:

```bash
cd backend
TEST_SEED=123456 RANDOM_TEST_CASES=1000 RANDOM_TEST_MAX_ACTIONS=800 \
  cargo test --lib randomized_backend_engine_cases_are_deterministic_and_replayable -- --exact --nocapture
```

Replay a single failing case:

```bash
cd backend
TEST_SEED=123456 RANDOM_TEST_CASES=17 RANDOM_TEST_CASE_INDEX=16 \
  cargo test --lib randomized_backend_engine_cases_are_deterministic_and_replayable -- --exact --nocapture
```

Notes:

- `TEST_SEED` fixes the root seed.
- `RANDOM_TEST_CASES` controls how many cases are generated.
- `RANDOM_TEST_MAX_ACTIONS` caps action count per game.
- `RANDOM_TEST_CASE_INDEX` isolates one generated case from the seeded run.
- If `TEST_SEED` is omitted, the suite generates one and prints it.

## Failure output

Backend randomized failures print:

- root seed
- case index
- derived case seed
- action index
- replay command
- action history
- failing action
- error
- serialized game-state snapshot

The intended workflow is:

1. copy the printed `TEST_SEED`
2. copy the `RANDOM_TEST_CASE_INDEX`
3. rerun exactly one case
4. fix the bug
5. add a permanent regression case if the bug should never recur

## Regression support

Permanent randomized seed regressions live in:

```text
backend/tests/regressions/jeopardy_random_seeds.txt
```

Format:

```text
<root-seed> <case-index> <optional note...>
```

Examples:

```text
7 0 baseline replay
42 0 small mixed board replay
1337 0 longer random board replay
```

Add a new saved replay by appending the seed and case index from a failing run.

The backend also includes direct hand-authored regression cases for:

- tie-game flow
- all players negative
- all players at zero
- Daily Double max wager
- repeated clue selection
- invalid action in the wrong phase
- completed board state

## Frontend commands

Run the frontend tests:

```bash
cd frontend
npm test
```

Run frontend type-checking and linting:

```bash
cd frontend
npm run check
npm run lint
```

The frontend randomized render coverage currently targets lobby-state rendering because the frontend does not yet render a full Jeopardy board or clue-resolution UI. As the frontend grows, render fixtures should stay display-focused and continue to consume backend-shaped state instead of reimplementing game rules.

## Current assumptions

The backend engine introduced for randomized testing makes a few explicit assumptions because the repository did not previously contain a Jeopardy rules implementation:

- zero-score players are allowed into Final Jeopardy and may wager zero
- negative-score players are excluded from Final Jeopardy
- Daily Double wagers must be between `1` and the allowed maximum
- when everyone misses a clue, control stays with the player who selected it
- Final Jeopardy reveal order runs from lower score to higher score

If the project later chooses different game rules, update the engine and randomized invariants together so replayability stays intact.
