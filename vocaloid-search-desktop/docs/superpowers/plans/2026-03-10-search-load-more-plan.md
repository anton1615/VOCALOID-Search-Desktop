# Search Load-more Independence Implementation Plan

> **For agentic workers:** REQUIRED: Use superpowers:subagent-driven-development (if subagents available) or superpowers:executing-plans to implement this plan. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Ensure Search load-more works regardless of active playback list, by validating list-context version rather than `active_playlist_type`.

**Architecture:** Keep active playback and browsing state independent. `load_more` should validate the requested list context (Search) using its version, not the active playback list. SearchView keeps its current behavior, but backend uses list-context/versions for gating.

**Tech Stack:** Rust (Tauri backend), Vue 3 frontend, Vitest/Jest (existing tests), cargo check.

---

## File Map
- Modify: `src-tauri/src/commands.rs` — change `load_more` gating logic
- Modify: `src-tauri/src/state.rs` — add helper for list-context gating if needed
- Modify: `src/views/SearchView.vue` — only if front-end needs to pass list-context id/version explicitly (prefer no change)
- Test: `src-tauri/src/state.rs` (unit tests) and/or frontend tests if applicable

## Chunk 1: Backend load-more gating
### Task 1: Replace active-playlist gating with list-context gating

**Files:**
- Modify: `src-tauri/src/commands.rs` (load_more)
- Modify: `src-tauri/src/state.rs` (helper + tests)

- [ ] **Step 1: Write failing test (Rust)**

```rust
#[test]
fn accepts_search_load_more_even_when_active_playback_is_history() {
    let active_list_id = ListContextId::Search;
    let requested_list_id = ListContextId::Search;
    let current_version = 2;
    let expected_version = 2;
    assert!(should_accept_list_context_load_more(&active_list_id, &requested_list_id, current_version, expected_version));
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cargo test -p vocaloid-search-desktop -- state::tests::accepts_search_load_more_even_when_active_playback_is_history`

Expected: FAIL (function not wired or wrong logic)

- [ ] **Step 3: Implement minimal backend change**

Update `load_more` in `src-tauri/src/commands.rs`:
- Replace `active_playlist_type` check with list-context check.
- Use `ListContextId::Search` as requested list id.
- If `state.list_contexts` has Search context, validate version against that; otherwise fall back to `search_state.version`.

- [ ] **Step 4: Run test to verify it passes**

Run: `cargo test -p vocaloid-search-desktop -- state::tests::accepts_search_load_more_even_when_active_playback_is_history`

Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src-tauri/src/state.rs src-tauri/src/commands.rs
git commit -m "fix: decouple search load-more from playback"
```

## Chunk 2: Verification
### Task 2: Manual verification

- [ ] **Step 1: Run app**

Run: `npm run tauri dev`

- [ ] **Step 2: Repro flow**

1. Play a video from History.
2. Switch to Search.
3. Scroll to the end of Search list.
4. Confirm new results append successfully.

- [ ] **Step 3: Commit verification note (optional)**

If you track manual verification in a note file, append it. Otherwise skip.

---

## Notes
- Keep existing frontend `loadMore()` signature (SearchView already passes version).
- Do not reintroduce playback coupling in future list-context work.
