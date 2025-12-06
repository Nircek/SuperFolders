# TODO

- **ui.rs**: Implement `TableState` scrolling properly inside `App` struct instead of recreating it in `draw`. This allows persisting scroll state if we implement advanced navigation.
- **ui.rs**: Add visual indicators or icons to the path column for better UX.
- **app.rs**: Add selection persistence when refreshing view (mostly handled, but can be improved).
- **config**: Move system folder names (`.git`, `node_modules`, etc.) to a configuration file instead of hardcoding.
- **ux**: Add progress bar when firstly recursively walking through path (with entries count).
- **ui**: Add in-app help (overlay or separate view) explaining terminology and controls.
- **feature**: Export the current view to CSV with the same columns as the TUI table.
