# Plan 10 — Async Panel Loading

## Goal

Move all blocking git I/O off the main thread to prevent UI freezes.
When the Wayland compositor detects the app is unresponsive, it kills the process.

## Problem

`BranchPanel::reload()`, `StatusPanel::reload()`, and `StatusBar::refresh()` all do blocking
git operations (`repo.branches()`, `repo.statuses()`, `repo.revwalk()`) on the main render thread.
This freezes the Wayland compositor which then shows "application not responding" and kills the process.

## Solution

Apply the same pattern already used in `garph.rs`:
1. Background thread opens its own `Repository::open(repo_path)`
2. Does the heavy work
3. Sends results via `mpsc::channel`
4. Poll `rx.try_recv()` in render loop
5. Show loading state while pending

## Modified Files

| File | Change |
|---|---|
| `src/branch.rs` | `reload()` → `spawn_reload()` on bg thread, `poll_reload()` in render, loading state |
| `src/status_panel.rs` | Same pattern — bg thread for `repo.statuses()` |
| `src/status_bar.rs` | Same pattern — bg thread for `repo.head()` + `repo.revwalk()` + `repo.statuses()` |
| `src/workspace.rs` | Pass `repo_path` to all panels in `reload_status_panels()` |

## Details

### BranchPanel
- Removed `repo: Rc<RefCell<...>>` field — uses `repo_path: Option<String>` instead
- `spawn_reload()` → `std::thread::spawn` → `load_branches_bg(repo_path, mode)`
- `poll_reload()` in `render()` — shows `⏳ loading...` while pending
- `set_mode()` calls `spawn_reload()` instead of sync `reload()`

### StatusPanel
- Same pattern — `load_status_bg(repo_path)` on bg thread
- Shows `Loading status...` while pending

### StatusBar
- Same pattern — `load_status_bar_bg(repo_path)` on bg thread
- Shows `loading...` in branch field while pending

### Workspace
- `reload_status_panels()` now calls `set_repo_path()` on all three panels

## Status: ✅ DONE
