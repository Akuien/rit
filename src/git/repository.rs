use anyhow::{anyhow, Result};
use std::env;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Repository {
    pub worktree: PathBuf,
    pub rit_dir: PathBuf,
}

impl Repository {
    pub fn new(worktree: PathBuf) -> Self {
        let rit_dir = worktree.join(".rit");

        Self { worktree, rit_dir }
    }

    pub fn discover() -> Result<Self> {
        let current_dir = env::current_dir()?;

        let repo = Self::new(current_dir);

        if !repo.rit_dir.exists() {
            return Err(anyhow!("not a rit repository: .rit directory was not found"));
        }

        Ok(repo)
    }

    pub fn init_current_dir() -> Result<Self> {
        let current_dir = env::current_dir()?;
        Ok(Self::new(current_dir))
    }

    pub fn objects_dir(&self) -> PathBuf {
        self.rit_dir.join("objects")
    }

    pub fn refs_dir(&self) -> PathBuf {
        self.rit_dir.join("refs")
    }

    pub fn refs_heads_dir(&self) -> PathBuf {
        self.refs_dir().join("heads")
    }

    pub fn head_path(&self) -> PathBuf {
        self.rit_dir.join("HEAD")
    }

    pub fn config_path(&self) -> PathBuf {
        self.rit_dir.join("config")
    }
}