use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;

use crate::git::repository::Repository;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Index {
    pub entries: HashMap<String, String>,
}

impl Index {
    pub fn load(repo: &Repository) -> Result<Self> {
        let path = repo.index_path();

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)?;
        let index = serde_json::from_str(&content)?;

        Ok(index)
    }

    pub fn save(&self, repo: &Repository) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(repo.index_path(), content)?;
        Ok(())
    }

    pub fn add(&mut self, path: String, hash: String) {
        self.entries.insert(path, hash);
    }
}