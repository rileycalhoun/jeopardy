# Classic Jeopardy Category Scale Design

**Date:** 2026-06-20

## Goal

Update the repo's category content model to use classic single-round Jeopardy values of `200`, `400`, `600`, `800`, and `1000`, then convert the raw category list into valid per-category JSON files under `backend/categories`.

## Scope

- Change the backend's canonical point-value constants and validation rules.
- Update tests and generated clue labels that currently assume `100` through `500`.
- Split the raw category source into individual category pack files that match the repo schema.
- Remove the invalid aggregate source file so category loading stays strict and clean.

## Non-Goals

- Support multiple point scales at runtime.
- Add a new import pipeline or parser for loose text files.
- Change gameplay beyond the clue values and category content format.

## Current State

- `backend/src/content/models.rs` defines `POINT_VALUES` as `[100, 200, 300, 400, 500]`.
- Category validation requires at least one question for each of those point values.
- Board generation uses those constants to choose one clue per value and label them as `"$<points>"`.
- `backend/categories/category_list.json` is not valid JSON for the loader and cannot be used directly.
- The provided source content is already structured around classic Jeopardy values: `200`, `400`, `600`, `800`, `1000`.

## Proposed Change

### 1. Canonical point scale

Adopt `POINT_VALUES = [200, 400, 600, 800, 1000]` as the single canonical scale for category packs and generated boards.

This keeps the code aligned with the incoming content and avoids adding conversion logic or configuration branches.

### 2. Content validation and board generation

Update category validation so every category must include at least one question for each of the new five point values.

Update board generation and related tests so:

- the required clue buckets are `200`, `400`, `600`, `800`, `1000`
- generated clue labels remain `"$<points>"`
- helper tests and fixtures no longer reference the old values

### 3. Category file format

Create one file per category in `backend/categories`, each shaped like:

```json
{
  "id": "movies",
  "title": "Movies",
  "questions": [
    { "points": 200, "question": "...", "answer": "..." },
    { "points": 400, "question": "...", "answer": "..." }
  ]
}
```

If a category has multiple questions at the same point value, keep all of them. The existing board builder already supports multiple questions per bucket and randomly selects one.

### 4. Raw source handling

Delete `backend/categories/category_list.json` after the individual files are created.

This avoids strict-loader failures in local development and ensures the directory contains only valid category packs.

## Category Mapping

The raw source will be split into these files:

- `backend/categories/movies.json`
- `backend/categories/tv.json`
- `backend/categories/music.json`
- `backend/categories/musicals.json`
- `backend/categories/science.json`
- `backend/categories/gen_z_slang.json`
- `backend/categories/us_history.json`
- `backend/categories/grammar.json`

Titles will remain user-facing title case. IDs will be normalized to the loader's allowed format using lowercase letters plus underscores where needed.

## Risks

- Any test or fixture that hardcodes the old point values will fail until updated.
- The raw source contains a few content-quality issues, but they are still structurally valid for the repo schema. This change will preserve the supplied wording rather than editorialize beyond necessary JSON escaping and whitespace cleanup.

## Verification

- Run `cargo test --manifest-path backend/Cargo.toml`
- Optionally inspect category loading by ensuring the `backend/categories` directory contains only valid JSON category packs

