use anyhow::{anyhow, Result};

#[derive(Debug, Clone)]
pub struct TreeEntry {
    pub mode: String,
    pub name: String,
    pub hash: String,
}

impl TreeEntry {
    pub fn new_file(name: String, hash: String) -> Self {
        Self {
            mode: "100644".to_string(),
            name,
            hash,
        }
    }
}

pub fn serialize_tree(entries: &[TreeEntry]) -> Result<Vec<u8>> {
    let mut sorted_entries = entries.to_vec();

    sorted_entries.sort_by(|a, b| a.name.cmp(&b.name));

    let mut result = Vec::new();

    for entry in sorted_entries {
        let header = format!("{} {}\0", entry.mode, entry.name);

        result.extend_from_slice(header.as_bytes());

        let raw_hash = hex::decode(&entry.hash)
            .map_err(|_| anyhow!("invalid object hash in tree entry: {}", entry.hash))?;

        if raw_hash.len() != 20 {
            return Err(anyhow!(
                "invalid object hash length for {}: expected 20 bytes, got {}",
                entry.name,
                raw_hash.len()
            ));
        }

        result.extend_from_slice(&raw_hash);
    }

    Ok(result)
}