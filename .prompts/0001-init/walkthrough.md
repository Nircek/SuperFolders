# Superfolder TUI Walkthrough

The Superfolder TUI application has been implemented. It allows you to visualize and organize your file system by collapsing folders into "Superfolders".

## Features
- **Tree Frontier Visualization**: Shows a flattened view of "leaves" (files or collapsed folders).
- **Atomic Superfolders**: Automatically detects `.git`, `node_modules`, `.venv` as atomic units.
- **User Superfolders**: Collapse any folder into a Superfolder using the Left Arrow.
- **Persistence**: Uses `.superfolder` marker files to remember collapsed state across sessions.

## How to Run

1. **Build and Run**:
   ```bash
   cargo run -- path/to/scan
   ```
   Example with test data:
   ```bash
   ./setup_test_env.sh
   cargo run -- test_data
   ```

## Controls
- **Up / Down**: Navigate the list.
- **Left Arrow**: Collapse the parent of the selected item into a Superfolder.
- **Right Arrow**: Expand a User Superfolder (cannot expand Atomic folders like `.git`).
- **Q / Esc**: Quit.

## Optimization
The application features optimized collapse/expand actions:
- **Collapsing**: Updates the in-memory tree immediately without rescanning the disk. O(1) disk operations.
- **Expanding**: Updates in-memory tree and lazily scans only the expanded subtree if needed.

## Release Preparation
The following files have been prepared for public release:
- `README.md`: User-friendly documentation.
- `TODO.md`: Extracted developer notes.
- `.github/workflows/ci.yml`: CI pipeline checks `cargo fmt`, `cargo build`, and `cargo test`.
- Codebase: Cleaned of dev comments and added Rustdoc.

## Verification
Unit tests have verify the core logic:
- `test_scanner_atomic`: Confirms that folders containing `.git` are treated as Atomic Superfolders.
- `test_scanner_user_superfolder`: Confirms that folders with `.superfolder` markers are treated as User Superfolders.
- `test_structure`: Confirms correct parsing of files and directories.

Run tests with:
```bash
cargo test
```
