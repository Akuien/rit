use anyhow::{anyhow, Result};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::git::commit::serialize_commit;
use crate::git::index::Index;
use crate::git::object::{write_object, ObjectType};
use crate::git::refs::{read_head_commit, update_head_commit, current_branch_name};
use crate::git::repository::Repository;

pub fn run(message: &str) -> Result<()> {
    let repo = Repository::discover()?;

    let index = Index::load(&repo)?;

    if index.entries.is_empty() {
        return Err(anyhow!("nothing added to commit"));
    }

    let tree_hash = index.write_tree(&repo)?;

    let parent = read_head_commit(&repo)?;

    let timestamp = current_timestamp();
    let author = format!("Akuien <akuien@example.com> {} +0200", timestamp);
    let committer = author.clone();

    let commit_content = serialize_commit(
        &tree_hash,
        parent.as_deref(),
        None,
        &author,
        &committer,
        message,
    );

    let commit_hash = write_object(&repo.objects_dir(), ObjectType::Commit, &commit_content)?;

    update_head_commit(&repo, &commit_hash)?;

    let branch = current_branch_name(&repo)?;
    println!("[{} {}] {}", branch, &commit_hash[..7], message);
    println!("tree {}", tree_hash);

    Ok(())
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}