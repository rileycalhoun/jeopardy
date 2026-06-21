# Jeopardy Clue Normalization Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Rewrite all category-pack clue text into Jeopardy-style clue statements with fully normalized question-form responses.

**Architecture:** Leave the category loader and schema untouched, and update only the pack content in `backend/categories/*.json`. Verification relies on JSON parsing plus a simple structural check that each response is in question form.

**Tech Stack:** JSON category packs, Python verification helpers

---

### Task 1: Rewrite category-pack content

**Files:**
- Modify: `backend/categories/gen_z_slang.json`
- Modify: `backend/categories/grammar.json`
- Modify: `backend/categories/movies.json`
- Modify: `backend/categories/music.json`
- Modify: `backend/categories/musicals.json`
- Modify: `backend/categories/science.json`
- Modify: `backend/categories/tv.json`
- Modify: `backend/categories/us_history.json`

- [ ] **Step 1: Rewrite every clue as an answer-style statement**

Convert direct questions like `What planet is closest to the sun` into clue statements like `This planet is closest to the sun.`

- [ ] **Step 2: Rewrite every response into question form**

Convert raw answers like `mercury` into responses like `What is Mercury?`

- [ ] **Step 3: Preserve structure**

Keep each file's `id`, `title`, `points`, and array structure unchanged.

### Task 2: Verify the normalized packs

**Files:**
- Test: `backend/categories/*.json`

- [ ] **Step 1: Parse every category file as JSON**

Run: `python3 - <<'PY'\nimport json\nfrom pathlib import Path\nfor path in sorted(Path('backend/categories').glob('*.json')):\n    json.loads(path.read_text())\n    print(path)\nPY`
Expected: every file path prints with no parse errors

- [ ] **Step 2: Check that every answer is in question form**

Run: `python3 - <<'PY'\nimport json, re\nfrom pathlib import Path\npattern = re.compile(r'^(Who|What|When|Where) (is|are) .+\\?$')\nfor path in sorted(Path('backend/categories').glob('*.json')):\n    data = json.loads(path.read_text())\n    for item in data['questions']:\n        if not pattern.match(item['answer']):\n            raise SystemExit(f\"bad answer form in {path}: {item['answer']}\")\nprint('all answers normalized')\nPY`
Expected: `all answers normalized`
