use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;

use crate::git::index::Index;
use crate::git::object::read_blob_content;
use crate::git::repository::Repository;

pub fn run(path: &str) -> Result<()> {
    let repo = Repository::discover()?;

    let index = Index::load(&repo)?;
    let repo_relative_path = normalize_path(path);

    let blob_hash = index
        .entries
        .get(&repo_relative_path)
        .ok_or_else(|| anyhow!("path is not tracked in index: {}", path))?;

    let content = read_blob_content(&repo.objects_dir(), blob_hash)?;

    let working_path = repo.worktree.join(&repo_relative_path);

    if let Some(parent) = working_path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(&working_path, content)?;

    println!("restored {}", repo_relative_path);

    Ok(())
}

fn normalize_path(path: &str) -> String {
    Path::new(path).to_string_lossy().to_string()
}