use anyhow::{anyhow, Result};

use crate::git::refs::{
    create_branch, current_branch_name, list_branches, read_head_commit,
};
use crate::git::repository::Repository;

pub fn run(name: Option<&str>) -> Result<()> {
    let repo = Repository::discover()?;

    match name {
        Some(branch_name) => create_new_branch(&repo, branch_name),
        None => list_existing_branches(&repo),
    }
}

fn create_new_branch(repo: &Repository, branch_name: &str) -> Result<()> {
    let head_commit = read_head_commit(repo)?
        .ok_or_else(|| anyhow!("cannot create branch: no commits yet"))?;

    create_branch(repo, branch_name, &head_commit)?;

    println!("Created branch {}", branch_name);

    Ok(())
}

fn list_existing_branches(repo: &Repository) -> Result<()> {
    let current = current_branch_name(repo)?;
    let branches = list_branches(repo)?;

    for branch in branches {
        if branch == current {
            println!("* {}", branch);
        } else {
            println!("  {}", branch);
        }
    }

    Ok(())
}