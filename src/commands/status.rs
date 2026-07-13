use anyhow::Result;
use std::collections::{BTreeSet, HashMap};

use crate::git::index::Index;
use crate::git::refs::{current_branch_name, read_head_commit};
use crate::git::repository::Repository;
use crate::git::status::{head_tree_file_map, working_tree_file_map};

pub fn run() -> Result<()> {
    let repo = Repository::discover()?;
    let branch = current_branch_name(&repo)?;

    println!("On branch {}", branch);
    println!();

    let head_commit = read_head_commit(&repo)?;
    let index = Index::load(&repo)?;
    let working_files = working_tree_file_map(&repo)?;

    let head_files = if let Some(head_hash) = head_commit {
        head_tree_file_map(&repo, &head_hash)?
    } else {
        HashMap::new()
    };

    if head_files.is_empty() && index.entries.is_empty() {
        println!("No commits yet");
        println!();

        let untracked = sorted_keys(&working_files);

        if !untracked.is_empty() {
            println!("Untracked files:");
            for path in untracked {
                println!("    {}", path);
            }
            println!();
        }

        return Ok(());
    }

    let staged = compare_maps(&head_files, &index.entries);
    let unstaged = compare_maps(&index.entries, &working_files);
    let untracked = find_untracked(&head_files, &index.entries, &working_files);

    if staged.is_empty() && unstaged.is_empty() && untracked.is_empty() {
        println!("nothing to commit, working tree clean");
        return Ok(());
    }

    if !staged.is_empty() {
        println!("Changes to be committed:");
        print_changes(&staged);
        println!();
    }

    if !unstaged.is_empty() {
        println!("Changes not staged for commit:");
        print_changes(&unstaged);
        println!();
    }

    if !untracked.is_empty() {
        println!("Untracked files:");
        for path in untracked {
            println!("    {}", path);
        }
        println!();
    }

    Ok(())
}

#[derive(Debug, Clone)]
enum ChangeKind {
    Added,
    Modified,
    Deleted,
}

#[derive(Debug, Clone)]
struct Change {
    kind: ChangeKind,
    path: String,
}

fn compare_maps(old: &HashMap<String, String>, new: &HashMap<String, String>) -> Vec<Change> {
    let mut all_paths = BTreeSet::new();

    for path in old.keys() {
        all_paths.insert(path.clone());
    }

    for path in new.keys() {
        all_paths.insert(path.clone());
    }

    let mut changes = Vec::new();

    for path in all_paths {
        match (old.get(&path), new.get(&path)) {
            (None, Some(_)) => {
                changes.push(Change {
                    kind: ChangeKind::Added,
                    path,
                });
            }

            (Some(_), None) => {
                changes.push(Change {
                    kind: ChangeKind::Deleted,
                    path,
                });
            }

            (Some(old_hash), Some(new_hash)) if old_hash != new_hash => {
                changes.push(Change {
                    kind: ChangeKind::Modified,
                    path,
                });
            }

            _ => {}
        }
    }

    changes
}

fn find_untracked(
    head: &HashMap<String, String>,
    index: &HashMap<String, String>,
    working: &HashMap<String, String>,
) -> Vec<String> {
    let mut result = Vec::new();

    for path in working.keys() {
        if !head.contains_key(path) && !index.contains_key(path) {
            result.push(path.clone());
        }
    }

    result.sort();
    result
}

fn print_changes(changes: &[Change]) {
    for change in changes {
        let label = match change.kind {
            ChangeKind::Added => "new file:",
            ChangeKind::Modified => "modified:",
            ChangeKind::Deleted => "deleted:",
        };

        println!("    {:<10} {}", label, change.path);
    }
}

fn sorted_keys(map: &HashMap<String, String>) -> Vec<String> {
    let mut keys: Vec<String> = map.keys().cloned().collect();
    keys.sort();
    keys
}