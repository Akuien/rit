use anyhow::Result;

use crate::git::merge_base::find_merge_base;
use crate::git::refs::resolve_name;
use crate::git::repository::Repository;

pub fn run(first: &str, second: &str) -> Result<()> {
    let repo = Repository::discover()?;

    let first_commit = resolve_name(&repo, first)?;
    let second_commit = resolve_name(&repo, second)?;

    let merge_base = find_merge_base(&repo, &first_commit, &second_commit)?;

    println!("{}", merge_base);

    Ok(())
}