# Plan 07 — Fix Text Input, Menu Bar, Auto-Suggest

## Problems

1. **Text input broken** — suggest dropdown overlaps the input, blocking click/type/select.
   - `suggest_open` auto-sets to `true` when matches exist, even while typing
   - Dropdown positioned inside the input's parent div, intercepts mouse events
   - Click on input area hits the suggest dropdown instead

2. **Menu bar doesn't work** — click on "File" button propagates to workspace global handler
   - Workspace `on_mouse_down` closes the menu dropdown immediately after opening
   - Need `stop_propagation` on menu button click

3. **Auto-suggest needs restructure** — decouple dropdown from input area
   - Move suggest dropdown to workspace-level overlay (like repo picker dropdown)
   - Only open on explicit focus or when user is actively typing
   - Close on Escape, click outside, or selection

## Fix Steps

### 07.1 Fix menu bar — stop propagation on menu button click
- Add `.stop_propagation()` to menu button's `on_click` handler in `menu.rs`

### 07.2 Fix suggest dropdown — move to workspace overlay
- Remove inline suggest dropdown from `path_bar.rs` render
- Expose `suggest_items()` and `suggest_open()` from PathBar
- Render suggest dropdown at workspace level (same pattern as repo picker dropdown)
- Add `stop_propagation` on suggest items

### 07.3 Fix suggest open logic
- Only open suggest when search input has focus AND has text
- Close on: selection, click outside, Escape, clear
- Don't auto-close while typing

### 07.4 Clean up dead code warning
- `select_suggestion` method is unused — wire it up or remove dead code

### 07.5 cargo check && cargo clippy

## Status: ✅ Done
