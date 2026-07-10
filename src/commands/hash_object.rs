use anyhow::{anyhow, Result};
use std::fs;
use std::path::Path;

use crate::git::object::{write_object, ObjectType};
use crate::git::repository::Repository;

pub fn run(path: &str) -> Result<()> {
    let file_path = Path::new(path);

    if !file_path.exists() {
        return Err(anyhow!("file does not exist: {}", path));
    }

    if !file_path.is_file() {
        return Err(anyhow!("path is not a file: {}", path));
    }

    let repo = Repository::discover()?;

    let content = fs::read(file_path)?;
    let hash = write_object(&repo.objects_dir(), ObjectType::Blob, &content)?;

    println!("{}", hash);

    Ok(())
}