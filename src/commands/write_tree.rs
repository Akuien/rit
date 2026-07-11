use anyhow::Result;

use crate::git::repository::Repository;
use crate::git::worktree::write_worktree;

pub fn run() -> Result<()> {
    let repo = Repository::discover()?;
    let tree_hash = write_worktree(&repo)?;

    println!("{}", tree_hash);

    Ok(())
}