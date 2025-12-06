# Superfolder TUI Implementation Plan

## Goal Description
Create a TUI application to intelligently group files and folders into "Superfolders". The app visualizes a "Tree Frontier" allowing users to collapse and expand folder structures to organize chaotic directories.

## Proposed Changes

### Project Structure
New Rust project `superfolder-tui`.

### Scan Logic (scanner.rs)
- **Struct `FsEntry`**: Path, Kind (File, Dir, Superfolder), Stats (dates, count), Children.
- **Function `scan_directory`**: DFS scan, respecting `.superfolder` markers and default atomic folders (git, node_modules).

### App State (app.rs)
- **Struct `App`**: Root entry, View list (flattened frontier), Selection state.
- **Actions**:
    - **Collapse (Left)**: Create `.superfolder` at current parent level, effectively grouping children.
    - **Expand (Right)**: Remove `.superfolder` file, revealing children.

### TUI (ui.rs)
- **Table View**: Columns: Min Date, Max Date, Count, Path.

### Persistence
- `.superfolder` file acts as the persistent marker for a collapsed state.

## Verification
- **Automated**: Unit tests for scanner aggregation.
- **Manual**: Run app on test directory, verify persistence of `.superfolder`.

## Optimization Plan
### Goal
Optimize `collapse` and `expand` actions to be near-instantaneous.

### Changes
1. **`app.rs`**: Add `find_node_mut` helper.
2. **`app.rs`**: Refactor `collapse_current` to update in-memory node state instead of full rescan.
3. **`app.rs`**: Refactor `expand_current` to update in-memory node state and perform partial scan if needed.

## Release Preparation
### Goal
Prepare the repository for public release on GitHub.

### Tasks
1. **Code Cleanup**: `src/app.rs`, `src/scanner.rs`, `src/ui.rs`, `src/main.rs`.
    - Add `///` documentation for pub structs and functions.
    - Remove developer comments/questions.
    - Extract TODOs to `TODO.md`.
2. **Documentation**: Create `README.md`.
    - Explain "Superfolder" (User-created group) vs "Atomic Superfolder" (System group like .git).
    - Explain Left/Right arrow interactions ("Collapse"/"Expand").
3. **CI/CD**: Create `.github/workflows/ci.yml`.
    - Triggers: `pull_request`, `push` to main.
    - Steps: Checkout, Cache, Rustfmt check, Build, Test.

## Refactoring & Polishing
### Goal
Address user feedback regarding naming, CI test stability, and future roadmap.

### Tasks
1.  **Renaming**: Change `superfolder-tui` to `SuperFolders` in `Cargo.toml`, `README.md`, code comments. Rename source directory if possible.
2.  **Terminology**:
    -   *System Folder*: `.git`, `node_modules`.
    -   *Atomic Superfolder*: The parent containing a System Folder.
    -   Update `README.md` and doc comments in `scanner.rs`/`app.rs`.
3.  **Test Fixes**:
    -   Add `tempfile` dev-dependency.
    -   Refactor `scanner.rs` tests to create a temporary directory structure instead of relying on `test_data`.
4.  **TODOs**: Add requested features (Config, Progress Bar, Help, CSV Export) to `TODO.md`.
