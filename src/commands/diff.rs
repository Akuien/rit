use anyhow::Result;
use std::fs;
use std::path::Path;

use crate::git::index::Index;
use crate::git::object::read_blob_content;
use crate::git::repository::Repository;
use crate::git::status::working_tree_file_map;

pub fn run() -> Result<()> {
    let repo = Repository::discover()?;
    let index = Index::load(&repo)?;
    let working_files = working_tree_file_map(&repo)?;

    let mut changed_paths = Vec::new();

    for (path, index_hash) in &index.entries {
        match working_files.get(path) {
            Some(working_hash) if working_hash != index_hash => {
                changed_paths.push(path.clone());
            }

            None => {
                changed_paths.push(path.clone());
            }

            _ => {}
        }
    }

    changed_paths.sort();

    for path in changed_paths {
        print_file_diff(&repo, &index, &path)?;
    }

    Ok(())
}

fn print_file_diff(repo: &Repository, index: &Index, path: &str) -> Result<()> {
    let old_content = match index.entries.get(path) {
        Some(hash) => read_blob_content(&repo.objects_dir(), hash)?,
        None => Vec::new(),
    };

    let full_path = repo.worktree.join(path);

    let new_content = if full_path.exists() {
        fs::read(&full_path)?
    } else {
        Vec::new()
    };

    print_simple_diff(path, &old_content, &new_content);

    Ok(())
}

fn print_simple_diff(path: &str, old_content: &[u8], new_content: &[u8]) {
    let old_text = String::from_utf8_lossy(old_content);
    let new_text = String::from_utf8_lossy(new_content);

    let old_lines: Vec<&str> = old_text.lines().collect();
    let new_lines: Vec<&str> = new_text.lines().collect();

    println!("diff --rit a/{0} b/{0}", path);
    println!("--- a/{}", path);
    println!("+++ b/{}", path);
    println!("@@");

    let max_len = old_lines.len().max(new_lines.len());

    for i in 0..max_len {
        match (old_lines.get(i), new_lines.get(i)) {
            (Some(old), Some(new)) if old == new => {
                println!(" {}", old);
            }

            (Some(old), Some(new)) => {
                println!("-{}", old);
                println!("+{}", new);
            }

            (Some(old), None) => {
                println!("-{}", old);
            }

            (None, Some(new)) => {
                println!("+{}", new);
            }

            (None, None) => {}
        }
    }

    println!();
}