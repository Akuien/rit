use anyhow::Result;
use chrono::Local;

use crate::git::commit::serialize_commit;
use crate::git::object::{write_object, ObjectType};
use crate::git::refs::{current_branch_name, read_head_commit, update_head_commit};
use crate::git::repository::Repository;
use crate::git::worktree::write_worktree;

pub fn run(message: &str) -> Result<()> {
    let repo = Repository::discover()?;

    let tree_hash = write_worktree(&repo)?;
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