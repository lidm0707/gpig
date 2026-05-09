# Plan 02 — Fix UI Freeze on File Selection

## Problem
When selecting a file or commit, the OS shows "Not Responding" because
git2 operations run synchronously on the main GPUI thread, blocking
the render loop.

## Root Cause
1. `on_file_selected` → `garph.compute_file_diff()` — sync, reads blobs, iterates all deltas
2. `load_changed_files` → `garph.get_changed_files()` — sync, iterates all deltas
3. Both block the GPUI event loop → OS detects unresponsive window

## Fix
Use `cx.spawn()` to run git2 operations on a background thread,
show loading state in the meantime, update UI when result arrives.

## Affected Code Paths
- `workspace.rs`: `on_file_selected`, `load_changed_files`
- `garph.rs`: `compute_file_diff`, `get_changed_files` — need `Send`-safe wrappers

## Status: ✅ Done
