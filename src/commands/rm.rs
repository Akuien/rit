use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;

use crate::git::index::Index;
use crate::git::repository::Repository;

pub fn run(path: &str) -> Result<()> {
    let repo = Repository::discover()?;

    let mut index = Index::load(&repo)?;

    let repo_relative_path = normalize_path(path);

    if !index.contains_path(&repo_relative_path) {
        return Err(anyhow!("path is not tracked: {}", path));
    }

    let working_path = repo.worktree.join(&repo_relative_path);

    if working_path.exists() {
        if working_path.is_file() {
            fs::remove_file(&working_path)?;
            remove_empty_parent_dirs(&repo, &working_path)?;
        } else {
            return Err(anyhow!("only removing files is supported for now"));
        }
    }

    index.remove(&repo_relative_path);
    index.save(&repo)?;

    println!("removed {}", repo_relative_path);

    Ok(())
}

fn normalize_path(path: &str) -> String {
    Path::new(path).to_string_lossy().to_string()
}

fn remove_empty_parent_dirs(repo: &Repository, path: &Path) -> Result<()> {
    let mut current = path.parent();

    while let Some(dir) = current {
        if dir == repo.worktree {
            break;
        }

        if dir.starts_with(&repo.rit_dir) {
            break;
        }

        match fs::remove_dir(dir) {
            Ok(_) => {
                current = dir.parent();
            }
            Err(_) => {
                break;
            }
        }
    }

    Ok(())
}