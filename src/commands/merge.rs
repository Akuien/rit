use anyhow::{anyhow, Result};
use std::collections::{BTreeSet, HashMap};
use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::git::commit::serialize_commit;
use crate::git::index::Index;
use crate::git::merge_base::find_merge_base;
use crate::git::object::{read_blob_content, write_object, ObjectType};
use crate::git::refs::{current_branch_name, read_head_commit, resolve_name, update_head_commit};
use crate::git::repository::Repository;
use crate::git::status::{head_tree_file_map, working_tree_file_map};

struct MergeResult {
    merged_files: HashMap<String, String>,
    conflicts: Vec<String>,
}

pub fn run(branch_name: &str) -> Result<()> {
    let repo = Repository::discover()?;

    ensure_working_tree_clean(&repo)?;

    let ours = read_head_commit(&repo)?.ok_or_else(|| anyhow!("cannot merge: no commits yet"))?;
    let theirs = resolve_name(&repo, branch_name)?;

    if ours == theirs {
        println!("Already up to date.");
        return Ok(());
    }

    let base = find_merge_base(&repo, &ours, &theirs)?;

    if base == theirs {
        println!("Already up to date.");
        return Ok(());
    }

    let base_files = head_tree_file_map(&repo, &base)?;
    let ours_files = head_tree_file_map(&repo, &ours)?;
    let theirs_files = head_tree_file_map(&repo, &theirs)?;

    let merge_result = merge_file_maps(&base_files, &ours_files, &theirs_files);

    if !merge_result.conflicts.is_empty() {
        write_conflict_files(
            &repo,
            &merge_result.conflicts,
            &ours_files,
            &theirs_files,
            branch_name,
        )?;

        eprintln!("Automatic merge failed. Fix conflicts and then commit the result.");
        eprintln!("Conflicts:");

        for path in &merge_result.conflicts {
            eprintln!("    {}", path);
        }

        return Err(anyhow!("merge failed due to conflicts"));
    }

    write_merged_worktree_and_index(&repo, &merge_result.merged_files)?;

    let tree_hash = Index {
        entries: merge_result.merged_files,
    }
    .write_tree(&repo)?;

    let current_branch = current_branch_name(&repo)?;
    let author = format!("Akuien <akuien@example.com> {} +0200", current_timestamp());
    let committer = author.clone();
    let message = format!("Merge branch '{}' into {}", branch_name, current_branch);

    let commit_content = serialize_commit(
        &tree_hash,
        Some(&ours),
        Some(&theirs),
        &author,
        &committer,
        &message,
    );

    let merge_commit_hash = write_object(&repo.objects_dir(), ObjectType::Commit, &commit_content)?;

    update_head_commit(&repo, &merge_commit_hash)?;

    println!("Merge made commit {}", &merge_commit_hash[..7]);

    Ok(())
}

fn ensure_working_tree_clean(repo: &Repository) -> Result<()> {
    let index = Index::load(repo)?;
    let working_files = working_tree_file_map(repo)?;

    if index.entries != working_files {
        return Err(anyhow!("cannot merge: working tree has uncommitted changes"));
    }

    Ok(())
}

fn merge_file_maps(
    base: &HashMap<String, String>,
    ours: &HashMap<String, String>,
    theirs: &HashMap<String, String>,
) -> MergeResult {
    let mut paths = BTreeSet::new();

    for path in base.keys() {
        paths.insert(path.clone());
    }

    for path in ours.keys() {
        paths.insert(path.clone());
    }

    for path in theirs.keys() {
        paths.insert(path.clone());
    }

    let mut merged = HashMap::new();
    let mut conflicts = Vec::new();

    for path in paths {
        let base_hash = base.get(&path);
        let ours_hash = ours.get(&path);
        let theirs_hash = theirs.get(&path);

        match (base_hash, ours_hash, theirs_hash) {
            (_, Some(ours), Some(theirs)) if ours == theirs => {
                merged.insert(path, ours.clone());
            }

            (base, ours, theirs) if ours == base && theirs != base => {
                if let Some(theirs_hash) = theirs {
                    merged.insert(path, theirs_hash.clone());
                }
            }

            (base, ours, theirs) if theirs == base && ours != base => {
                if let Some(ours_hash) = ours {
                    merged.insert(path, ours_hash.clone());
                }
            }

            (None, None, Some(theirs_hash)) => {
                merged.insert(path, theirs_hash.clone());
            }

            (None, Some(ours_hash), None) => {
                merged.insert(path, ours_hash.clone());
            }

            (Some(_), None, None) => {
                // Deleted by both sides, so keep deleted.
            }

            _ => {
                conflicts.push(path);
            }
        }
    }

    MergeResult {
        merged_files: merged,
        conflicts,
    }
}

fn write_merged_worktree_and_index(
    repo: &Repository,
    merged_files: &HashMap<String, String>,
) -> Result<()> {
    let current_index = Index::load(repo)?;

    for path in current_index.entries.keys() {
        if !merged_files.contains_key(path) {
            let full_path = repo.worktree.join(path);

            if full_path.exists() && full_path.is_file() {
                fs::remove_file(full_path)?;
            }
        }
    }

    for (path, hash) in merged_files {
        let full_path = repo.worktree.join(path);

        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let content = read_blob_content(&repo.objects_dir(), hash)?;
        fs::write(full_path, content)?;
    }

    let index = Index {
        entries: merged_files.clone(),
    };

    index.save(repo)?;

    Ok(())
}

fn write_conflict_files(
    repo: &Repository,
    conflicts: &[String],
    ours_files: &HashMap<String, String>,
    theirs_files: &HashMap<String, String>,
    branch_name: &str,
) -> Result<()> {
    for path in conflicts {
        let ours_content = match ours_files.get(path) {
            Some(hash) => read_blob_content(&repo.objects_dir(), hash)?,
            None => Vec::new(),
        };

        let theirs_content = match theirs_files.get(path) {
            Some(hash) => read_blob_content(&repo.objects_dir(), hash)?,
            None => Vec::new(),
        };

        let ours_text = String::from_utf8_lossy(&ours_content);
        let theirs_text = String::from_utf8_lossy(&theirs_content);

        let conflict_text = format!(
            "<<<<<<< HEAD\n{}=======\n{}>>>>>>> {}\n",
            ensure_trailing_newline(&ours_text),
            ensure_trailing_newline(&theirs_text),
            branch_name
        );

        let full_path = repo.worktree.join(path);

        if let Some(parent) = full_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::write(full_path, conflict_text)?;
    }

    Ok(())
}

fn ensure_trailing_newline(text: &str) -> String {
    if text.ends_with('\n') {
        text.to_string()
    } else {
        format!("{}\n", text)
    }
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}