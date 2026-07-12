use anyhow::{anyhow, Result};

use crate::git::commit::parse_commit;
use crate::git::object::{read_object, ObjectType};
use crate::git::refs::read_head_commit;
use crate::git::repository::Repository;

pub fn run() -> Result<()> {
    let repo = Repository::discover()?;

    let mut current_hash = read_head_commit(&repo)?;

    if current_hash.is_none() {
        println!("No commits yet");
        return Ok(());
    }

    while let Some(hash) = current_hash {
        let object = read_object(&repo.objects_dir(), &hash)?;

        if object.object_type != ObjectType::Commit {
            return Err(anyhow!("object is not a commit: {}", hash));
        }

        let commit = parse_commit(&object.content)?;

        print_commit(&hash, &commit);

        current_hash = commit.parent;
    }

    Ok(())
}

fn print_commit(hash: &str, commit: &crate::git::commit::Commit) {
    println!("commit {}", hash);
    println!("Author: {}", display_author_without_timestamp(&commit.author));
    println!();

    for line in commit.message.lines() {
        println!("    {}", line);
    }

    println!();
}

fn display_author_without_timestamp(author: &str) -> String {
    let parts: Vec<&str> = author.split_whitespace().collect();

    if parts.len() < 3 {
        return author.to_string();
    }

    parts[..parts.len() - 2].join(" ")
}