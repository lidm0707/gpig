# Plan 09 — Repo View Mode (Local / Remote)

## Goal

A toggle near the repo picker to switch the **view** of the selected repo:
- **LOCAL** — local branches, commit graph from HEAD (current behavior)
- **REMOTE** — remote branches, commit graph from all remotes

This is just a view filter. No clone, no new repo opening. Same repo, different perspective.

## Current State

| What | Status |
|---|---|
| Repo picker dropdown | ✅ works — scan + select local repos |
| Branch panel | shows `BranchType::Local` only |
| Graph (`recompute_bg`) | walks `push_head()` only — local HEAD |
| Status bar | shows branch name from HEAD |

## Design

```
┌──────────────────────────────────────────────────────┐
│  Path Bar                                            │
│  REPO [▼ repo picker] [LOCAL ●] [REMOTE ○] | PATH…  │
└──────────────────────────────────────────────────────┘
```

### LOCAL view (default, current behavior)
- Branch panel: `repo.branches(Local)`
- Graph: `revwalk.push_head()`
- Status bar: branch from HEAD

### REMOTE view
- Branch panel: `repo.branches(Remote)` — show `origin/main`, `origin/feat`, etc.
- Graph: `revwalk.push_head()` still (remote branches share same commits)
- Status bar: show tracking branch info
- Click remote branch → checkout its local tracking branch (if exists)

## Modified Files

| File | Action | Notes |
|---|---|---|
| `src/path_bar.rs` | MODIFIED | Add `RepoMode` toggle, emit `ViewModeChanged` |
| `src/branch.rs` | MODIFIED | Accept mode, reload local or remote branches |
| `src/garph.rs` | MODIFIED | Optionally walk remote refs in REMOTE mode |
| `src/workspace.rs` | MODIFIED | Listen to `ViewModeChanged`, reload panels |

## Details

### `path_bar.rs`

- Add `RepoMode { Local, Remote }` enum
- Add mode toggle buttons after repo picker
- Emit `ViewModeChanged { mode: RepoMode }` event
- Remove previous clone-related code (`url_input`, `pending_clone_rx`, `cloning`, clone functions)

### `branch.rs`

- Add `mode: RepoMode` field
- `reload()` uses mode to pick `BranchType::Local` or `BranchType::Remote`
- Remote branch display: show full name `origin/main` or strip remote prefix
- Click remote branch: find local tracking branch and checkout, or show error

### `garph.rs`

- No fundamental change to graph walking — commits are same locally and remotely
- Could add remote ref markers on graph nodes later (stretch goal)

### `workspace.rs`

- Subscribe to `ViewModeChanged` from path_bar
- Forward mode to branch_panel and status_bar

## Implementation Steps

- [x] 1. Clean `path_bar.rs` — remove clone code, add `RepoMode` + `ViewModeChanged` event
- [x] 2. Add mode toggle render after repo picker in path_bar
- [x] 3. Update `branch.rs` — add mode field, reload by `BranchType`
- [x] 4. Wire `ViewModeChanged` in `workspace.rs`
- [x] 5. `cargo check && cargo clippy` pass
- [x] 6. Update this plan

## Status: ✅ DONE

## Post-fix

Branch checkout now runs on a background thread (`std::thread::spawn` + `mpsc::channel`) and polls result in render loop — same pattern as `garph.rs`. Shows `⏳ branch_name` while checkout is in progress. Blocks clicks while busy.

### Bug fix: RepoPathChanged never emitted

`Garph.update_repo()` declared `EventEmitter<RepoPathChanged>` but never emitted the event. This meant:
- `on_repo_path_changed` in workspace never fired
- `reload_status_panels()` never ran on repo open
- `BranchPanel.repo_path` stayed `None` → checkout silently returned early
- Branch list never reloaded after repo selection

Fix: `update_repo()` now takes `cx: &mut Context<Self>` and emits `RepoPathChanged` + `cx.notify()`. Removed duplicate `spawn_path_collection` from `on_repo_path_submitted` since `on_repo_path_changed` already handles it.
