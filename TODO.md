# TODO

## Completed ✅

- ~~**ui.rs**: Implement `TableState` scrolling properly inside `App` struct~~ - Done
- ~~**ui.rs**: Add visual indicators or icons to the path column~~ - Done (📦 icon)
- ~~**app.rs**: Add selection persistence when refreshing view~~ - Done
- ~~**config**: Move system folder names to configuration~~ - Done (TOML/env var support)
- ~~**ux**: Add progress bar when firstly recursively walking through path~~ - Done (with items/s)
- ~~**ui**: Add in-app help~~ - Done (scrollable overlay)
- ~~**feature**: Export CSV~~ - Done (using csv crate)
- ~~**ui**: Add status bar~~ - Done (shows total entries and messages)

## New Ideas & Improvements

- **feature**: Add filtering/search functionality (e.g., press `/` to search by path/name)
- **feature**: Add sorting options (by date, size, name, count)
- **ux**: Add keyboard shortcuts for jumping (e.g., `g` for top, `G` for bottom)
- **feature**: Add bookmarks/favorites for frequently accessed directories
- **feature**: Add "open in file manager" or "open terminal here" commands
- **config**: Allow custom colors/themes in config file
- **feature**: Add "refresh" key to rescan directory without restarting
- **performance**: Cache directory scans for faster repeated access
- **feature**: Add breadcrumb navigation showing current path hierarchy
- **ux**: Add confirmation dialog before collapsing/expanding large directories
