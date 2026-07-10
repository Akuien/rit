use anyhow::Result;
use std::fs;

use crate::git::object::{write_object, ObjectType};
use crate::git::repository::Repository;
use crate::git::tree::{serialize_tree, TreeEntry};

pub fn run() -> Result<()> {
    let repo = Repository::discover()?;

    let mut entries = Vec::new();

    for entry_result in fs::read_dir(&repo.worktree)? {
        let entry = entry_result?;
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();

        if file_name == ".rit" || file_name == ".git" || file_name == "target" {
            continue;
        }

        if path.is_file() {
            let content = fs::read(&path)?;
            let blob_hash = write_object(&repo.objects_dir(), ObjectType::Blob, &content)?;

            entries.push(TreeEntry::new_file(file_name, blob_hash));
        }
    }

    let tree_content = serialize_tree(&entries)?;
    let tree_hash = write_object(&repo.objects_dir(), ObjectType::Tree, &tree_content)?;

    println!("{}", tree_hash);

    Ok(())
}