use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};

/// Configuration for SuperFolders application
#[derive(Debug, Clone, Serialize, Deserialize)]
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
        Self::load().unwrap_or_default()
    }

    /// Load configuration from file or environment variables
    pub fn load() -> Option<Self> {
        // First, check for environment variable
        if let Ok(env_folders) = env::var("SUPERFOLDERS_SYSTEM_FOLDERS") {
            let folders: Vec<String> = env_folders
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
            if !folders.is_empty() {
                return Some(Config {
                    system_folders: folders,
                });
            }
        }

        // Then, check for config file next to binary or in home directory
        let config_paths = vec![Self::get_binary_dir_config(), Self::get_home_dir_config()];

        for path in config_paths.into_iter().flatten() {
            if !path.exists() {
                continue;
            }

            let content = match fs::read_to_string(&path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            if let Ok(config) = toml::from_str::<Config>(&content) {
                return Some(config);
            }
        }

        None
    }

    fn get_binary_dir_config() -> Option<PathBuf> {
        let exe_path = env::current_exe().ok()?;
        let exe_dir = exe_path.parent()?;
        Some(exe_dir.join("superfolders.toml"))
    }

    fn get_home_dir_config() -> Option<PathBuf> {
        let home = dirs::home_dir()?;
        Some(home.join(".superfolders.toml"))
    }

    /// Save the current configuration to a file
    pub fn save(&self, path: &Path) -> std::io::Result<()> {
        let content = toml::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        fs::write(path, content)
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
