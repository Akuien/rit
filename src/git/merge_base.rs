use anyhow::{anyhow, Result};
use std::collections::HashSet;

use crate::git::commit::parse_commit;
use crate::git::object::{read_object, ObjectType};
use crate::git::repository::Repository;

pub fn find_merge_base(repo: &Repository, first: &str, second: &str) -> Result<String> {
    let first_ancestors = collect_ancestors(repo, first)?;

    let mut current = Some(second.to_string());

    while let Some(commit_hash) = current {
        if first_ancestors.contains(&commit_hash) {
            return Ok(commit_hash);
        }

        current = parent_of(repo, &commit_hash)?;
    }

    Err(anyhow!("no common ancestor found"))
}

fn collect_ancestors(repo: &Repository, commit_hash: &str) -> Result<HashSet<String>> {
    let mut ancestors = HashSet::new();
    let mut current = Some(commit_hash.to_string());

    while let Some(hash) = current {
        ancestors.insert(hash.clone());
        current = parent_of(repo, &hash)?;
    }

    Ok(ancestors)
}

fn parent_of(repo: &Repository, commit_hash: &str) -> Result<Option<String>> {
    let object = read_object(&repo.objects_dir(), commit_hash)?;

    if object.object_type != ObjectType::Commit {
        return Err(anyhow!("object is not a commit: {}", commit_hash));
    }

    let commit = parse_commit(&object.content)?;

    Ok(commit.parent)
}