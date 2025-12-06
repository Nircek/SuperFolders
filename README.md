# SuperFolders

SuperFolders is a terminal-based application designed to bring order to chaotic file systems. It introduces the concept of **Superfolders**—logical groupings that allow you to collapse complex directory structures into single, manageable atomic units.

## Key Concepts

### Terminology
- **Item**: A file or a directory.
- **Entry**: A row in the table view (can be a File, an Empty Folder, or a Superfolder).
- **Count**: The number of items contained within an entry.
  - Files and Folders have a count of **1**.
  - Superfolders show the **total recursive count** of all files and folders inside them.

### Superfolders
A **Superfolder** is a folder that "swallows" its content in the view. It aggregates statistics (file count, modification dates) of everything inside it.
- **Atomic Superfolders**: Automatically detected project folders (parent of system folder).
- **User Superfolders**: Manually collapsed folders (parent of `.superfolder`).

### The "Tree Frontier"
The view shows a flat list of entries representing the current "Frontier".
- Files do not have a trailing slash (e.g., `file.txt`).
- Folders always have a trailing slash (e.g., `folder/`).

### The "Tree Frontier"
Instead of a traditional indented tree view, Superfolder TUI shows a flat list representing the current "Frontier" of your file system. This is the highest level of detail you have chosen to see. Deeper details are hidden behind Superfolders.

## Controls

| Key | Action | Description |
| :--- | :--- | :--- |
| **Up / Down** | Navigation | Move selection up or down. |
| **Left Arrow** | **Collapse** | Collapses the **parent** of the current selection into a User Superfolder. |
| **Right Arrow** | **Expand** | Expands the selected User Superfolder, revealing its contents. (Does not work on Atomic Superfolders). |
| **E** | **Export** | Exports the current view to a CSV file with timestamp. |
| **H / ?** | **Help** | Shows/hides the in-app help overlay with controls and terminology. |
| **Q / Esc** | Quit | Exits the application. |

## Features

- **Visual Indicators**: Superfolders are marked with 📦 icon and [SF] tag for easy identification
- **Persistent Scroll State**: Table state is maintained properly in the App struct for smooth navigation
- **CSV Export**: Export current view to timestamped CSV files (press 'E')
- **In-App Help**: Press 'H' or '?' to see controls and terminology
- **Progress Indicator**: Shows real-time progress during initial directory scan
- **Configurable System Folders**: System folder detection is configurable and extensible

## Installation & Usage

1.  **Build**:
    ```bash
    cargo build --release
    ```

2.  **Run**:
    ```bash
    ./target/release/SuperFolders <path-to-directory>
    ```
    If no path is provided, it defaults to the current directory.
    
3.  **Export Data**:
    ```bash
    # While running, press 'E' to export the current view
    # Creates: superfolders_export_YYYYMMDD_HHMMSS.csv
    ```

## Hackability & Customization

SuperFolders is designed to be easily hackable and customizable:

### Configuration System

The system folder detection is fully configurable. SuperFolders loads configuration in this priority order:

1. **Environment Variable** (highest priority)
   ```bash
   export SUPERFOLDERS_SYSTEM_FOLDERS=".git,node_modules,target,vendor"
   ```

2. **Config File next to binary**
   ```bash
   # Create superfolders.toml next to the executable
   system_folders = [".git", "node_modules", "target"]
   ```

3. **Config File in home directory**
   ```bash
   # Create ~/.superfolders.toml
   system_folders = [".git", "node_modules", "target"]
   ```

4. **Built-in defaults** (lowest priority)

Default system folders include:
- `.git` (Git repositories)
- `node_modules` (Node.js)
- `.venv`, `venv`, `__pycache__` (Python)
- `target` (Rust)
- `build`, `dist` (Build artifacts)
- `.idea`, `.vscode` (IDEs)

See `superfolders.toml.example` for a complete configuration file example.

### Progress Tracking

The scanner supports optional progress tracking via `AtomicUsize` counters:

```rust
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

let counter = Arc::new(AtomicUsize::new(0));
let root = scan_directory_with_progress(&path, &config, Some(counter.clone()));
```

### CSV Export Format

The CSV export follows a simple format:
```csv
Min Date,Max Date,Count,Path
"2024-01-01","2024-01-15","42","path/to/folder"
```

### Modular Architecture

The codebase is organized into clear modules:
- `src/main.rs` - Entry point and terminal setup
- `src/app.rs` - Application state and logic
- `src/scanner.rs` - File system scanning with progress tracking
- `src/ui.rs` - TUI rendering with ratatui
- `src/config.rs` - Configuration management

### Extending the UI

The UI uses `ratatui` and can be easily customized:
- Modify colors in `src/ui.rs`
- Add new views or overlays (see `draw_help_overlay` as example)
- Customize table columns and formatting

### Adding New Commands

To add new keyboard shortcuts:
1. Add key handling in `src/main.rs` in the `run_app` function
2. Implement corresponding method in `src/app.rs`
3. Update help overlay in `src/ui.rs` (optional)
4. Update README Controls section
