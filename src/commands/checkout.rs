use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;

use crate::git::commit::parse_commit;
use crate::git::object::{read_object, ObjectType};
use crate::git::repository::Repository;
use crate::git::tree::parse_tree;

pub fn run(commit_hash: &str) -> Result<()> {
    let repo = Repository::discover()?;

    let commit_object = read_object(&repo.objects_dir(), commit_hash)?;

    if commit_object.object_type != ObjectType::Commit {
        return Err(anyhow!("object is not a commit: {}", commit_hash));
    }

    let commit = parse_commit(&commit_object.content)?;

    checkout_tree(&repo, &commit.tree, &repo.worktree)?;

    println!("Checked out commit {}", commit_hash);

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