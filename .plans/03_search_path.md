# Plan 03 — In-App Path Input + Search Path

## Goal
1. Replace OS file dialog (`rfd::FileDialog`) with **in-app path input**.
2. Add **search path** to filter commits by file/directory (like `git log -- <path>`).

## Problem Now
- `main.rs` uses `rfd::FileDialog::new().pick_folder()` → OS popup to pick repo.
- No in-app way to type a repo path.
- No way to filter commits by file path.

## Architecture

```
┌──────────────────────────────────────────────────────┐
│  TitleBar                                            │
│  MenuBar                                             │
│  ┌──────────────────────────────────────────────────┐│
│  │  PathBar (new)                                   ││
│  │  [ repo path input ] [Open]  [search path] [🔍]  ││
│  └──────────────────────────────────────────────────┘│
├──────────┬───────────────────────────────────────────┤
│ Sidebar  │  Content                                  │
│ branches │  commit graph (filtered by search path)   │
│ status   │  changed files / diff                     │
├──────────┴───────────────────────────────────────────┤
│  StatusBar                                           │
└──────────────────────────────────────────────────────┘
```

---

## Implementation Steps

### Step 01 — PathBar component
**New file**: `src/path_bar.rs`

A horizontal bar below the menu with two text inputs:

- **Repo path** input + "Open" button → calls `garph.update_repo(&path)`.
- **Search path** input + "Filter" button → calls `garph.set_search_path(path)`.

Fields:
```
struct PathBar {
    repo_input: String,      // typed repo path
    search_input: String,    // typed search/filter path
    error_msg: Option<String>, // show if repo open fails
}
```

Events emitted:
- `RepoPathSubmitted { path: String }`
- `SearchPathSubmitted { path: String }`
- `SearchPathCleared`

**Status**: ✅ Done

### Step 02 — Remove rfd dependency
- Remove `rfd` from `Cargo.toml`.
- Remove `OpenFile` action from `actions.rs`.
- Remove `rfd::FileDialog` logic from `main.rs`.
- Remove `OpenFile` handler from `main.rs`.

**Files**: `Cargo.toml`, `main.rs`, `actions.rs`, `workspace.rs` (menu Open item)

**Status**: ✅ Done

### Step 03 — Repo change propagation
When user submits a new repo path via PathBar:
- `Workspace` receives `RepoPathSubmitted`.
- Calls `garph.update_repo(&path)`.
- `Garph` already emits `RepoPathChanged`.
- Workspace subscribes → reloads `BranchPanel`, `StatusPanel`, `StatusBar`.
- Clears search input.

**Files**: `workspace.rs`

**Status**: ✅ Done

### Step 04 — Path-filtered revwalk in garph.rs
Add search path filtering to `Garph`:

- New field: `search_path: Option<String>`.
- New method: `set_search_path(&mut self, path: Option<String>)`.
- `recompute_bg` uses `git2::DiffOptions::pathspec()` to filter commits.

Core logic (background thread):
```
if search_path is Some(path):
    for each commit in revwalk:
        let diff = diff_tree_to_tree(parent, commit, DiffOptions.pathspec(&path))
        if diff.deltas().count() > 0 → include commit
else:
    normal revwalk (show all)
```

**Files**: `garph.rs`

**Status**: ✅ Done

### Step 05 — Wire PathBar search to garph
- Workspace subscribes to `SearchPathSubmitted` → `garph.set_search_path(Some(path))`.
- Workspace subscribes to `SearchPathCleared` → `garph.set_search_path(None)`.
- Graph recomputes in background thread.

**Files**: `workspace.rs`

**Status**: ✅ Done

---

## File Changes Summary

| File | Action | Notes |
|---|---|---|
| `src/path_bar.rs` | **NEW** | Repo path + search path input bar |
| `src/garph.rs` | **MODIFY** | Add `search_path` field + pathspec filter |
| `src/workspace.rs` | **MODIFY** | Add PathBar, wire events, repo change propagation |
| `src/actions.rs` | **MODIFY** | Remove `OpenFile`, keep `Quit` |
| `src/main.rs` | **MODIFY** | Remove `rfd` + `OpenFile` handler |
| `src/lib.rs` | **MODIFY** | Add `pub mod path_bar` |
| `Cargo.toml` | **MODIFY** | Remove `rfd` |

---

## Data Flow

```
User types repo path → clicks "Open"
  → PathBar emits RepoPathSubmitted
    → Workspace calls garph.update_repo(path)
      → Garph opens repo, emits RepoPathChanged
        → Workspace reloads all panels
        → Clears search input

User types search path → clicks "Filter"
  → PathBar emits SearchPathSubmitted
    → Workspace calls garph.set_search_path(Some(path))
      → Garph marks dirty, spawns recompute_bg
        → recompute_bg uses pathspec filter
          → returns filtered GraphData
            → Garph renders filtered graph

User clicks "Clear" on search
  → PathBar emits SearchPathCleared
    → Workspace calls garph.set_search_path(None)
      → Full graph recompute
```

---

## Constraints
- No OS dialogs. All input is in-app text fields.
- Zero copy: `Rc<RefCell<Option<Repository>>>` shared, no cloning repo.
- Background thread: pathspec filtering off main thread.
- Keep `LIMIT_ROW` — cap results.
- Partial match: pathspec supports prefix matching.

## Status: ✅ Done
