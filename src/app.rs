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
    /// Selection state
    #[allow(dead_code)] // (reserved for future use)
    pub is_selected: bool,
}

impl App {
    /// Initialize the application with the given root path
    pub fn new(root_path: PathBuf) -> Self {
        Self::new_with_progress(root_path, None)
    }

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
        let relative_path = entry
            .path
            .strip_prefix(root)
            .unwrap_or(&entry.path)
            .to_string_lossy()
            .to_string();
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

        let count_str = match entry.kind {
            EntryKind::File => "-".to_string(),
            _ => entry.stats.count.to_string(),
        };

        let is_superfolder = matches!(entry.kind, EntryKind::Superfolder { .. });

        list.push(ViewItem {
            path: relative_path,
            full_path: entry.path.clone(),
            min_date,
            max_date,
            count_str,
            is_superfolder,
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
        let item = &self.view_items[self.selected_index];
        let path = item.full_path.clone();

        let parent = match path.parent() {
            Some(p) => p,
            None => return Ok(()),
        };

        if parent == self.root_path && path == self.root_path {
            return Ok(());
        }

        let target_path = parent.to_path_buf();

        // 1. IO Action
        let marker = target_path.join(".superfolder");
        if !marker.exists() {
            fs::File::create(&marker)?;
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
    }

    /// Export the current view to CSV file
    pub fn export_to_csv(&self) -> io::Result<()> {
        use std::io::Write;

        let filename = format!(
            "superfolders_export_{}.csv",
            chrono::Local::now().format("%Y%m%d_%H%M%S")
        );

        let mut file = fs::File::create(&filename)?;

        // Write header
        writeln!(file, "Min Date,Max Date,Count,Path")?;

        // Write data rows
        for item in &self.view_items {
            writeln!(
                file,
                "\"{}\",\"{}\",\"{}\",\"{}\"",
                item.min_date, item.max_date, item.count_str, item.path
            )?;
        }

        Ok(())
    }
}
