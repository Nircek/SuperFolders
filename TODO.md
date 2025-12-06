# TODO

- **feature**: Add filtering/search functionality (press `/` to search by regex in path)
- **ux**: Add keyboard shortcuts for jumping (e.g., `g` for top, `G` for bottom)
- **feature**: Add bookmarks/favorites for frequently accessed directories
- **feature**: Add "reveal" (open file manager here) or "open terminal here" commands
- **config**: Allow custom colors/themes in config file
- **feature**: Add "refresh" key to rescan directory without restarting
- **performance**: Cache directory scans for faster repeated access
- **ux**: Proper error handling (no access, invalid config, etc.)

- **future**: Add support for changing the system directiories on the fly (with export to config)

## Refactoring / Technical Debt
- **refactor**: Separate IO operations from state logic in `App` (e.g. `collapse_current`, `expand_current`). Create a `FileSystemService` trait.
- **refactor**: Optimize `flatten_tree` to avoid recursion on every view update; consider caching or iterative approach.
- **refactor**: Split monolithic `ui::draw` function into smaller components (`draw_table`, `draw_status`, `draw_help`).
- **refactor**: Reduce string cloning in `FsEntry` and `ViewItem` (consider `Cow` or `Rc<str>`).
- **refactor**: Extract event handling logic from `main.rs` into a dedicated `EventHandler`.
- **fix**: Replace raw `unwrap`/`panic` calls with proper error propagation in `scanner.rs` and `app.rs`.
- **config**: Replace magic numbers (e.g. layout constraints, default sizes) with named constants.
