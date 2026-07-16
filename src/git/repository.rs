use anyhow::{anyhow, Result};
use std::env;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Repository {
    pub worktree: PathBuf,
    pub rit_dir: PathBuf,
}

impl Repository {
    pub fn discover() -> Result<Self> {
        let mut current = env::current_dir()?;

        loop {
            let rit_dir = current.join(".rit");

            if rit_dir.is_dir() {
                return Ok(Self {
                    worktree: current,
                    rit_dir,
                });
            }

            if !current.pop() {
                return Err(anyhow!("not a rit repository"));
            }
        }
    }

    pub fn init_current_dir() -> Result<Self> {
        let worktree = env::current_dir()?;
        let rit_dir = worktree.join(".rit");

        if rit_dir.exists() {
            return Err(anyhow!("rit repository already exists"));
        }

        fs::create_dir_all(rit_dir.join("objects"))?;
        fs::create_dir_all(rit_dir.join("refs").join("heads"))?;

        fs::write(rit_dir.join("HEAD"), "ref: refs/heads/main\n")?;
        fs::write(rit_dir.join("config"), "")?;

        Ok(Self { worktree, rit_dir })
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

    pub fn index_path(&self) -> PathBuf {
        self.rit_dir.join("index")
    }

    pub fn merge_head_path(&self) -> PathBuf {
        self.rit_dir.join("MERGE_HEAD")
    }
}