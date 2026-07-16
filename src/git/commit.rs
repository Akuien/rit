#[derive(Debug, Clone)]
pub struct Commit {
    pub tree: String,
    pub parent: Option<String>,
    pub author: String,
    pub committer: String,
    pub message: String,
}

pub fn serialize_commit(
    tree: &str,
    parent: Option<&str>,
    second_parent: Option<&str>,
    author: &str,
    committer: &str,
    message: &str,
) -> Vec<u8> {
    let mut content = String::new();

    content.push_str(&format!("tree {}\n", tree));

    if let Some(parent_hash) = parent {
        content.push_str(&format!("parent {}\n", parent_hash));
    }

    if let Some(second_parent_hash) = second_parent {
        content.push_str(&format!("parent {}\n", second_parent_hash));
    }

    content.push_str(&format!("author {}\n", author));
    content.push_str(&format!("committer {}\n", committer));
    content.push('\n');
    content.push_str(message);
    content.push('\n');

    content.into_bytes()
}

pub fn parse_commit(data: &[u8]) -> anyhow::Result<Commit> {
    let text = std::str::from_utf8(data)?;

    let mut headers = Vec::new();
    let mut message_lines = Vec::new();
    let mut in_message = false;

    for line in text.lines() {
        if line.is_empty() && !in_message {
            in_message = true;
            continue;
        }

        if in_message {
            message_lines.push(line);
        } else {
            headers.push(line);
        }
    }

    let mut tree = None;
    let mut parent = None;
    let mut author = None;
    let mut committer = None;

    for header in headers {
        if let Some(value) = header.strip_prefix("tree ") {
            tree = Some(value.to_string());
        } else if let Some(value) = header.strip_prefix("parent ") {
            // A merge commit can have multiple parent lines.
            // For now, rit stores only the first parent so normal log traversal
            if parent.is_none() {
                parent = Some(value.to_string());
            }
        } else if let Some(value) = header.strip_prefix("author ") {
            author = Some(value.to_string());
        } else if let Some(value) = header.strip_prefix("committer ") {
            committer = Some(value.to_string());
        }
    }

    Ok(Commit {
        tree: tree.ok_or_else(|| anyhow::anyhow!("commit missing tree"))?,
        parent,
        author: author.ok_or_else(|| anyhow::anyhow!("commit missing author"))?,
        committer: committer.ok_or_else(|| anyhow::anyhow!("commit missing committer"))?,
        message: message_lines.join("\n"),
    })
}