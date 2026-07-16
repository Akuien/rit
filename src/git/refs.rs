use anyhow::{anyhow, Result};
use std::fs;
use std::path::PathBuf;

use crate::git::repository::Repository;



pub fn branch_path(repo: &Repository, branch_name: &str) -> PathBuf {
    repo.refs_heads_dir().join(branch_name)
}

pub fn create_branch(repo: &Repository, branch_name: &str, commit_hash: &str) -> Result<()> {
    validate_branch_name(branch_name)?;

    let path = branch_path(repo, branch_name);

    if path.exists() {
        return Err(anyhow!("branch already exists: {}", branch_name));
    }

    if let Some(parent_dir) = path.parent() {
        fs::create_dir_all(parent_dir)?;
    }

    fs::write(path, format!("{}\n", commit_hash))?;

    Ok(())
}

pub fn list_branches(repo: &Repository) -> Result<Vec<String>> {
    let mut branches = Vec::new();

    let heads_dir = repo.refs_heads_dir();

    if !heads_dir.exists() {
        return Ok(branches);
    }

    for entry_result in fs::read_dir(heads_dir)? {
        let entry = entry_result?;
        let path = entry.path();

        if path.is_file() {
            let name = entry.file_name().to_string_lossy().to_string();
            branches.push(name);
        }
    }

    branches.sort();

    Ok(branches)
}

fn validate_branch_name(branch_name: &str) -> Result<()> {
    if branch_name.trim().is_empty() {
        return Err(anyhow!("branch name cannot be empty"));
    }

    if branch_name.contains(' ') {
        return Err(anyhow!("branch name cannot contain spaces"));
    }

    if branch_name.contains("..") {
        return Err(anyhow!("branch name cannot contain '..'"));
    }

    if branch_name.starts_with('/') || branch_name.ends_with('/') {
        return Err(anyhow!("branch name cannot start or end with '/'"));
    }

    Ok(())
}


pub fn branch_exists(repo: &Repository, branch_name: &str) -> bool {
    branch_path(repo, branch_name).exists()
}

pub fn read_branch_commit(repo: &Repository, branch_name: &str) -> Result<String> {
    let path = branch_path(repo, branch_name);

    if !path.exists() {
        return Err(anyhow!("branch does not exist: {}", branch_name));
    }

    let hash = fs::read_to_string(path)?;
    let hash = hash.trim().to_string();

    if hash.is_empty() {
        return Err(anyhow!("branch has no commits: {}", branch_name));
    }

    Ok(hash)
}

pub fn set_current_branch(repo: &Repository, branch_name: &str) -> Result<()> {
    validate_branch_name(branch_name)?;

    if !branch_exists(repo, branch_name) {
        return Err(anyhow!("branch does not exist: {}", branch_name));
    }

    fs::write(
        repo.head_path(),
        format!("ref: refs/heads/{}\n", branch_name),
    )?;

    Ok(())
}

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

pub fn delete_branch(repo: &Repository, branch_name: &str) -> Result<()> {
    validate_branch_name(branch_name)?;

    let current_branch = current_branch_name(repo)?;

    if branch_name == current_branch {
        return Err(anyhow!("cannot delete current branch: {}", branch_name));
    }

    let path = branch_path(repo, branch_name);

    if !path.exists() {
        return Err(anyhow!("branch does not exist: {}", branch_name));
    }

    fs::remove_file(path)?;

    Ok(())
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

pub fn resolve_name(repo: &Repository, name: &str) -> Result<String> {
    if branch_exists(repo, name) {
        return read_branch_commit(repo, name);
    }

    Ok(name.to_string())
}