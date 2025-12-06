# Tasks

- [x] Locate help text rendering logic <!-- id: 0 -->
- [x] Create implementation plan <!-- id: 1 -->
- [x] Update [App](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/app.rs#11-31) state to support help scrolling <!-- id: 2 -->
- [x] Implement scrolling logic in input handler <!-- id: 3 -->
- [x] Refactor help rendering to be scrollable and use default colors <!-- id: 4 -->

- [x] Verify changes <!-- id: 5 -->
- [x] Add PageUp/PageDown/Home/End support to [App](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/app.rs#11-31) (Help & Table) <!-- id: 6 -->
- [x] Map new keys in input handler (Context Aware) <!-- id: 7 -->
- [x] Verify extended navigation <!-- id: 8 -->

- [x] Create benchmark test for rendering large lists <!-- id: 9 -->
- [x] Measure current baseline performance <!-- id: 10 -->
- [x] Create implementation plan for rendering optimization <!-- id: 11 -->
- [x] Implement list virtualization in `ui::draw` <!-- id: 12 -->

- [x] Verify performance improvement <!-- id: 13 -->

- [x] Add `page_height` and `help_height` to [App](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/app.rs#11-35) struct <!-- id: 14 -->
- [/] Update `ui::draw` to set heights in [App](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/app.rs#11-35) <!-- id: 15 -->
- [x] Update [App](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/app.rs#11-35) navigation methods to use dynamic heights <!-- id: 16 -->
- [x] Implement "Merge Up" logic in [collapse_current](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/app.rs#184-262) <!-- id: 17 -->


- [ ] Create integration test for collapse logic <!-- id: 18 -->
- [x] Verify performance improvement <!-- id: 13 -->
- [x] Update [FsStats](file:///Users/marcin.zepp/repos/superfolder-tui/SuperFolders/src/scanner.rs#23-31) to count all files and directories recursively (ignoring .superfolder) <!-- id: 20 -->
- [x] Update `App::add_view_item` logic for path suffixes and count strings <!-- id: 21 -->

- [x] Update README and Help text with new definitions <!-- id: 22 -->
- [x] Add/Update tests for counts and formatting <!-- id: 23 -->

- [x] Update `TODO.md` with identified code smells <!-- id: 24 -->

- [x] Add unit tests for `App` navigation (next/prev wrapping) <!-- id: 25 -->
- [x] Add unit tests for `App` help toggle state <!-- id: 26 -->
- [x] Add unit tests for `App` selection boundaries <!-- id: 27 -->
- [x] Verify all tests pass <!-- id: 28 -->

