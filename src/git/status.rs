use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::git::commit::parse_commit;
use crate::git::object::{hash_object_data, read_object, serialize_object, ObjectType};
use crate::git::repository::Repository;
use crate::git::tree::parse_tree;

pub type FileMap = HashMap<String, String>;

pub fn head_tree_file_map(repo: &Repository, commit_hash: &str) -> Result<FileMap> {
    let commit_object = read_object(&repo.objects_dir(), commit_hash)?;

    if commit_object.object_type != ObjectType::Commit {
        return Err(anyhow!("HEAD does not point to a commit"));
    }

    let commit = parse_commit(&commit_object.content)?;

    let mut files = HashMap::new();
    collect_tree_files(repo, &commit.tree, Path::new(""), &mut files)?;

    Ok(files)
}

pub fn working_tree_file_map(repo: &Repository) -> Result<FileMap> {
    let mut files = HashMap::new();

    collect_working_files(repo, &repo.worktree, Path::new(""), &mut files)?;

    Ok(files)
}

fn collect_tree_files(
    repo: &Repository,
    tree_hash: &str,
    prefix: &Path,
    files: &mut FileMap,
) -> Result<()> {
    let tree_object = read_object(&repo.objects_dir(), tree_hash)?;

    if tree_object.object_type != ObjectType::Tree {
        return Err(anyhow!("object is not a tree: {}", tree_hash));
    }

    let entries = parse_tree(&tree_object.content)?;

    for entry in entries {
        let path = prefix.join(&entry.name);
        let path_string = path.to_string_lossy().to_string();

        match entry.mode.as_str() {
            "100644" => {
                files.insert(path_string, entry.hash);
            }

            "40000" => {
                collect_tree_files(repo, &entry.hash, &path, files)?;
            }

            other => {
                return Err(anyhow!("unsupported tree entry mode: {}", other));
            }
        }
    }

    Ok(())
}

fn collect_working_files(
    repo: &Repository,
    directory: &Path,
    prefix: &Path,
    files: &mut FileMap,
) -> Result<()> {
    for entry_result in fs::read_dir(directory)? {
        let entry = entry_result?;
        let path = entry.path();
        let file_name = entry.file_name().to_string_lossy().to_string();

        if should_ignore(&file_name, &path) {
            continue;
        }

        let relative_path = prefix.join(&file_name);

        if path.is_file() {
            let content = fs::read(&path)?;

            let serialized = serialize_object(&ObjectType::Blob, &content);
            let hash = hash_object_data(&serialized);

            files.insert(relative_path.to_string_lossy().to_string(), hash);
        } else if path.is_dir() {
            collect_working_files(repo, &path, &relative_path, files)?;
        }
    }

    Ok(())
}

fn should_ignore(file_name: &str, path: &PathBuf) -> bool {
    if file_name == ".rit" || file_name == ".git" || file_name == "target" {
        return true;
    }

    let path_text = path.to_string_lossy();

    if path_text.contains("/target/") {
        return true;
    }

    false
}