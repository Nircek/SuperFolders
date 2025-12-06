# SuperFolders

SuperFolders is a terminal-based application designed to bring order to chaotic file systems. It introduces the concept of **Superfolders**—logical groupings that allow you to collapse complex directory structures into single, manageable atomic units.

## Key Concepts

### Superfolders
A **Superfolder** is a folder that "swallows" its content in the view. It aggregates statistics (file count, modification dates) of everything inside it, presenting a clean "Frontier" of your file tree.

There are two types of Superfolders:
1.  **Atomic Superfolders (System Wrapper)**: Automatically detected folders that contain **System Folders** (e.g. `.git`, `node_modules`, `.venv`).
    -   *System Folder*: The technical folder identifying the project type (e.g. `.git`).
    -   *Atomic Superfolder*: The parent folder containing the System Folder. This parent is treated as an atomic unit because it represents a complete project/repo.
2.  **User Superfolders**: Any folder you choose to collapse manually.

### The "Tree Frontier"
Instead of a traditional indented tree view, Superfolder TUI shows a flat list representing the current "Frontier" of your file system. This is the highest level of detail you have chosen to see. Deeper details are hidden behind Superfolders.

## Controls

| Key | Action | Description |
| :--- | :--- | :--- |
| **Up / Down** | Navigation | Move selection up or down. |
| **Left Arrow** | **Collapse** | Collapses the **parent** of the current selection into a User Superfolder. |
| **Right Arrow** | **Expand** | Expands the selected User Superfolder, revealing its contents. (Does not work on Atomic Superfolders). |
| **Q / Esc** | Quit | Exits the application. |

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
