use crate::config::Config;
use chrono::{DateTime, Local};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Type of a file system entry
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EntryKind {
    /// Regular file
    File,
    /// Regular directory
    Directory,
    /// A "Superfolder" that aggregates contents.
    /// `is_atomic` is true for **Atomic Superfolders** (wrappers containing **System Folders** like `.git` or `node_modules`).
    Superfolder { is_atomic: bool },
}

/// Aggregated statistics for a directory or file
#[derive(Debug, Clone)]
pub struct FsStats {
    /// Earliest modification date found in the subtree
    pub min_date: Option<DateTime<Local>>,
    /// Latest modification date found in the subtree
    pub max_date: Option<DateTime<Local>>,
    /// Total count of files and folders in the subtree (recursive)
    pub count: usize,
}

impl FsStats {
    pub fn new() -> Self {
        Self {
            min_date: None,
            max_date: None,
            count: 0,
        }
    }

    /// Merges statistics from another entry into this one
    pub fn merge(&mut self, other: &FsStats) {
        if let Some(other_min) = other.min_date {
            self.min_date = Some(match self.min_date {
                Some(current) => {
                    if other_min < current {
                        other_min
                    } else {
                        current
                    }
                }
                None => other_min,
            });
        }
        if let Some(other_max) = other.max_date {
            self.max_date = Some(match self.max_date {
                Some(current) => {
                    if other_max > current {
                        other_max
                    } else {
                        current
                    }
                }
                None => other_max,
            });
        }
        self.count += other.count;
    }

    /// Updates stats with a single file's modification time
    pub fn add_file(&mut self, modified: SystemTime) {
        let date: DateTime<Local> = modified.into();
        self.min_date = Some(match self.min_date {
            Some(current) => {
                if date < current {
                    date
                } else {
                    current
                }
            }
            None => date,
        });
        self.max_date = Some(match self.max_date {
            Some(current) => {
                if date > current {
                    date
                } else {
                    current
                }
            }
            None => date,
        });
        self.count += 1;
    }
}

/// Representation of a node in the file system tree
#[derive(Debug, Clone)]
pub struct FsEntry {
    /// Absolute path to the entry
    pub path: PathBuf,
    /// Name of the entry (file or folder name)
    pub name: String,
    /// Type of entry
    pub kind: EntryKind,
    /// Aggregated statistics
    pub stats: FsStats,
    /// Child nodes (empty for files or collapsed Superfolders)
    pub children: Vec<FsEntry>,
    /// Calculated field: true if a `.superfolder` marker was found here
    pub is_user_superfolder: bool,
}

impl FsEntry {
    pub fn new_file(path: PathBuf, modified: SystemTime) -> Self {
        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();
        let mut stats = FsStats::new();
        stats.add_file(modified);
        Self {
            path,
            name,
            kind: EntryKind::File,
            stats,
            children: vec![],
            is_user_superfolder: false,
        }
    }
}

/// Scans a directory recursively to build the `FsEntry` tree.
///
/// Respects `.superfolder` markers and atomic folders configured in Config.
pub fn scan_directory(path: &Path, config: &Config) -> FsEntry {
    let name = path
        .file_name()
        .unwrap_or_default()
        .to_string_lossy()
        .to_string();
    let mut stats = FsStats::new();
    let mut children = Vec::new();

    // Check availability/permission
    let read_dir = match fs::read_dir(path) {
        Ok(rd) => rd,
        Err(_) => {
            return FsEntry {
                path: path.to_path_buf(),
                name,
                kind: EntryKind::Directory,
                stats, // empty
                children: vec![],
                is_user_superfolder: false,
            };
        }
    };

    // Check for .superfolder marker
    let is_user_superfolder = path.join(".superfolder").exists();

    // Check for Atomic folders using config
    let is_atomic = config.contains_system_folder(path);

    if is_user_superfolder || is_atomic {
        let (deep_stats, _deep_children) = scan_recursive_for_stats(path);

        return FsEntry {
            path: path.to_path_buf(),
            name,
            kind: EntryKind::Superfolder { is_atomic },
            stats: deep_stats,
            children: vec![],
            is_user_superfolder,
        };
    }

    // Normal Directory
    for entry in read_dir.flatten() {
        let entry_path = entry.path();
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        if entry_path
            .file_name()
            .map(|n| n == ".superfolder")
            .unwrap_or(false)
        {
            continue;
        }

        let child = if metadata.is_dir() {
            scan_directory(&entry_path, config)
        } else {
            let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            FsEntry::new_file(entry_path, modified)
        };

        stats.merge(&child.stats);
        children.push(child);
    }

    // Sorting children alphabetically
    children.sort_by(|a, b| a.name.cmp(&b.name));

    FsEntry {
        path: path.to_path_buf(),
        name,
        kind: EntryKind::Directory,
        stats,
        children,
        is_user_superfolder: false,
    }
}

/// Helper scan that only computes stats, used for atomic/superfolders where children are not needed in memory yet.
fn scan_recursive_for_stats(path: &Path) -> (FsStats, Vec<FsEntry>) {
    let mut stats = FsStats::new();
    let read_dir = match fs::read_dir(path) {
        Ok(rd) => rd,
        Err(_) => return (stats, vec![]),
    };

    for entry in read_dir.flatten() {
        let path = entry.path();
        let metadata = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };

        if path
            .file_name()
            .map(|n| n == ".superfolder")
            .unwrap_or(false)
        {
            continue;
        }

        if metadata.is_dir() {
            // Recurse
            let (child_stats, _) = scan_recursive_for_stats(&path);
            stats.merge(&child_stats);
        } else {
            let modified = metadata.modified().unwrap_or(SystemTime::UNIX_EPOCH);
            let mut file_stats = FsStats::new();
            file_stats.add_file(modified);
            stats.merge(&file_stats);
        }
    }
    (stats, vec![])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::tempdir;

    #[test]
    fn test_scanner_atomic() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        // Create repos/my-repo/.git structure
        let repo_path = root.join("repos/my-repo");
        fs::create_dir_all(&repo_path).unwrap();
        fs::create_dir(repo_path.join(".git")).unwrap();

        let config = Config::new();
        let entry = scan_directory(&repo_path, &config);
        match entry.kind {
            EntryKind::Superfolder { is_atomic } => assert!(is_atomic),
            _ => panic!("Expected atomic superfolder for .git containing dir"),
        }
    }

    #[test]
    fn test_scanner_user_superfolder() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        // Create archived/old_stuff/.superfolder structure
        let old_stuff = root.join("archived/old_stuff");
        fs::create_dir_all(&old_stuff).unwrap();
        File::create(old_stuff.join(".superfolder")).unwrap();
        File::create(old_stuff.join("some_file.txt")).unwrap();

        let config = Config::new();
        let entry = scan_directory(&old_stuff, &config);
        match entry.kind {
            EntryKind::Superfolder { is_atomic } => assert!(!is_atomic), // User superfolder
            _ => panic!("Expected user superfolder"),
        }

        // It should have stats
        assert!(entry.stats.count > 0, "Should have count > 0");
    }

    #[test]
    fn test_structure() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        // Create test structure:
        // root/
        //   doc1.pdf
        //   repos/
        //     my-repo/
        //       .git/

        File::create(root.join("doc1.pdf")).unwrap();
        let repos = root.join("repos");
        fs::create_dir(&repos).unwrap();
        let my_repo = repos.join("my-repo");
        fs::create_dir(&my_repo).unwrap();
        fs::create_dir(my_repo.join(".git")).unwrap();

        let config = Config::new();
        let entry = scan_directory(root, &config);

        // Root is directory
        assert!(matches!(entry.kind, EntryKind::Directory));

        // Locate 'doc1.pdf'
        let doc1 = entry.children.iter().find(|c| c.name == "doc1.pdf");
        assert!(doc1.is_some());
        assert!(matches!(doc1.unwrap().kind, EntryKind::File));

        // Locate 'repos' -> 'my-repo' (which is Superfolder)
        let repos_node = entry.children.iter().find(|c| c.name == "repos").unwrap();
        let my_repo_node = repos_node
            .children
            .iter()
            .find(|c| c.name == "my-repo")
            .unwrap();
        assert!(matches!(
            my_repo_node.kind,
            EntryKind::Superfolder { is_atomic: true }
        ));
        assert!(
            my_repo_node.children.is_empty(),
            "Superfolder should not expose children in tree"
        );
    }
}
