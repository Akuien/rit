use anyhow::{anyhow, Result};
use chrono::Local;

use crate::git::commit::serialize_commit;
use crate::git::index::Index;
use crate::git::object::{write_object, ObjectType};
use crate::git::refs::{current_branch_name, read_head_commit, update_head_commit};
use crate::git::repository::Repository;

pub fn run(message: &str) -> Result<()> {
    let repo = Repository::discover()?;

    let index = Index::load(&repo)?;

    if index.entries.is_empty() {
        return Err(anyhow!("nothing added to commit"));
    }

    let tree_hash = index.write_tree(&repo)?;
    let parent_hash = read_head_commit(&repo)?;

    let now = Local::now();
    let timestamp = now.timestamp();
    let timezone = now.format("%z").to_string();

    let author = "Akuien <akuien@example.com>";

    let commit_content = serialize_commit(
        &tree_hash,
        parent_hash.as_deref(),
        author,
        timestamp,
        &timezone,
        message,
    );

    let commit_hash = write_object(
        &repo.objects_dir(),
        ObjectType::Commit,
        &commit_content,
    )?;

    update_head_commit(&repo, &commit_hash)?;

    let branch = current_branch_name(&repo)?;

    println!("[{} {}] {}", branch, &commit_hash[..7], message);
    println!("tree {}", tree_hash);

    Ok(())
}