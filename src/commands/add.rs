use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;

use crate::git::index::Index;
use crate::git::object::{write_object, ObjectType};
use crate::git::repository::Repository;

pub fn run(path: &str) -> Result<()> {
    let repo = Repository::discover()?;

    let input_path = Path::new(path);

    if !input_path.exists() {
        return Err(anyhow!("path does not exist: {}", path));
    }

    if input_path.is_file() {
        add_file(&repo, input_path)?;
    } else {
        return Err(anyhow!("only adding individual files is supported for now"));
    }

    Ok(())
}

fn add_file(repo: &Repository, path: &Path) -> Result<()> {
    let content = fs::read(path)?;
    let hash = write_object(&repo.objects_dir(), ObjectType::Blob, &content)?;

    let relative_path = path
        .strip_prefix(&repo.worktree)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string();

    let mut index = Index::load(repo)?;
    index.add(relative_path.clone(), hash);
    index.save(repo)?;

    println!("added {}", relative_path);

    Ok(())
}