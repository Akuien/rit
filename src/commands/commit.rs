use anyhow::Result;
use chrono::Local;
use std::fs;

use crate::git::commit::serialize_commit;
use crate::git::object::{write_object, ObjectType};
use crate::git::repository::Repository;
use crate::git::worktree::write_worktree;

pub fn run(message: &str) -> Result<()> {
    let repo = Repository::discover()?;

    let tree_hash = write_worktree(&repo)?;
    let parent_hash = read_head_commit(&repo)?;

    let now = Local::now();
    let timestamp = now.timestamp();
    let timezone = now.format("%z").to_string();

    let author = "Akuien <akuien@example.com>"; // For now the author is hardcoded, but in the future it will be configurable.

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

    update_current_branch(&repo, &commit_hash)?;

    println!("[main {}] {}", &commit_hash[..7], message);
    println!("tree {}", tree_hash);

    Ok(())
}

fn read_head_commit(repo: &Repository) -> Result<Option<String>> {
    let head_content = fs::read_to_string(repo.head_path())?;

    if let Some(ref_path) = head_content.strip_prefix("ref: ") {
        let ref_path = ref_path.trim();
        let branch_path = repo.rit_dir.join(ref_path);

        if branch_path.exists() {
            let hash = fs::read_to_string(branch_path)?;
            let hash = hash.trim().to_string();

            if !hash.is_empty() {
                return Ok(Some(hash));
            }
        }
    }

    Ok(None)
}

fn update_current_branch(repo: &Repository, commit_hash: &str) -> Result<()> {
    let head_content = fs::read_to_string(repo.head_path())?;

    if let Some(ref_path) = head_content.strip_prefix("ref: ") {
        let ref_path = ref_path.trim();
        let branch_path = repo.rit_dir.join(ref_path);

        if let Some(parent_dir) = branch_path.parent() {
            fs::create_dir_all(parent_dir)?;
        }

        fs::write(branch_path, format!("{}\n", commit_hash))?;
    }

    Ok(())
}