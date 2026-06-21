# Category Set Replacement Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Replace the current category-pack set with the Jeopardy-normalized categories from `backend/categories/categories.json`.

**Architecture:** Keep the existing category-pack schema and loader untouched, and replace only the content files under `backend/categories`. Remove the raw source file after import so the directory contains only valid pack JSON files.

**Tech Stack:** JSON category packs, Rust backend verification

---

### Task 1: Replace the category pack set

**Files:**
- Modify: `backend/categories/movies.json`
- Modify: `backend/categories/tv.json`
- Modify: `backend/categories/music.json`
- Modify: `backend/categories/musicals.json`
- Modify: `backend/categories/science.json`
- Modify: `backend/categories/gen_z_slang.json`
- Modify: `backend/categories/us_history.json`
- Modify: `backend/categories/grammar.json`
- Create: `backend/categories/u_u.json`
- Create: `backend/categories/disney.json`
- Create: `backend/categories/cartoon_network.json`
- Create: `backend/categories/nickelodeon.json`
- Delete: `backend/categories/categories.json`

- [ ] **Step 1: Rewrite the overlapping packs from the new source**
- [ ] **Step 2: Add the new packs that did not previously exist**
- [ ] **Step 3: Delete the raw source file so the loader sees only valid packs**

### Task 2: Verify the replacement

**Files:**
- Test: `backend/categories/*.json`

- [ ] **Step 1: Parse every category file as JSON**

Run: `python3 - <<'PY'\nimport json\nfrom pathlib import Path\nfor path in sorted(Path('backend/categories').glob('*.json')):\n    json.loads(path.read_text())\n    print(path)\nPY`
Expected: every pack file prints with no parse errors

- [ ] **Step 2: Check the resulting category file list**

Run: `find backend/categories -maxdepth 1 -type f | sort`
Expected: only the desired pack files remain

- [ ] **Step 3: Run backend verification**

Run: `cargo test --manifest-path backend/Cargo.toml`
Expected: PASS with the existing integration tests still ignored by default
