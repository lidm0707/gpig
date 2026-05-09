# Plan 04 — Repo Dropdown Picker

## Goal
Replace repo path text input with a **dropdown list** of discovered Git repos.
On app start, scan for directories containing `.git` and show them as selectable items.

## Architecture

```
┌──────────────────────────────────────────────────────────┐
│  PathBar                                                 │
│  [▼ repo dropdown (selected repo)]  [search] [Filter]    │
│  ┌─────────────────────┐                                 │
│  │  ~/git/project-a    │  ← dropdown overlay on click    │
│  │  ~/git/project-b    │                                 │
│  │  ~/other/repo       │                                 │
│  └─────────────────────┘                                 │
└──────────────────────────────────────────────────────────┘
```

## Implementation Steps

### Step 01 — repo_scanner module
**Status**: ✅ Done

### Step 02 — RepoPicker component
**Status**: ✅ Done

### Step 03 — Wire RepoPicker into PathBar
**Status**: ✅ Done

### Step 04 — Background scanning
**Status**: ✅ Done

### Step 05 — Update workspace wiring
**Status**: ✅ Done

## Status: ✅ Done
