pub fn serialize_commit(
    tree_hash: &str,
    parent_hash: Option<&str>,
    author: &str,
    timestamp: i64,
    timezone: &str,
    message: &str,
) -> Vec<u8> {
    let mut content = String::new();

    content.push_str(&format!("tree {}\n", tree_hash));

    if let Some(parent) = parent_hash {
        content.push_str(&format!("parent {}\n", parent));
    }

    content.push_str(&format!(
        "author {} {} {}\n",
        author, timestamp, timezone
    ));

    content.push_str(&format!(
        "committer {} {} {}\n",
        author, timestamp, timezone
    ));

    content.push('\n');
    content.push_str(message);
    content.push('\n');

    content.into_bytes()
}