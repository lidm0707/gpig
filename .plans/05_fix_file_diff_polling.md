# Plan 05 — Fix File Click Diff Bug

## Problem

When clicking a file in the changed files list to view its diff, the diff never appears. The "Loading diff..." spinner stays forever.

## Root Cause

`poll_pending_results()` uses `try_recv()` on the mpsc channels but the `let Ok(...)` guard pattern short-circuits when the channel is empty. No re-render is scheduled, so:

1. `on_file_selected()` spawns background thread, sets `loading_diff = true`, calls `cx.notify()`
2. Next render: `poll_pending_results()` called, `try_recv()` fails (thread not done), **no `cx.notify()` called**
3. No further renders happen → diff result is never picked up

Same bug exists for `pending_files_rx` (changed files list).

## Fix

Split the `if let ... && let Ok(...)` into nested if/else. When the channel exists but is empty, call `cx.notify()` to schedule another render on the next frame — creating a polling loop that stops once results arrive.

## Files

| File | Action | Notes |
|---|---|---|
| `src/workspace.rs` | MODIFIED | `poll_pending_results()` — add `cx.notify()` on empty channel |

## Status: ✅ DONE
