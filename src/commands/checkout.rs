use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;

use crate::git::commit::parse_commit;
use crate::git::object::{read_object, ObjectType};
use crate::git::refs::{branch_exists, read_branch_commit, set_current_branch};
use crate::git::repository::Repository;
use crate::git::tree::parse_tree;
use crate::git::index::Index;
use crate::git::status::working_tree_file_map;

pub fn run(target: &str) -> Result<()> {
    let repo = Repository::discover()?;

    ensure_clean_working_tree(&repo)?;

    if branch_exists(&repo, target) {
        checkout_branch(&repo, target)
    } else {
        checkout_commit(&repo, target)
    }
}

fn checkout_branch(repo: &Repository, branch_name: &str) -> Result<()> {
    let commit_hash = read_branch_commit(repo, branch_name)?;

    restore_commit(repo, &commit_hash)?;

    set_current_branch(repo, branch_name)?;

    println!("Switched to branch '{}'", branch_name);

    Ok(())
}

fn checkout_commit(repo: &Repository, commit_hash: &str) -> Result<()> {
    restore_commit(repo, commit_hash)?;

    println!("Checked out commit {}", commit_hash);

    Ok(())
}

fn restore_commit(repo: &Repository, commit_hash: &str) -> Result<()> {
    let commit_object = read_object(&repo.objects_dir(), commit_hash)?;

    if commit_object.object_type != ObjectType::Commit {
        return Err(anyhow!("object is not a commit: {}", commit_hash));
    }

    let commit = parse_commit(&commit_object.content)?;

    let current_index = Index::load(repo)?;
    let target_index = Index::from_tree(repo, &commit.tree)?;

    remove_files_not_in_target(repo, &current_index, &target_index)?;

    checkout_tree(repo, &commit.tree, &repo.worktree)?;

    target_index.save(repo)?;

    Ok(())
}


fn remove_files_not_in_target(
    repo: &Repository,
    current_index: &Index,
    target_index: &Index,
) -> Result<()> {
    for path in current_index.entries.keys() {
        if !target_index.contains_path(path) {
            let full_path = repo.worktree.join(path);

            if full_path.exists() && full_path.is_file() {
                fs::remove_file(&full_path)?;
                remove_empty_parent_dirs(repo, &full_path)?;
            }
        }
    }

    Ok(())
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

fn checkout_tree(repo: &Repository, tree_hash: &str, destination: &Path) -> Result<()> {
    let tree_object = read_object(&repo.objects_dir(), tree_hash)?;

    if tree_object.object_type != ObjectType::Tree {
        return Err(anyhow!("object is not a tree: {}", tree_hash));
    }

    let entries = parse_tree(&tree_object.content)?;

    for entry in entries {
        let target_path = destination.join(&entry.name);

        match entry.mode.as_str() {
            "100644" => {
                let blob = read_object(&repo.objects_dir(), &entry.hash)?;

                if blob.object_type != ObjectType::Blob {
                    return Err(anyhow!("tree entry is not a blob: {}", entry.hash));
                }

                fs::write(target_path, blob.content)?;
            }

            "40000" => {
                fs::create_dir_all(&target_path)?;
                checkout_tree(repo, &entry.hash, &target_path)?;
            }

            other => {
                return Err(anyhow!("unsupported tree entry mode: {}", other));
            }
        }
    }

    Ok(())
}


fn ensure_clean_working_tree(repo: &Repository) -> Result<()> {
    let index = Index::load(repo)?;
    let working_files = working_tree_file_map(repo)?;

    let mut dirty_paths = Vec::new();

    for (path, index_hash) in &index.entries {
        match working_files.get(path) {
            Some(working_hash) if working_hash != index_hash => {
                dirty_paths.push(path.clone());
            }

            None => {
                dirty_paths.push(path.clone());
            }

            _ => {}
        }
    }

    for path in working_files.keys() {
        if !index.entries.contains_key(path) {
            dirty_paths.push(path.clone());
        }
    }

    dirty_paths.sort();
    dirty_paths.dedup();

    if !dirty_paths.is_empty() {
        eprintln!("error: your local changes would be overwritten by checkout");
        eprintln!("Please commit or discard your changes before switching branches.");
        eprintln!();

        for path in dirty_paths {
            eprintln!("    {}", path);
        }

        return Err(anyhow!("checkout aborted"));
    }

    Ok(())
}