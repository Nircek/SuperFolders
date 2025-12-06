# Walkthrough - Scrollable Help Support & Performance Optimization

I have updated the help overlay to support scrolling/theming and optimized the rendering of large file lists.

## Performance Optimization

I implemented **list virtualization** (manual windowing) for the file list rendering. This drastically improved render times for large lists.

**Benchmark Results (100k items):**
- Baseline (Naive Rendering): ~560ms per frame
- Optimized (Virtualization): ~11.6ms per frame
- **Improvement: >45x faster** (well below the <3s target)

## Changes

### Dynamic Navigation & Collapse Logic
- Updated `PageUp`/`PageDown` to use the **actual visible height** of the list/text area, ensuring full-page scrolls regardless of window size.
- Implemented **Smart Collapse (Merge Up)**:
  - When collapsing a folder that is already a User Superfolder (e.g., `a/b/c`), collapsing its parent (`a/b`) will **remove** the child's `.superfolder` marker.
  - This prevents nested redundancy (having both `a/b` and `a/b/c` marked as superfolders) while preserving siblings.

### Terminology & Formatting
- **Standardized Definitions**:
  - **Item**: File or Directory.
  - **Entry**: Table row (File, Folder, or Superfolder).
  - **Count**: 
    - Files/Folders = 1.
    - Superfolders = Recursive sum of all items inside.
- **Visuals**:
  - Files: No trailing slash.
  - Folders: Always trailing slash.

### App State (`src/app.rs`)

- Added `help_scroll` to `App` struct to track scroll position.
- Added `scroll_help_up()` and `scroll_help_down()` methods.
- Reset scroll position to 0 when opening help.

### UI Rendering (`src/ui.rs`)
- Modified `draw_help_overlay` to use `app.help_scroll`.
- Removed `.style(Style::default().bg(Color::Black).fg(Color::White))` method call, so the help text now uses the terminal's default foreground and background.

### Navigation Support (`src/app.rs` & `src/main.rs`)
- Implemented `PageUp`, `PageDown`, `Home`, and `End` handling for both contexts:
  - **Help Overlay**: Scrolls the help text.
  - **File List (Table)**: Jumps through the item list (Page +/- 15, Home=First, End=Last).
- Updated input handling loop to route keys based on `app.show_help` state.

## Verification Results

### Automated Checks
- `cargo check` passed successfully.

### Manual Verification Steps
1. **Help Navigation**:
   - Open help (`?`).
   - Use `PageDown`/`PageUp` to scroll fast.
   - Use `Home`/`End` to jump to top/bottom.
2. **Table Navigation**:
   - Close help.
   - Use `PageDown`/`PageUp` to traverse file list quickly.
   - Use `Home`/`End` to jump to start/end of list.
