# Help Text Scrolling and Styling

## Goal Description
The user wants the help text in the application to be scrollable and to use the default terminal colors instead of a hardcoded white-on-black scheme.

## User Review Required
> [!NOTE]
> I will be removing the explicit `.bg(Color::Black).fg(Color::White)` style from the help popup. This means it will inherit the terminal's default foreground and background colors.

## Proposed Changes

### [src/app.rs](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/app.rs)

#### [MODIFY] [app.rs](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/app.rs)
- Add `help_scroll: u16` field to [App](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/app.rs#11-31) struct.
- Initialize `help_scroll` to `0` in `App::new_with_progress`.
- Add methods [scroll_help_down](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/app.rs#290-294) and [scroll_help_up](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/app.rs#295-299) to modify the scroll value. 
  - *Note:* Since `Paragraph` scrolling in `ratatui` handles out-of-bounds gracefully (mostly), simple increment/decrement is a good start. I might need to clamp it against the text height if I want to be precise, but for a simple help text, just letting it scroll is usually enough (or tracking lines). `Paragraph` scroll is (offset_x, offset_y).

### [src/ui.rs](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/ui.rs)

#### [MODIFY] [ui.rs](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/ui.rs)
- In [draw_help_overlay](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/ui.rs#87-141):
  - Remove `.style(Style::default().bg(Color::Black).fg(Color::White))`.
  - Update `.scroll((0, 0))` to `.scroll((app.help_scroll, 0))`.
  - Ensure the block borders still look okay (they use `Color::Cyan` currently, which is fine).

### [src/app.rs](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/app.rs)

#### [MODIFY] [app.rs](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/app.rs)
- Add methods for Help:
  - `scroll_help_page_up()`: decrease scroll by 15.
  - `scroll_help_page_down()`: increase scroll by 15.
  - `scroll_help_home()`: set scroll to 0.
  - `scroll_help_end()`: set scroll to `u16::MAX`.
- Add methods for Table (File List):
  - `table_page_up()`: decrease `selected_index` by 15 (clamped to 0).
  - `table_page_down()`: increase `selected_index` by 15 (clamped to len-1).
  - `table_home()`: set `selected_index` to 0.
  - `table_end()`: set `selected_index` to `view_items.len() - 1`.

### [src/main.rs](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/main.rs)

#### [MODIFY] [main.rs](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/main.rs)
- Map `PageUp`, `PageDown`, `Home`, `End` keys:
  - If `show_help` is true: Call corresponding help scroll methods.
  - If `show_help` is false: Call corresponding table navigation methods.

## Verification Plan

### Manual Verification
1.  **Help Navigation**:
    - Open help (`?`).
    - Test `PageDown`/`PageUp`/`Home`/`End`.
2.  **Table Navigation**:
    - Close help.
    - Test `PageDown` moves selection down by ~15 items.
    - Test `End` moves selection to the last item.
    - Test `Home` moves selection to the first item.
    - Test `PageUp` moves selection up.

# Performance Optimization: List Virtualization

## Goal Description
The current rendering implementation processes all items in the list every frame (formatting strings, allocating styles), which causes significant lag (~560ms) when the list contains 100k items. The goal is to reduce this to <50ms by only processing and rendering the items visible in the viewport.

## User Review Required
> [!NOTE]
> I will be implementing manual scroll handling in the UI layer. This means I will manually calculate the visual offset and feed only the visible slice to the `Table` widget. This avoids processing 100k items per frame.

## Proposed Changes

### [src/ui.rs](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/ui.rs)

#### [MODIFY] [ui.rs](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/ui.rs)
- Modify [draw](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/ui.rs#9-86) function:
  1. Calculate the available height for the table (`chunks[0].height`).
  2. Implement logic to ensure `app.selected_index` is visible:
     - Get current offset from `app.table_state.offset()`.
     - Adjust offset if selected index is out of bounds [offset, offset + height].
     - Update `app.table_state` with the new offset.
  3. Determine the start and end indices for the slice: `start = offset`, `end = min(len, start + height)`.
  4. Create `rows` iterator from `app.view_items[start..end]`.
  5. Adjust specific row styling logic to use the loop index + start offset.
  6. Render `Table` with the small slice.
  7. Pass a **temporary** `TableState` to `render_stateful_widget` with `selected(Some(app.selected_index - start))` and `offset(0)`.
     - *Reason*: The table widget only sees the slice, so logical index 0 in the table corresponds to `start` in our data.

### [src/app.rs](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/app.rs)

#### [MODIFY] [app.rs](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/app.rs)
- No changes needed to [App](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/app.rs#11-31) struct, we utilize existing `table_state`.

## Verification Plan

### Automated Tests
1. Run `cargo test --bin SuperFolders ui::tests::benchmark_rendering_large_list -- --nocapture`
   - Expect render time to drop significantly (e.g., < 10ms).

### Manual Verification
1. Run app with a large folder (or mock).
2. Scroll down/up and ensure selection moves correctly and items appear/disappear at edges correctly.
3. Test PageUp/PageDown/Home/End.

# Dynamic Navigation and Collapse Logic

## Goal Description
1.  **Dynamic Scrolling**: Update `PageUp` and `PageDown` to scroll by the actual visible page height instead of a fixed amount.
2.  **Smart Collapse**: Implement logic to remove the child's superfolder marker when collapsing a parent if the child was a user-created superfolder. This allows "merging up" the grouping.

## Proposed Changes

### [src/app.rs](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/app.rs)

#### [MODIFY] [app.rs](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/app.rs)
- Add `page_height: u16` and `help_height: u16` to [App](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/app.rs#11-31) struct.
- Initialize them to safe defaults (e.g., 15) in [new_with_progress](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/app.rs#55-73).
- Update `scroll_help_page_down/up` to use `self.help_height`.
- Update `table_page_down/up` to use `self.page_height`.
- Modify [collapse_current](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/app.rs#167-215):
  - Check if the current selected item is a *User Superfolder*.
  - If yes, and we successfully collapse the parent:
    - Remove the `.superfolder` marker from the *current item's* path.
    - (Preserve markers for siblings, as requested).

### [src/ui.rs](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/ui.rs)

#### [MODIFY] [ui.rs](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/ui.rs)
- In [draw](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/ui.rs#9-114):
  - Calculate `available_height` for the table.
  - Update `app.page_height = available_height as u16`.
- In [draw_help_overlay](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/ui.rs#115-169):
  - Calculate height of the text area.
  - Update `app.help_height = height as u16`.

## Verification Plan

### Automated Tests
- Create `tests/collapse_logic.rs`:
  - Reproduce the scenario: `a/b/c/x.txt`, `a/b/d/.superfolder`.
  - Step 1: Collapse `a/b/c`. Verify `a/b/c/.superfolder` exists.
  - Step 2: Collapse `a/b`. Verify `a/b/c/.superfolder` is GONE.
  - Step 3: Verify `a/b/d/.superfolder` still exists.

### Manual Verification
- Run app.
- Resize terminal to be very short or very tall.
- Test `PageDown` in help and table. Ensure it scrolls exactly one page.

# Standardization: Items & Entries

## Goal Description
Align application logic with user-defined terms:
- **Item**: File or Directory.
- **Entry**: Table row (File, Empty Folder, or Superfolder).
- **Counts**:
  - File Entry: 1.
  - Empty Folder Entry: 1.
  - Superfolder Entry: Recursive count of all contained items (files + directories), ignoring `.superfolder`.
- **Formatting**:
  - File path: No trailing `/`.
  - Folder path (Empty or Superfolder): Must have trailing `/`.

## Proposed Changes

### `src/scanner.rs`
#### [MODIFY] [scanner.rs](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/scanner.rs)
- Update `scan_directory` and `scan_recursive_for_stats`:
  - When processing children, increment `stats.count` by `1` for **every** child (File or Directory), unless it is `.superfolder` (ignored).
  - Also merge `child.stats` for directories.
  - This ensures `stats.count` represents the total recursive node count.

### `src/app.rs`
#### [MODIFY] [app.rs](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/app.rs)
- Update `add_view_item`:
  - For `EntryKind::File`: `count_str` = "1". Ensure no trailing `/`.
  - For `EntryKind::Directory` (Empty): `count_str` = "1". Ensure trailing `/`.
  - For `EntryKind::Superfolder`: `count_str` = `entry.stats.count`. Ensure trailing `/`.

### Documentation
- Update `draw_help_overlay` in `src/ui.rs`.
- Update `README.md`.

## Verification Plan

### Automated Tests
- Create `tests/test_definitions.rs`:
  - Structure:
    - `file.txt` (Count 1, No slash)
    - `empty_dir/` (Count 1, Slash)
    - `sf/` (Superfolder containing `dir/` and `file`)
      - `dir/` contains `inner_file`.
      - Total items in `sf`:
        - `dir` (1)
        - `file` (1)
        - `inner_file` (1 inside dir)
        - Total: 3.
  - Verify counts and path strings in `App::view_items`.

### Manual Verification
- Check UI for slashes and counts.
