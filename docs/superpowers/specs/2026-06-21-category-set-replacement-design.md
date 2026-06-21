# Category Set Replacement Design

**Date:** 2026-06-21

## Goal

Replace the existing category-pack set with only the categories listed in `backend/categories/categories.json`, preserving the repo JSON schema and Jeopardy-style normalization.

## Scope

- Rewrite the overlapping category packs from the new source.
- Add new category packs for `U&U`, `Disney`, `Cartoon Network`, and `Nickelodeon`.
- Remove any raw or reference category files that should not be loaded by the app.

## Rules

- The only active categories should be the ones listed in `backend/categories/categories.json`.
- Each pack remains one JSON file with `id`, `title`, and `questions`.
- Each `question` stays as a Jeopardy-style clue statement.
- Each `answer` stays fully normalized in question form.
- Category IDs must conform to the loader format.

## Verification

- Parse every `backend/categories/*.json` file as JSON.
- Confirm only the desired pack files remain.
- Run `cargo test --manifest-path backend/Cargo.toml`.
