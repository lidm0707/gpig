# Plan 08 — Side-by-Side Diff Viewer

## Goal

Transform the diff view from plain text to an accordion-style file list with inline side-by-side colored diff.

## Requirements

1. Show all changed file names in a list
2. Files are collapsed by default — just show name
3. Click a file name to expand/collapse its diff inline (accordion)
4. Diff lines are color-coded (green = added, red = removed, gray = context)
5. Side-by-side layout: left = old content, right = new content, line-by-line

## New File

| File | Action | Notes |
|---|---|---|
| `src/diff_viewer.rs` | CREATED | Parser + renderer for side-by-side diff |

### `diff_viewer.rs`

- `DiffLineKind` enum: Context, Added, Removed
- `DiffLine` struct: kind, content, line_no
- `SideBySideRow` enum: Hunk(header) or Line(left, right)
- `parse_diff(raw) -> Vec<SideBySideRow>` — parse unified diff into paired rows
- `render_side_by_side(rows) -> AnyElement` — render two-column colored diff
- `is_binary_or_error(raw) -> bool` — detect non-parseable content

## Modified Files

| File | Action | Notes |
|---|---|---|
| `src/lib.rs` | MODIFIED | Add `pub mod diff_viewer` |
| `src/workspace.rs` | MODIFIED | Accordion file list + inline side-by-side diff |

### `workspace.rs` changes

- Rename `selected_file` → `expanded_file`
- New `on_file_toggled` — click to expand/collapse (replaces `on_file_selected` + `on_back_to_file_list`)
- New `render_file_panel` — unified accordion (replaces `render_file_list` + `render_file_diff`)
- Remove old `render_file_list` and `render_file_diff`

## Status: ✅ DONE
