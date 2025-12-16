use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Global project registry stored at ~/.devhub/registry.json
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Registry {
    pub projects: HashMap<String, ProjectEntry>,

    #[serde(skip)]
    config_dir: PathBuf,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectEntry {
    pub path: PathBuf,
    pub registered_at: DateTime<Utc>,
}

impl Registry {
    /// Load registry from default location (~/.devhub/registry.json)
    pub fn load() -> Result<Self> {
        let config_dir = get_config_dir()?;
        let registry_path = config_dir.join("registry.json");

        let mut registry = if registry_path.exists() {
            let content = std::fs::read_to_string(&registry_path)?;
            serde_json::from_str(&content)?
        } else {
            Registry::default()
        };

        registry.config_dir = config_dir;
        Ok(registry)
    }

    /// Save registry to disk
    pub fn save(&self) -> Result<()> {
        // Ensure config directory exists
        std::fs::create_dir_all(&self.config_dir)?;

        let registry_path = self.config_dir.join("registry.json");
        let content = serde_json::to_string_pretty(&self)?;
        std::fs::write(registry_path, content)?;
        Ok(())
    }

    /// Register a project
    pub fn register(&mut self, name: &str, path: &Path) -> Result<()> {
        self.projects.insert(
            name.to_string(),
            ProjectEntry {
                path: path.to_path_buf(),
                registered_at: Utc::now(),
            },
        );
        Ok(())
    }

    /// Unregister a project
    pub fn unregister(&mut self, name: &str) -> bool {
        self.projects.remove(name).is_some()
    }

    /// Get a project by name
    pub fn get(&self, name: &str) -> Option<&ProjectEntry> {
        self.projects.get(name)
    }

    /// List all projects
    pub fn list(&self) -> Vec<(&String, &ProjectEntry)> {
        let mut projects: Vec<_> = self.projects.iter().collect();
        projects.sort_by(|a, b| a.0.cmp(b.0));
        projects
    }

    /// Find project by path
    pub fn find_by_path(&self, path: &PathBuf) -> Option<(&String, &ProjectEntry)> {
        self.projects.iter().find(|(_, entry)| &entry.path == path)
    }
}

/// Get the DevHub configuration directory
fn get_config_dir() -> Result<PathBuf> {
    let base_dirs = directories::BaseDirs::new()
        .ok_or_else(|| anyhow::anyhow!("Could not determine home directory"))?;

    Ok(base_dirs.home_dir().join(".devhub"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_register_and_list() {
        let mut registry = Registry::default();
        let path = PathBuf::from("/test/path");

        registry.register("test-project", &path).unwrap();

        assert!(registry.get("test-project").is_some());
        assert_eq!(registry.list().len(), 1);
    }

    #[test]
    fn test_unregister() {
        let mut registry = Registry::default();
        let path = PathBuf::from("/test/path");

        registry.register("test-project", &path).unwrap();
        assert!(registry.unregister("test-project"));
        assert!(!registry.unregister("test-project")); // Already removed
        assert!(registry.get("test-project").is_none());
    }
}
