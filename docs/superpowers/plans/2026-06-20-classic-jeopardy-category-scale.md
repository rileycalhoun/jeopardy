# Classic Jeopardy Category Scale Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Update the backend category model to use classic Jeopardy values `200`, `400`, `600`, `800`, and `1000`, then replace the raw category list with valid per-category JSON packs.

**Architecture:** Keep the existing category loading and board generation flow, but change the shared `POINT_VALUES` constant so validation, selection, and clue labeling all derive from one classic scale. Preserve the current loader format by generating one JSON file per category in `backend/categories` and removing the invalid aggregate source file.

**Tech Stack:** Rust, serde, sqlx-backed backend tests, JSON category packs

---

### Task 1: Update classic point-scale tests first

**Files:**
- Modify: `backend/src/content/models.rs`
- Modify: `backend/src/content/service.rs`
- Test: `backend/src/content/models.rs`
- Test: `backend/src/content/service.rs`

- [ ] **Step 1: Write the failing tests**

```rust
#[test]
fn category_requires_all_classic_jeopardy_point_values_for_gameplay() {
    let mut category = valid_category();
    category.questions.retain(|question| question.points != 1000);

    let err = category
        .validate()
        .expect_err("missing point value should be rejected");

    assert!(
        err.to_string()
            .contains("no questions for required point value 1000")
    );
}

#[test]
fn generated_board_uses_one_question_per_classic_point_value_per_category() {
    let categories = vec![
        category_pack("video_games", "Video Games"),
        category_pack("movies", "Movies"),
    ];

    let board = build_board_from_categories(&categories).expect("board should build");

    assert_eq!(board.rounds[0].categories[0].clues.len(), 5);
    for category in &board.rounds[0].categories {
        let values: Vec<i32> = category.clues.iter().map(|clue| clue.value).collect();
        assert_eq!(values, vec![200, 400, 600, 800, 1000]);
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --manifest-path backend/Cargo.toml content::models::category_tests::category_requires_all_classic_jeopardy_point_values_for_gameplay content::service::tests::generated_board_uses_one_question_per_classic_point_value_per_category`
Expected: FAIL because the current implementation still uses `100`, `200`, `300`, `400`, `500`

- [ ] **Step 3: Write the minimal implementation**

```rust
pub const POINT_VALUES: [i32; 5] = [200, 400, 600, 800, 1000];
```

```rust
for points in [200, 400, 600, 800, 1000] {
    questions.push(category_question(points, "A"));
    questions.push(category_question(points, "B"));
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --manifest-path backend/Cargo.toml content::models::category_tests::category_requires_all_classic_jeopardy_point_values_for_gameplay content::service::tests::generated_board_uses_one_question_per_classic_point_value_per_category`
Expected: PASS

### Task 2: Convert the raw categories into valid pack files

**Files:**
- Create: `backend/categories/movies.json`
- Create: `backend/categories/tv.json`
- Create: `backend/categories/music.json`
- Create: `backend/categories/musicals.json`
- Create: `backend/categories/science.json`
- Create: `backend/categories/gen_z_slang.json`
- Create: `backend/categories/us_history.json`
- Create: `backend/categories/grammar.json`
- Delete: `backend/categories/category_list.json`
- Test: `backend/categories/*.json`

- [ ] **Step 1: Create valid category-pack JSON files**

Each file must follow:

```json
{
  "id": "movies",
  "title": "Movies",
  "questions": [
    { "points": 200, "question": "...", "answer": "..." },
    { "points": 400, "question": "...", "answer": "..." },
    { "points": 600, "question": "...", "answer": "..." },
    { "points": 800, "question": "...", "answer": "..." },
    { "points": 1000, "question": "...", "answer": "..." }
  ]
}
```

- [ ] **Step 2: Remove the invalid aggregate source**

Delete:

```text
backend/categories/category_list.json
```

- [ ] **Step 3: Verify the files are present and shaped correctly**

Run: `find backend/categories -maxdepth 1 -type f | sort`
Expected: only per-category `.json` files remain in the directory

### Task 3: Run full backend verification

**Files:**
- Modify: `backend/src/content/models.rs`
- Modify: `backend/src/content/service.rs`
- Create: `backend/categories/*.json`
- Delete: `backend/categories/category_list.json`
- Test: `backend/Cargo.toml`

- [ ] **Step 1: Run the full backend test suite**

Run: `cargo test --manifest-path backend/Cargo.toml`
Expected: PASS with the existing integration tests still ignored by default

- [ ] **Step 2: Spot-check category content loading shape**

Run: `find backend/categories -maxdepth 1 -type f | sort`
Expected: the directory contains valid individual category packs and no invalid aggregate file
