use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;

use crate::git::index::Index;
use crate::git::object::read_blob_content;
use crate::git::refs::read_head_commit;
use crate::git::repository::Repository;
use crate::git::status::head_tree_file_map;

pub fn run(path: &str, staged: bool) -> Result<()> {
    let repo = Repository::discover()?;

    if staged {
        restore_staged(&repo, path)
    } else {
        restore_working_tree(&repo, path)
    }
}

fn restore_working_tree(repo: &Repository, path: &str) -> Result<()> {
    let index = Index::load(repo)?;
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

fn restore_staged(repo: &Repository, path: &str) -> Result<()> {
    let repo_relative_path = normalize_path(path);

    let head_commit = read_head_commit(repo)?
        .ok_or_else(|| anyhow!("cannot restore staged path: no commits yet"))?;

    let head_files = head_tree_file_map(repo, &head_commit)?;

    let mut index = Index::load(repo)?;

    match head_files.get(&repo_relative_path) {
        Some(head_hash) => {
            index.add(repo_relative_path.clone(), head_hash.clone());
        }

        None => {
            if !index.contains_path(&repo_relative_path) {
                return Err(anyhow!("path is not staged: {}", path));
            }

            index.remove(&repo_relative_path);
        }
    }

    index.save(repo)?;

    println!("unstaged {}", repo_relative_path);

    Ok(())
}

fn normalize_path(path: &str) -> String {
    Path::new(path).to_string_lossy().to_string()
}