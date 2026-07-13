use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::Path;

use crate::git::object::{read_object, write_object, ObjectType};
use crate::git::repository::Repository;
use crate::git::tree::{parse_tree, serialize_tree, TreeEntry};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Index {
    pub entries: HashMap<String, String>,
}

#[derive(Debug, Default)]
struct IndexTreeNode {
    files: BTreeMap<String, String>,
    directories: BTreeMap<String, IndexTreeNode>,
}

impl Index {
    pub fn load(repo: &Repository) -> Result<Self> {
        let path = repo.index_path();

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(path)?;
        let index = serde_json::from_str(&content)?;

        Ok(index)
    }

    pub fn save(&self, repo: &Repository) -> Result<()> {
        let content = serde_json::to_string_pretty(self)?;
        fs::write(repo.index_path(), content)?;
        Ok(())
    }

    pub fn add(&mut self, path: String, hash: String) {
        self.entries.insert(path, hash);
    }

    pub fn remove(&mut self, path: &str) -> Option<String> {
    self.entries.remove(path)
    }

    pub fn contains_path(&self, path: &str) -> bool {
    self.entries.contains_key(path)
    }

    pub fn write_tree(&self, repo: &Repository) -> Result<String> {
        let mut root = IndexTreeNode::default();

        for (path, hash) in &self.entries {
            insert_path(&mut root, path, hash)?;
        }

        write_index_tree_node(repo, &root)
    }

    pub fn from_tree(repo: &Repository, tree_hash: &str) -> Result<Self> {
    let mut index = Self::default();

    collect_tree_entries(repo, tree_hash, Path::new(""), &mut index.entries)?;

    Ok(index)
    }
}

fn insert_path(root: &mut IndexTreeNode, path: &str, hash: &str) -> Result<()> {
    let parts: Vec<&str> = path.split('/').filter(|part| !part.is_empty()).collect();

    if parts.is_empty() {
        return Err(anyhow!("invalid empty index path"));
    }

    let mut current = root;

    for directory in &parts[..parts.len() - 1] {
        current = current
            .directories
            .entry((*directory).to_string())
            .or_default();
    }

    let file_name = parts[parts.len() - 1].to_string();
    current.files.insert(file_name, hash.to_string());

    Ok(())
}

fn write_index_tree_node(repo: &Repository, node: &IndexTreeNode) -> Result<String> {
    let mut entries = Vec::new();

    for (name, hash) in &node.files {
        entries.push(TreeEntry::new_file(name.clone(), hash.clone()));
    }

    for (name, child_node) in &node.directories {
        let subtree_hash = write_index_tree_node(repo, child_node)?;
        entries.push(TreeEntry::new_tree(name.clone(), subtree_hash));
    }

    let tree_content = serialize_tree(&entries)?;
    let tree_hash = write_object(&repo.objects_dir(), ObjectType::Tree, &tree_content)?;

    Ok(tree_hash)
}


fn collect_tree_entries(
    repo: &Repository,
    tree_hash: &str,
    prefix: &Path,
    entries: &mut HashMap<String, String>,
) -> Result<()> {
    let tree_object = read_object(&repo.objects_dir(), tree_hash)?;

    if tree_object.object_type != ObjectType::Tree {
        return Err(anyhow!("object is not a tree: {}", tree_hash));
    }

    let tree_entries = parse_tree(&tree_object.content)?;

    for entry in tree_entries {
        let path = prefix.join(&entry.name);
        let path_string = path.to_string_lossy().to_string();

        match entry.mode.as_str() {
            "100644" => {
                entries.insert(path_string, entry.hash);
            }

            "40000" => {
                collect_tree_entries(repo, &entry.hash, &path, entries)?;
            }

            other => {
                return Err(anyhow!("unsupported tree entry mode: {}", other));
            }
        }
    }

    Ok(())
}