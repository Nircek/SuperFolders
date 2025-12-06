use std::path::Path;

/// Configuration for SuperFolders application
#[derive(Debug, Clone)]
pub struct Config {
    /// List of system folder names that identify atomic superfolders
    pub system_folders: Vec<String>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            system_folders: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                ".venv".to_string(),
                "venv".to_string(),
                "__pycache__".to_string(),
                "target".to_string(),  // Rust build directory
                "build".to_string(),   // Common build directory
                "dist".to_string(),    // Distribution directory
                ".idea".to_string(),   // IntelliJ IDEA
                ".vscode".to_string(), // Visual Studio Code
            ],
        }
    }
}

impl Config {
    /// Create a new configuration with default system folders
    pub fn new() -> Self {
        Self::default()
    }

    /// Check if a directory contains any system folders
    pub fn contains_system_folder(&self, path: &Path) -> bool {
        self.system_folders
            .iter()
            .any(|folder| path.join(folder).exists())
    }

    /// Add a custom system folder to the configuration
    #[allow(dead_code)]
    pub fn add_system_folder(&mut self, folder: String) {
        if !self.system_folders.contains(&folder) {
            self.system_folders.push(folder);
        }
    }

    /// Remove a system folder from the configuration
    #[allow(dead_code)]
    pub fn remove_system_folder(&mut self, folder: &str) -> bool {
        if let Some(pos) = self.system_folders.iter().position(|f| f == folder) {
            self.system_folders.remove(pos);
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_default_system_folders() {
        let config = Config::default();
        assert!(config.system_folders.contains(&".git".to_string()));
        assert!(config.system_folders.contains(&"node_modules".to_string()));
    }

    #[test]
    fn test_contains_system_folder() {
        let dir = tempdir().unwrap();
        let root = dir.path();

        // Create .git directory
        fs::create_dir(root.join(".git")).unwrap();

        let config = Config::new();
        assert!(config.contains_system_folder(root));
    }

    #[test]
    fn test_add_remove_system_folder() {
        let mut config = Config::new();

        // Add custom folder
        config.add_system_folder("custom_folder".to_string());
        assert!(config.system_folders.contains(&"custom_folder".to_string()));

        // Remove folder
        assert!(config.remove_system_folder("custom_folder"));
        assert!(!config.system_folders.contains(&"custom_folder".to_string()));
    }
}
