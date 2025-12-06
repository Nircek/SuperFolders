use crate::config::Config;
use crate::scanner::{EntryKind, FsEntry, scan_directory, scan_directory_with_progress};
use ratatui::widgets::TableState;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicUsize;

/// Application state and logic
pub struct App {
    /// Path to the root directory being scanned
    pub root_path: PathBuf,
    /// Root node of the file system tree
    pub root: FsEntry,
    /// Flattened list of items currently visible in the UI
    pub view_items: Vec<ViewItem>,
    /// Index of the currently selected item in `view_items`
    pub selected_index: usize,
    /// Table state for proper scrolling
    pub table_state: TableState,
    /// Application configuration
    pub config: Config,
    /// Whether help overlay is shown
    pub show_help: bool,
    /// Status bar message with timestamp
    pub status_message: Option<(String, std::time::Instant)>,
    /// Current vertical scroll position of the help overlay
    pub help_scroll: u16,
    /// Height of the file list page (visible rows)
    pub page_height: u16,
    /// Height of the help text area
    pub help_height: u16,
}

/// A single item in the flattened view list
#[derive(Debug, Clone)]
pub struct ViewItem {
    /// Relative path to display
    pub path: String,
    /// Absolute path to the file/folder
    pub full_path: PathBuf,
    /// Formatted minimum modification date
    pub min_date: String,
    /// Formatted maximum modification date
    pub max_date: String,
    /// Formatted file count (or "-" for files)
    pub count_str: String,
    /// Whether this item is a Superfolder (User or Atomic)
    pub is_superfolder: bool,
    /// Whether this superfolder is atomic (system folder wrapper)
    pub is_atomic: bool,
    /// Selection state
    #[allow(dead_code)] // (reserved for future use)
    pub is_selected: bool,
}

impl App {
    /// Initialize the application with optional progress tracking
    pub fn new_with_progress(root_path: PathBuf, counter: Option<Arc<AtomicUsize>>) -> Self {
        let config = Config::new();
        let root = scan_directory_with_progress(&root_path, &config, counter);
        let mut app = Self {
            root_path: root_path.clone(),
            root,
            view_items: vec![],
            selected_index: 0,
            table_state: TableState::default(),
            config,
            show_help: false,
            status_message: None,
            help_scroll: 0,
            // Default heights, will be updated by UI on first draw
            page_height: 15,
            help_height: 15,
        };
        app.update_view();
        app
    }

    /// Full refresh of the file system scan
    pub fn refresh(&mut self) {
        self.root = scan_directory(&self.root_path, &self.config);
        self.update_view();
        if self.selected_index >= self.view_items.len() && !self.view_items.is_empty() {
            self.selected_index = self.view_items.len() - 1;
        }
    }

    /// Flattens the tree into a list based on current expansion state
    fn flatten_tree(entry: &FsEntry, root: &PathBuf, list: &mut Vec<ViewItem>) {
        match entry.kind {
            EntryKind::Directory => {
                if entry.children.is_empty() {
                    Self::add_view_item(entry, root, list);
                } else {
                    for child in &entry.children {
                        Self::flatten_tree(child, root, list);
                    }
                }
            }
            EntryKind::File | EntryKind::Superfolder { .. } => {
                Self::add_view_item(entry, root, list);
            }
        }
    }

    fn add_view_item(entry: &FsEntry, root: &PathBuf, list: &mut Vec<ViewItem>) {
        let raw_path = entry
            .path
            .strip_prefix(root)
            .unwrap_or(&entry.path)
            .to_string_lossy()
            .to_string();

        let count_str = match entry.kind {
            EntryKind::File => "1".to_string(),
            EntryKind::Directory => "1".to_string(), // Empty directory counts as 1 item
            EntryKind::Superfolder { .. } => entry.stats.count.to_string(),
        };

        // Append slash to directory paths if missing
        let is_dir_like = matches!(
            entry.kind,
            EntryKind::Directory | EntryKind::Superfolder { .. }
        );
        let path = if is_dir_like && !raw_path.ends_with(std::path::MAIN_SEPARATOR) {
            format!("{}{}", raw_path, std::path::MAIN_SEPARATOR)
        } else {
            raw_path
        };

        let min_date = entry
            .stats
            .min_date
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "-".to_string());
        let max_date = entry
            .stats
            .max_date
            .map(|d| d.format("%Y-%m-%d").to_string())
            .unwrap_or_else(|| "-".to_string());

        let (is_superfolder, is_atomic) = match entry.kind {
            EntryKind::Superfolder { is_atomic } => (true, is_atomic),
            _ => (false, false),
        };

        list.push(ViewItem {
            path,
            full_path: entry.path.clone(),
            min_date,
            max_date,
            count_str,
            is_superfolder,
            is_atomic,
            is_selected: false,
        });
    }

    /// Rebuilds the view items list
    pub fn update_view(&mut self) {
        let mut list = Vec::new();
        Self::flatten_tree(&self.root, &self.root_path, &mut list);
        self.view_items = list;
        self.table_state.select(Some(self.selected_index));
    }

    pub fn next(&mut self) {
        if !self.view_items.is_empty() {
            self.selected_index = (self.selected_index + 1) % self.view_items.len();
            self.table_state.select(Some(self.selected_index));
        }
    }

    pub fn previous(&mut self) {
        if !self.view_items.is_empty() {
            if self.selected_index > 0 {
                self.selected_index -= 1;
            } else {
                self.selected_index = self.view_items.len() - 1;
            }
            self.table_state.select(Some(self.selected_index));
        }
    }

    pub fn collapse_current(&mut self) -> io::Result<()> {
        if self.view_items.is_empty() {
            return Ok(());
        }
        let current_index = self.selected_index;
        // Clone data to avoid holding borrow on self.view_items
        let (path, is_superfolder, is_atomic) = {
            let item = &self.view_items[current_index];
            (item.full_path.clone(), item.is_superfolder, item.is_atomic)
        };

        let parent = match path.parent() {
            Some(p) => p,
            None => return Ok(()),
        };

        if parent == self.root_path && path == self.root_path {
            return Ok(());
        }

        let target_path = parent.to_path_buf();

        // 1. IO Action for new parent marker
        let marker = target_path.join(".superfolder");
        if !marker.exists() {
            fs::File::create(&marker)?;
            self.set_status(format!("Created: {}", marker.display()));
        }

        // 1.5. Merge Up Logic: Remove child marker if it was a user superfolder
        // Only trigger this if we are effectively collapsing the parent of a superfolder
        if is_superfolder && !is_atomic {
            let child_marker = path.join(".superfolder");
            if child_marker.exists() {
                // We remove the marker to "merge it up"
                if let Err(e) = fs::remove_file(&child_marker) {
                    // Log error or ignore if strictly necessary
                    self.set_status(format!("Error removing child marker: {}", e));
                }
            }

            // Note: We also need to update the in-memory state of this child node
            // so it doesn't appear as a superfolder when we rescan/refresh for the parent.
            if let Some(node) = Self::find_node_mut(&mut self.root, &path) {
                node.is_user_superfolder = false;
                // We don't necessarily need to change EntryKind back to Directory immediately
                // because the parent's collapse will hide it anyway.
                // However, correct state is better.
                if !matches!(node.kind, EntryKind::Superfolder { is_atomic: true }) {
                    node.kind = EntryKind::Directory;
                }
            }
        }

        // 2. In-Memory Update
        if let Some(node) = Self::find_node_mut(&mut self.root, &target_path) {
            node.is_user_superfolder = true;
            if !matches!(node.kind, EntryKind::Superfolder { .. }) {
                node.kind = EntryKind::Superfolder { is_atomic: false };
            }
        } else {
            self.refresh();
            return Ok(());
        }

        self.update_view();

        if let Some(pos) = self
            .view_items
            .iter()
            .position(|i| i.full_path == target_path)
        {
            self.selected_index = pos;
        }

        Ok(())
    }

    pub fn expand_current(&mut self) -> io::Result<()> {
        if self.view_items.is_empty() {
            return Ok(());
        }

        let item = &self.view_items[self.selected_index];
        if !item.is_superfolder {
            return Ok(());
        }
        let target_path = item.full_path.clone();

        // 1. IO Action
        let marker = target_path.join(".superfolder");
        let was_atomic = if let Some(node) = Self::find_node_mut(&mut self.root, &target_path) {
            matches!(node.kind, EntryKind::Superfolder { is_atomic: true })
        } else {
            false
        };

        if was_atomic && !marker.exists() {
            return Ok(());
        }

        if marker.exists() {
            fs::remove_file(marker)?;
        }

        // 2. In-Memory Update
        if let Some(node) = Self::find_node_mut(&mut self.root, &target_path) {
            node.is_user_superfolder = false;
            node.kind = EntryKind::Directory;

            if node.children.is_empty() {
                let fresh_node = scan_directory(&node.path, &self.config);
                node.children = fresh_node.children;
                node.stats = fresh_node.stats;
            }
        } else {
            self.refresh();
            return Ok(());
        }

        self.update_view();
        Ok(())
    }

    fn find_node_mut<'a>(
        node: &'a mut FsEntry,
        target: &std::path::Path,
    ) -> Option<&'a mut FsEntry> {
        if node.path == target {
            return Some(node);
        }
        if !target.starts_with(&node.path) {
            return None;
        }

        for child in &mut node.children {
            if let Some(found) = Self::find_node_mut(child, target) {
                return Some(found);
            }
        }
        None
    }

    /// Toggle help overlay
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        // Reset scroll when opening help
        if self.show_help {
            self.help_scroll = 0;
        }
    }

    /// Scroll help text down
    pub fn scroll_help_down(&mut self) {
        self.help_scroll = self.help_scroll.saturating_add(1);
    }

    /// Scroll help text up
    pub fn scroll_help_up(&mut self) {
        self.help_scroll = self.help_scroll.saturating_sub(1);
    }

    /// Scroll help text page down (dynamic)
    pub fn scroll_help_page_down(&mut self) {
        self.help_scroll = self.help_scroll.saturating_add(self.help_height);
    }

    /// Scroll help text page up (dynamic)
    pub fn scroll_help_page_up(&mut self) {
        self.help_scroll = self.help_scroll.saturating_sub(self.help_height);
    }

    /// Scroll help text to top
    pub fn scroll_help_home(&mut self) {
        self.help_scroll = 0;
    }

    /// Scroll help text to bottom (max u16)
    pub fn scroll_help_end(&mut self) {
        self.help_scroll = u16::MAX;
    }

    /// Move table selection page down (dynamic)
    pub fn table_page_down(&mut self) {
        if !self.view_items.is_empty() {
            let step = self.page_height as usize;
            let new_index = self.selected_index.saturating_add(step);
            self.selected_index = new_index.min(self.view_items.len() - 1);
            self.table_state.select(Some(self.selected_index));
        }
    }

    /// Move table selection page up (dynamic)
    pub fn table_page_up(&mut self) {
        if !self.view_items.is_empty() {
            let step = self.page_height as usize;
            self.selected_index = self.selected_index.saturating_sub(step);
            self.table_state.select(Some(self.selected_index));
        }
    }

    /// Move table selection to first item
    pub fn table_home(&mut self) {
        if !self.view_items.is_empty() {
            self.selected_index = 0;
            self.table_state.select(Some(self.selected_index));
        }
    }

    /// Move table selection to last item
    pub fn table_end(&mut self) {
        if !self.view_items.is_empty() {
            self.selected_index = self.view_items.len() - 1;
            self.table_state.select(Some(self.selected_index));
        }
    }

    /// Set a status message that will be shown for a few seconds
    pub fn set_status(&mut self, message: String) {
        self.status_message = Some((message, std::time::Instant::now()));
    }

    /// Get the current status message if it's still valid
    pub fn get_status(&self) -> Option<&str> {
        if let Some((msg, instant)) = &self.status_message
            && instant.elapsed().as_secs() < 3
        {
            return Some(msg);
        }
        None
    }

    /// Export the current view to CSV file
    pub fn export_to_csv(&mut self) -> io::Result<()> {
        let filename = format!(
            "superfolders_export_{}.csv",
            chrono::Local::now().format("%Y%m%d_%H%M%S")
        );

        let file = fs::File::create(&filename)?;
        let mut writer = csv::Writer::from_writer(file);

        // Write header
        writer.write_record(["Min Date", "Max Date", "Count", "Path"])?;

        // Write data rows
        for item in &self.view_items {
            writer.write_record([&item.min_date, &item.max_date, &item.count_str, &item.path])?;
        }

        writer.flush()?;
        self.set_status(format!("Exported to: {}", filename));
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::tempdir;

    #[test]
    fn test_merge_up_logic() {
        // Setup directory structure:
        // root/
        //   a/
        //     b/
        //       c/
        //         x.txt
        //       d/
        //         .superfolder
        //         y.txt

        let dir = tempdir().unwrap();
        let root = dir.path().to_path_buf();

        let path_c = root.join("a/b/c");
        fs::create_dir_all(&path_c).unwrap();
        File::create(path_c.join("x.txt")).unwrap();

        let path_d = root.join("a/b/d");
        fs::create_dir_all(&path_d).unwrap();
        File::create(path_d.join(".superfolder")).unwrap();
        File::create(path_d.join("y.txt")).unwrap();

        // Initialize App
        let mut app = App::new_with_progress(root.clone(), None);

        // Locate a/b/c/x.txt in view_items
        let x_path = path_c.join("x.txt");
        if let Some(pos) = app.view_items.iter().position(|i| i.full_path == x_path) {
            app.selected_index = pos;
        } else {
            // Debug print available items if not found
            for item in &app.view_items {
                println!("Item: {:?}", item.full_path);
            }
            panic!("Could not find x.txt in view items. Root: {:?}", root);
        }

        // 1. Collapse a/b/c -> a/b/c should become User Superfolder
        app.collapse_current().unwrap();

        // Verify a/b/c/.superfolder exists
        assert!(
            path_c.join(".superfolder").exists(),
            "a/b/c/.superfolder should exist"
        );

        // Verify selection is now on a/b/c
        let selected_item = &app.view_items[app.selected_index];
        assert_eq!(selected_item.full_path, path_c);
        assert!(selected_item.is_superfolder);

        // 2. Collapse a/b/c again (it's selected) -> should collapse parent a/b
        // AND should remove a/b/c/.superfolder (Merge Up)
        app.collapse_current().unwrap();

        let path_b = root.join("a/b");

        // Verify a/b/.superfolder exists
        assert!(
            path_b.join(".superfolder").exists(),
            "a/b/.superfolder should exist"
        );

        // Verify a/b/c/.superfolder is GONE (Merge Up Success)
        assert!(
            !path_c.join(".superfolder").exists(),
            "a/b/c/.superfolder should have been removed"
        );

        // Verify a/b/d/.superfolder still exists (Sibling Untouched)
        assert!(
            path_d.join(".superfolder").exists(),
            "a/b/d/.superfolder should still exist"
        );

        // Verify view has collapsed to a/b
        let selected_item_b = &app.view_items[app.selected_index];
        assert_eq!(selected_item_b.full_path, path_b);
    }
}

#[cfg(test)]
mod navigation_tests {
    use super::*;
    use crate::config::Config;
    use crate::scanner::{EntryKind, FsEntry, FsStats};
    use std::path::PathBuf;

    // Mock setup helper
    fn setup_mock_app(count: usize) -> App {
        let mut items = Vec::new();
        for i in 0..count {
            items.push(ViewItem {
                path: format!("item_{}", i),
                full_path: PathBuf::from(format!("/item_{}", i)),
                min_date: "-".to_string(),
                max_date: "-".to_string(),
                count_str: "1".to_string(),
                is_superfolder: false,
                is_atomic: false,
                is_selected: false,
            });
        }

        App {
            root_path: PathBuf::from("/"),
            root: FsEntry {
                path: PathBuf::from("/"),
                name: "root".to_string(),
                kind: EntryKind::Directory,
                stats: FsStats::new(),
                children: vec![],
                is_user_superfolder: false,
            },
            view_items: items,
            selected_index: 0,
            table_state: ratatui::widgets::TableState::default(),
            config: Config::new(),
            show_help: false,
            status_message: None,
            help_scroll: 0,
            page_height: 5, // Small page for testing
            help_height: 5,
        }
    }

    #[test]
    fn test_navigation_wrapping() {
        let mut app = setup_mock_app(3); // items: 0, 1, 2

        // Start at 0
        assert_eq!(app.selected_index, 0);

        // Previous -> Wrap to last (2)
        app.previous();
        assert_eq!(app.selected_index, 2);

        // Next -> Wrap to start (0)
        app.next();
        assert_eq!(app.selected_index, 0);

        // Normal navigation
        app.next();
        assert_eq!(app.selected_index, 1);
    }

    #[test]
    fn test_paging() {
        let mut app = setup_mock_app(10); // 0..9
        app.page_height = 3;

        // Start at 0
        // Page down -> 0 + 3 = 3
        app.table_page_down();
        assert_eq!(app.selected_index, 3);

        // Page down -> 3 + 3 = 6
        app.table_page_down();
        assert_eq!(app.selected_index, 6);

        // Page down -> 6 + 3 = 9 (max)
        app.table_page_down();
        assert_eq!(app.selected_index, 9);

        // Page down at max -> stays at 9
        app.table_page_down();
        assert_eq!(app.selected_index, 9);

        // Page up -> 9 - 3 = 6
        app.table_page_up();
        assert_eq!(app.selected_index, 6);
    }

    #[test]
    fn test_help_toggle_reset() {
        let mut app = setup_mock_app(1);
        app.help_scroll = 10;
        app.show_help = false;

        // Open help -> should reset scroll
        app.toggle_help();
        assert!(app.show_help);
        assert_eq!(app.help_scroll, 0);

        // Scroll down
        app.scroll_help_down();
        assert_eq!(app.help_scroll, 1);

        // Close help -> scroll preserved or irrelevant, but toggle back logic (state flip)
        app.toggle_help();
        assert!(!app.show_help);

        // Open again -> reset again
        app.toggle_help();
        assert!(app.show_help);
        assert_eq!(app.help_scroll, 0);
    }
}
