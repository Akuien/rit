use anyhow::{anyhow, Result};

use crate::git::refs::{
    create_branch, current_branch_name, delete_branch, list_branches, read_head_commit,
};
use crate::git::repository::Repository;

pub fn run(name: Option<&str>, delete: bool) -> Result<()> {
    let repo = Repository::discover()?;

    match (delete, name) {
        (true, Some(branch_name)) => delete_existing_branch(&repo, branch_name),
        (true, None) => Err(anyhow!("branch name required for deletion")),
        (false, Some(branch_name)) => create_new_branch(&repo, branch_name),
        (false, None) => list_existing_branches(&repo),
    }
}

fn create_new_branch(repo: &Repository, branch_name: &str) -> Result<()> {
    let head_commit =
        read_head_commit(repo)?.ok_or_else(|| anyhow!("cannot create branch: no commits yet"))?;

    create_branch(repo, branch_name, &head_commit)?;

    println!("Created branch {}", branch_name);

    Ok(())
}

fn delete_existing_branch(repo: &Repository, branch_name: &str) -> Result<()> {
    delete_branch(repo, branch_name)?;

    println!("Deleted branch {}", branch_name);

    Ok(())
}

fn list_existing_branches(repo: &Repository) -> Result<()> {
    let branches = list_branches(repo)?;
    let current = current_branch_name(repo)?;

    for branch in branches {
        if branch == current {
            println!("* {}", branch);
        } else {
            println!("  {}", branch);
        }
    }

    Ok(())
}