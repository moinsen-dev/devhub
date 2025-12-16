use anyhow::Result;
use chrono::{DateTime, Utc};
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;
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
    #[serde(default)]
    pub last_used: Option<DateTime<Utc>>,
    #[serde(default)]
    pub favorite: bool,
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
                last_used: None,
                favorite: false,
            },
        );
        Ok(())
    }

    /// Mark a project as recently used
    pub fn touch(&mut self, name: &str) {
        if let Some(entry) = self.projects.get_mut(name) {
            entry.last_used = Some(Utc::now());
        }
    }

    /// Toggle favorite status for a project
    pub fn toggle_favorite(&mut self, name: &str) -> Option<bool> {
        if let Some(entry) = self.projects.get_mut(name) {
            entry.favorite = !entry.favorite;
            Some(entry.favorite)
        } else {
            None
        }
    }

    /// Set favorite status for a project
    pub fn set_favorite(&mut self, name: &str, favorite: bool) -> bool {
        if let Some(entry) = self.projects.get_mut(name) {
            entry.favorite = favorite;
            true
        } else {
            false
        }
    }

    /// List favorite projects
    pub fn favorites(&self) -> Vec<(&String, &ProjectEntry)> {
        let mut projects: Vec<_> = self
            .projects
            .iter()
            .filter(|(_, entry)| entry.favorite)
            .collect();
        projects.sort_by(|a, b| a.0.cmp(b.0));
        projects
    }

    /// List recent projects (sorted by last_used, most recent first)
    pub fn recent(&self, limit: usize) -> Vec<(&String, &ProjectEntry)> {
        let mut projects: Vec<_> = self
            .projects
            .iter()
            .filter(|(_, entry)| entry.last_used.is_some())
            .collect();
        projects.sort_by(|a, b| b.1.last_used.cmp(&a.1.last_used));
        projects.into_iter().take(limit).collect()
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

    /// Fuzzy search for projects by name
    /// Returns projects sorted by match score (best match first)
    pub fn fuzzy_search(&self, query: &str) -> Vec<(&String, &ProjectEntry, i64)> {
        let matcher = SkimMatcherV2::default();
        let mut matches: Vec<_> = self
            .projects
            .iter()
            .filter_map(|(name, entry)| {
                matcher
                    .fuzzy_match(name, query)
                    .map(|score| (name, entry, score))
            })
            .collect();

        // Sort by score (highest first)
        matches.sort_by(|a, b| b.2.cmp(&a.2));
        matches
    }

    /// Find a single project by exact name or fuzzy match
    /// Returns the best match if query doesn't match exactly
    #[allow(dead_code)]
    pub fn find_fuzzy(&self, query: &str) -> Option<(&String, &ProjectEntry)> {
        // First try exact match
        if let Some(entry) = self.projects.get(query) {
            return Some((self.projects.keys().find(|k| *k == query).unwrap(), entry));
        }

        // Fall back to fuzzy search
        self.fuzzy_search(query)
            .into_iter()
            .next()
            .map(|(name, entry, _)| (name, entry))
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
