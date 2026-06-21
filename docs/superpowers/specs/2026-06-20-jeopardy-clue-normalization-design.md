# Jeopardy Clue Normalization Design

**Date:** 2026-06-20

## Goal

Normalize every category pack entry so each `question` field reads like a Jeopardy clue statement and each `answer` field is fully formatted as a response in the form of a question.

## Scope

- Rewrite `backend/categories/*.json` content only.
- Keep file names, IDs, titles, point values, and JSON structure unchanged.
- Normalize capitalization, punctuation, and response phrasing where needed.

## Non-Goals

- Changing the category schema.
- Re-researching or replacing category subject matter.
- Adding import scripts or runtime formatting logic.

## Formatting Rules

### Clue field (`question`)

Each `question` value should become an answer-style clue statement rather than a direct question.

Examples:

- direct question: `What planet is closest to the sun`
- normalized clue: `This planet is closest to the sun.`

- direct question: `Who invented the telephone`
- normalized clue: `This inventor is credited with inventing the telephone.`

### Response field (`answer`)

Each `answer` value should become a fully normalized Jeopardy-style response:

- people: `Who is ...?`
- plural people/groups: `Who are ...?`
- things, places, concepts, titles, years: `What is ...?`
- plural things/lists: `What are ...?`

## Content Policy For This Rewrite

- Preserve the factual intent of the source material.
- Smooth awkward wording into playable clue language.
- Fix obvious capitalization and punctuation issues.
- Do not broaden scope into fact-checking or category replacement work.

## Verification

- Parse every `backend/categories/*.json` file as JSON.
- Check that every `answer` is in question form.
- Spot-check that `question` fields are clue statements rather than direct interrogatives.
