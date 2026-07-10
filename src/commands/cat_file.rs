use anyhow::Result;
use std::io::{self, Write};

use crate::git::object::read_object;
use crate::git::repository::Repository;

pub fn run(hash: &str) -> Result<()> {
    let repo = Repository::discover()?;
    let object = read_object(&repo.objects_dir(), hash)?;

    io::stdout().write_all(&object.content)?;

    Ok(())
}