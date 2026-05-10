# Plan 11 — Fix Slow Panel Loading

## Problem

When a repo opens or branch changes, 4 background threads all open `Repository::open()`
simultaneously, and the status bar walks every commit with `revwalk.count()`.

### Bottlenecks

| # | Where | Cost | Fix |
|---|---|---|---|
| 1 | `load_status_bar_bg` → `revwalk.count()` | O(n) all commits — **the worst** | Remove. Use garph `nodes.len()` or skip |
| 2 | `repo.statuses()` called twice | Disk scan × 2 | StatusPanel owns dirty count, StatusBar reads it |
| 3 | `Repository::open()` × 4 threads | 4 × pack index load | Merge status + branch into one thread |
| 4 | All panels reload on every checkout | Unnecessary work | Only reload what changed |

## Design

### Consolidate into 2 background tasks (not 4)

**Task A — garph recompute** (unchanged): walks revwalk, builds graph nodes

**Task B — panel data**: single thread opens repo once, computes:
- branch list (local or remote)
- status entries (dirty files)
- branch name (current HEAD)
- dirty count

Returns `PanelData` with all results. Distributes to BranchPanel, StatusPanel, StatusBar.

### Remove `revwalk.count()` from StatusBar

Commit count is now shown as graph nodes count from garph (already computed).

### StatusBar gets dirty_count from StatusPanel

No separate `repo.statuses()` call.

## Modified Files

| File | Change |
|---|---|
| `src/branch.rs` | `reload()` accepts `PanelData` result from workspace |
| `src/status_panel.rs` | `reload()` accepts `PanelData` result from workspace |
| `src/status_bar.rs` | Remove `revwalk.count()`, get dirty_count from workspace |
| `src/workspace.rs` | Single `spawn_panel_reload()` bg task, distributes results |
| `src/garph.rs` | Expose `nodes.len()` as `pub fn node_count` |

## Status: ✅ DONE
