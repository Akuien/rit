use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::git::object::{write_object, ObjectType};
use crate::git::repository::Repository;
use crate::git::tree::{serialize_tree, TreeEntry};

pub fn run() -> Result<()> {
    let repo = Repository::discover()?;

    let tree_hash = write_tree_for_directory(
        &repo.worktree,
        &repo.worktree,
        &repo.objects_dir(),
    )?;

    println!("{}", tree_hash);

    Ok(())
}

fn write_tree_for_directory(
    root: &Path,
    directory: &Path,
    objects_dir: &Path,
) -> Result<String> {
    let mut entries = Vec::new();

    for entry_result in fs::read_dir(directory)? {
        let entry = entry_result?;
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();

        if should_ignore(root, &path, &file_name) {
            continue;
        }

        if path.is_file() {
            let content = fs::read(&path)?;
            let blob_hash = write_object(objects_dir, ObjectType::Blob, &content)?;

            entries.push(TreeEntry::new_file(file_name, blob_hash));
        } else if path.is_dir() {
            let subtree_hash = write_tree_for_directory(root, &path, objects_dir)?;

            entries.push(TreeEntry::new_tree(file_name, subtree_hash));
        }
    }

    let tree_content = serialize_tree(&entries)?;
    let tree_hash = write_object(objects_dir, ObjectType::Tree, &tree_content)?;

    Ok(tree_hash)
}

fn should_ignore(root: &Path, path: &Path, file_name: &str) -> bool {
    if file_name == ".rit" || file_name == ".git" || file_name == "target" {
        return true;
    }

    if let Ok(relative_path) = path.strip_prefix(root) {
        let relative = relative_path.to_string_lossy();

        if relative.starts_with("target/") {
            return true;
        }
    }

    false
}