# Plan 06 — Type & Auto-Suggest

## Problem

The path bar has a text input for filtering commits by file path, but it offers no assistance:
- User must know the exact path to type
- No feedback on what paths exist in the repo
- No auto-complete / suggestion dropdown
- Only filters after pressing Enter or clicking "Filter"

## Goal

Add real-time auto-suggest that shows matching file paths as the user types in the search input. Similar to how the repo picker dropdown works.

## Architecture

```
┌─────────────────────────────────────────────────┐
│ REPO [▼ picker] │ PATH [src/lib.r█] [Filter] [✕]│
│                  │ ┌─────────────────────────┐  │
│                  │ │ src/lib.rs              │  │
│                  │ │ src/lane.rs             │  │
│                  │ │ src/language.rs         │  │
│                  │ └─────────────────────────┘  │
└─────────────────────────────────────────────────┘
```

## Data Flow

```
User types char → TextInput content changes
  → PathBar detects change (poll or event)
  → filter suggestions from cached file list
  → show suggestion dropdown
User clicks suggestion → set TextInput text → emit SearchPathSubmitted
```

## Implementation Steps

### Step 1: Create `src/suggest.rs` — AutoSuggest module

New module with:
- `SuggestState` — holds cached file paths from repo + filtered results
- `SuggestState::new()` — empty
- `SuggestState::set_paths(paths: Vec<String>)` — cache all known paths
- `SuggestState::filter(query: &str) -> Vec<&str>` — fuzzy-prefix match, returns top N
- `SuggestState::is_empty() -> bool`

Key design:
- Zero-copy filter: borrow `&str` slices from cached paths
- Cache populated lazily when repo loads (background thread via `git ls-tree -r` or `repo.index()`)
- Max 10 suggestions shown
- Prefix match (case-insensitive) on path segments

### Step 2: Background path collection

Add to `garph.rs`:
- `pub fn collect_paths_bg(repo_path: String) -> Result<Vec<String>, Box<dyn Error + Send + Sync>>`
- Uses `repo.index()` to get all tracked file paths (fast, no diff needed)
- Called once when repo is set, cached in workspace

### Step 3: Wire `SuggestState` into `PathBar`

- `PathBar` holds `SuggestState` + `suggest_open: bool`
- On every render, read `search_input.text()`, call `suggest_state.filter()`
- Render suggestion dropdown below the input (absolute positioned)
- Click suggestion → set input text → emit `SearchPathSubmitted`
- Close dropdown on click outside or Escape

### Step 4: Populate paths on repo load

In `workspace.rs`:
- When `on_repo_path_submitted` fires, also spawn `collect_paths_bg`
- Store result via `mpsc::channel` + poll pattern (same as other bg ops)
- Pass paths to `PathBar` via `path_bar.update(cx, |pb, _| pb.set_suggest_paths(paths))`

### Step 5: Text change detection

Current `TextInput` doesn't emit per-keystroke events. Two options:

**Option A (minimal):** Poll text on every `PathBar::render()` — already happens since `render()` runs frequently. Call `filter()` each render. Simple, no changes to `TextInput`.

**Option B (event-driven):** Add `TextChanged` event to `TextInput`. More proper but more changes.

→ **Go with Option A** — lean, no over-engineering. GPUI re-renders on focus/keystroke anyway since the text input handles key events and calls `cx.notify()`.

## Files

| File | Action | Notes |
|---|---|---|
| `src/suggest.rs` | NEW | `SuggestState` — path cache + filter |
| `src/garph.rs` | MODIFY | Add `collect_paths_bg()` function |
| `src/path_bar.rs` | MODIFY | Hold `SuggestState`, render dropdown, handle selection |
| `src/workspace.rs` | MODIFY | Spawn path collection on repo load, pass results to PathBar |
| `src/lib.rs` | MODIFY | Add `pub mod suggest` |
| `.plans/06_type_auto_suggest.md` | NEW | This plan |

## Constants

```rust
// suggest.rs
const MAX_SUGGESTIONS: usize = 10;
const MIN_QUERY_LEN: usize = 1; // start suggesting after 1 char
```

## Execution Order

```
06.1  create suggest.rs — SuggestState struct + filter logic
06.2  add collect_paths_bg() to garph.rs
06.3  wire SuggestState into PathBar — render dropdown
06.4  wire path collection in workspace.rs — populate on repo load
06.5  cargo check && cargo clippy
06.6  cargo test
```

## Status: ✅ Done
