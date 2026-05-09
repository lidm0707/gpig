# Plan 01 — Small Git Management App

## Current State Analysis

### What Exists (`src/`)

| File | Purpose | Status |
|---|---|---|
| `main.rs` | App bootstrap, window, keybindings, file dialog | ✅ working |
| `lib.rs` | Module declarations | ✅ working |
| `actions.rs` | `Quit`, `OpenFile` actions | ✅ working |
| `workspace.rs` | Main layout: dock + content pane, commit selection, file list, diff view | ✅ working |
| `garph.rs` | Git graph: revwalk, commit nodes, edges, diff computation, changed files | ✅ working |
| `commit.rs` | `CommitNode` data struct | ✅ working |
| `lane.rs` | `LaneManager` for graph lane assignment | ✅ working |
| `edge.rs` | `Edge` + `EdgeManager` for graph edges | ✅ working |
| `history_oid.rs` | `HistoryOidManager` for tracking parent positions | ✅ working |
| `color.rs` | `ColorManager` for lane colors | ✅ working |
| `diff_pane.rs` | Standalone diff pane (unused in current flow) | ⚠️ not wired |
| `menu.rs` | MenuBar with File dropdown | ✅ working |
| `title.rs` | TitleBar with window controls | ✅ working |

### What's Missing for a "Git Management" App

| Feature | Priority | Notes |
|---|---|---|
| **Branch list & switch** | P0 | Show branches, checkout — no branch UI at all |
| **Status panel (working dir)** | P0 | Show modified/untracked files — no `repo.statuses()` usage |
| **Stage & Commit** | P1 | Index add, write tree, commit — no write operations |
| **Branch create/delete** | P1 | Only read branches — no create/delete |
| **Stash** | P2 | Save/pop/list |
| **Remote fetch/push** | P2 | Network operations |
| **Blame view** | P2 | Line-level authorship |
| **Diff highlighting** | P1 | Color +/- lines in diff pane |
| **Keyboard navigation** | P1 | Navigate commits, files with keyboard |
| **Search/filter** | P2 | Filter commits by message/author |

### Known Issues

1. **User(-7) error** on certain diffs — non-fatal, logged in milestone
2. **`diff_pane.rs` unused** — workspace renders diff inline instead
3. **`eprintln!` debug logs** left in `garph.rs` (`get_changed_files`)
4. **Graph re-renders on every frame** — `recompute()` called in `render()`
5. **No error UI** — errors only in stderr, user sees nothing

---

## Architecture Plan

```
┌─────────────────────────────────────────────────────┐
│  TitleBar                                           │
├──────┬──────────────────────────────────────────────┤
│ Menu │                                              │
├──────┴──────┬────────────────────┬─────────────────┤
│             │                    │                 │
│  Sidebar    │   Main Content     │  Inspector      │
│             │                    │  (optional)     │
│ ┌─────────┐ │  ┌──────────────┐  │                 │
│ │ Branch  │ │  │  Commit      │  │  Diff detail    │
│ │ List    │ │  │  Graph       │  │  or blame       │
│ ├─────────┤ │  │  (existing)  │  │                 │
│ │ Status  │ │  │              │  │                 │
│ │ Panel   │ │  └──────────────┘  │                 │
│ ├─────────┤ │  ┌──────────────┐  │                 │
│ │ Stash   │ │  │ Changed Files│  │                 │
│ │ List    │ │  │ + Diff View  │  │                 │
│ └─────────┘ │  └──────────────┘  │                 │
│             │                    │                 │
├─────────────┴────────────────────┴─────────────────┤
│  Status Bar (repo path, branch, commit count)      │
└─────────────────────────────────────────────────────┘
```

---

## Implementation Steps

### Phase 1 — Core Read Features (P0)

#### Step 1.1: Clean up existing code
- **Files**: `garph.rs`, `diff_pane.rs`
- Remove `eprintln!` debug logs from `get_changed_files`
- Remove or wire `diff_pane.rs` (currently dead code)
- Move `recompute()` out of `render()` — call on repo update only
- Add error state to workspace UI (show errors to user)

#### Step 1.2: Branch panel (`src/branch.rs`)
- **New file**: `branch.rs`
- List local branches with `repo.branches()`
- Show current branch (bold/highlight)
- Click to checkout branch
- Update graph on branch switch
- Add `BranchSelected` event

#### Step 1.3: Status panel (`src/status_panel.rs`)
- **New file**: `status_panel.rs`
- Call `repo.statuses()` for working directory changes
- Show staged / unstaged / untracked groups
- Color-coded status icons (A/M/D/?)
- Click file to show working diff (`diff_index_to_workdir`)
- Refresh on focus / manual trigger

#### Step 1.4: Status bar (`src/status_bar.rs`)
- **New file**: `status_bar.rs`
- Show: repo name, current branch, commit count, dirty count
- Update on repo change

### Phase 2 — Write Operations (P1)

#### Step 2.1: Stage & commit (`src/staging.rs`)
- **New file**: `staging.rs`
- Toggle stage/unstage individual files
- Stage all / unstage all
- Commit message input
- Call `index.add_path()` + `repo.commit()`
- Refresh graph after commit

#### Step 2.2: Branch create/delete
- Add to branch panel
- Create branch from HEAD or selected commit
- Delete branch (with confirmation)
- Rename branch

#### Step 2.3: Diff syntax highlighting
- Parse diff lines (+/-/space)
- Color: green for additions, red for deletions, gray for context
- Line numbers
- Use `gpui::span()` or similar for inline coloring

#### Step 2.4: Keyboard navigation
- `j/k` or arrow keys: move between commits
- `Enter`: select commit / open file diff
- `Esc`: back to list
- `b`: branch panel toggle
- `/`: search filter

### Phase 3 — Advanced Features (P2)

#### Step 3.1: Stash management
- Save stash with message
- List stashes
- Pop / apply / drop
- Show stash diff

#### Step 3.2: Remote operations
- List remotes
- Fetch (with credentials callback)
- Push current branch
- Pull (fetch + merge)

#### Step 3.3: Blame view
- Line-by-line blame for selected file
- Show commit SHA + author per line
- Click to navigate to commit

#### Step 3.4: Search & filter
- Filter commits by message text
- Filter by author
- Filter by date range
- Filter by path (file history)

---

## File Structure After

```
src/
├── main.rs              # bootstrap (existing)
├── lib.rs               # module declarations (existing)
├── actions.rs           # global actions (existing, extend)
├── workspace.rs         # main layout (existing, modify)
├── garph.rs             # commit graph (existing, modify)
├── commit.rs            # CommitNode (existing)
├── lane.rs              # LaneManager (existing)
├── edge.rs              # EdgeManager (existing)
├── history_oid.rs       # HistoryOidManager (existing)
├── color.rs             # ColorManager (existing)
├── menu.rs              # MenuBar (existing, extend)
├── title.rs             # TitleBar (existing)
├── diff_pane.rs         # DiffPane (existing, remove or rewire)
├── branch.rs            # NEW — branch list & switch
├── status_panel.rs      # NEW — working directory status
├── status_bar.rs        # NEW — bottom status bar
├── staging.rs           # NEW — stage/commit UI
└── diff_highlight.rs    # NEW — colored diff rendering
```

---

## Module Dependency Graph

```
main.rs
  └─ workspace.rs
       ├─ garph.rs ─── commit.rs, lane.rs, edge.rs, history_oid.rs, color.rs
       ├─ branch.rs ─── (git2)
       ├─ status_panel.rs ─── (git2)
       ├─ status_bar.rs ─── (git2)
       ├─ staging.rs ─── (git2)
       ├─ diff_highlight.rs
       ├─ menu.rs
       └─ title.rs
```

---

## Key Design Decisions

1. **Shared Repository**: Keep `Rc<RefCell<Option<Repository>>>` pattern — `Repository` is `!Send + !Sync`, single-thread access is fine for GPUI
2. **Event-driven**: Use `EventEmitter` pattern (already established) for communication between panels
3. **Lazy loading**: Only compute diffs when user selects a commit/file
4. **Performance limits**: Keep the `MAX_TOTAL_LINES`, `MAX_FILES_TO_SHOW` constants — prevent memory issues
5. **No external state**: All git state comes from `git2` directly, no separate database

---

## Execution Order (recommended)

```
01  clean up garph.rs (remove eprintln, fix recompute)          ✅ DONE
02  create branch.rs — list + checkout
03  create status_panel.rs — working dir status
04  create status_bar.rs — repo info bar
05  wire branch + status into workspace.rs layout
06  create diff_highlight.rs — colored diff lines
07  create staging.rs — stage & commit
08  extend branch.rs — create / delete
09  add keyboard navigation
10  add stash support
11  add remote fetch/push
12  add blame view
13  add search/filter
```

---

## Status: 📋 Plan Ready
