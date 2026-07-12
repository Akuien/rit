use anyhow::Result;

use crate::git::refs::{current_branch_name, read_head_commit};
use crate::git::repository::Repository;
use crate::git::status::{head_tree_file_map, working_tree_file_map};

pub fn run() -> Result<()> {
    let repo = Repository::discover()?;
    let branch = current_branch_name(&repo)?;

    println!("On branch {}", branch);
    println!();

    let head_commit = read_head_commit(&repo)?;

    if head_commit.is_none() {
        println!("No commits yet");
        println!();
        println!("Untracked files:");

        let working_files = working_tree_file_map(&repo)?;

        for path in sorted_keys(&working_files) {
            println!("    {}", path);
        }

        return Ok(());
    }

    let head_hash = head_commit.unwrap();

    let head_files = head_tree_file_map(&repo, &head_hash)?;
    let working_files = working_tree_file_map(&repo)?;

    let mut modified = Vec::new();
    let mut deleted = Vec::new();
    let mut untracked = Vec::new();

    for path in head_files.keys() {
        match working_files.get(path) {
            Some(working_hash) => {
                let head_hash = head_files.get(path).unwrap();

                if working_hash != head_hash {
                    modified.push(path.clone());
                }
            }

            None => {
                deleted.push(path.clone());
            }
        }
    }

    for path in working_files.keys() {
        if !head_files.contains_key(path) {
            untracked.push(path.clone());
        }
    }

    modified.sort();
    deleted.sort();
    untracked.sort();

    if modified.is_empty() && deleted.is_empty() && untracked.is_empty() {
        println!("nothing to commit, working tree clean");
        return Ok(());
    }

    if !modified.is_empty() || !deleted.is_empty() {
        println!("Changes not staged for commit:");

        for path in modified {
            println!("    modified: {}", path);
        }

        for path in deleted {
            println!("    deleted:  {}", path);
        }

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

fn sorted_keys(map: &std::collections::HashMap<String, String>) -> Vec<String> {
    let mut keys: Vec<String> = map.keys().cloned().collect();
    keys.sort();
    keys
}