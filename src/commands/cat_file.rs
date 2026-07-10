use anyhow::Result;
use std::io::{self, Write};

use crate::git::object::{read_object, ObjectType};
use crate::git::repository::Repository;
use crate::git::tree::parse_tree;

pub fn run(hash: &str) -> Result<()> {
    let repo = Repository::discover()?;
    let object = read_object(&repo.objects_dir(), hash)?;

    match object.object_type {
        ObjectType::Blob => {
            io::stdout().write_all(&object.content)?;
        }

        ObjectType::Tree => {
            let entries = parse_tree(&object.content)?;

            for entry in entries {
                println!(
                    "{} {} {}\t{}",
                    entry.mode,
                    object_kind_from_mode(&entry.mode),
                    entry.hash,
                    entry.name
                );
            }
        }

        ObjectType::Commit => {
            io::stdout().write_all(&object.content)?;
        }

        ObjectType::Tag => {
            io::stdout().write_all(&object.content)?;
        }
    }

    Ok(())
}

fn object_kind_from_mode(mode: &str) -> &'static str {
    match mode {
        "40000" => "tree",
        _ => "blob",
    }
}