use anyhow::{bail, Result};
use std::fs;

use crate::git::repository::Repository;

pub fn run() -> Result<()> {
    let repo = Repository::init_current_dir()?;

    if repo.rit_dir.exists() {
        bail!("rit repository already exists");
    }

    fs::create_dir(&repo.rit_dir)?;
    fs::create_dir(repo.objects_dir())?;
    fs::create_dir_all(repo.refs_heads_dir())?;

    fs::write(repo.head_path(), "ref: refs/heads/main\n")?;

    fs::write(
        repo.config_path(),
        "[core]\n\trepositoryformatversion = 0\n\tfilemode = true\n\tbare = false\n",
    )?;

    println!("Initialized empty rit repository in {}", repo.rit_dir.display());

    Ok(())
}