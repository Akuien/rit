use anyhow::{anyhow, Result};
use std::fs;

use crate::git::repository::Repository;

pub fn current_branch_name(repo: &Repository) -> Result<String> {
    let head_content = fs::read_to_string(repo.head_path())?;

    let ref_path = head_content
        .strip_prefix("ref: ")
        .ok_or_else(|| anyhow!("detached HEAD is not supported yet"))?
        .trim();

    let branch_name = ref_path
        .strip_prefix("refs/heads/")
        .ok_or_else(|| anyhow!("HEAD does not point to a local branch"))?;

    Ok(branch_name.to_string())
}

pub fn current_branch_path(repo: &Repository) -> Result<std::path::PathBuf> {
    let head_content = fs::read_to_string(repo.head_path())?;

    let ref_path = head_content
        .strip_prefix("ref: ")
        .ok_or_else(|| anyhow!("detached HEAD is not supported yet"))?
        .trim();

    Ok(repo.rit_dir.join(ref_path))
}

pub fn read_head_commit(repo: &Repository) -> Result<Option<String>> {
    let branch_path = current_branch_path(repo)?;

    if !branch_path.exists() {
        return Ok(None);
    }

    let hash = fs::read_to_string(branch_path)?;
    let hash = hash.trim().to_string();

    if hash.is_empty() {
        Ok(None)
    } else {
        Ok(Some(hash))
    }
}

pub fn update_head_commit(repo: &Repository, commit_hash: &str) -> Result<()> {
    let branch_path = current_branch_path(repo)?;

    if let Some(parent_dir) = branch_path.parent() {
        fs::create_dir_all(parent_dir)?;
    }

    fs::write(branch_path, format!("{}\n", commit_hash))?;

    Ok(())
}