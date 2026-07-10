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

pub fn parse_tree(data: &[u8]) -> Result<Vec<TreeEntry>> {
    let mut entries = Vec::new();
    let mut index = 0;

    while index < data.len() {
        let mode_start = index;

        while index < data.len() && data[index] != b' ' {
            index += 1;
        }

        if index >= data.len() {
            return Err(anyhow!("invalid tree object: missing space after mode"));
        }

        let mode = std::str::from_utf8(&data[mode_start..index])?.to_string();

        index += 1;

        let name_start = index;

        while index < data.len() && data[index] != 0 {
            index += 1;
        }

        if index >= data.len() {
            return Err(anyhow!("invalid tree object: missing null byte after name"));
        }

        let name = std::str::from_utf8(&data[name_start..index])?.to_string();

        index += 1;

        if index + 20 > data.len() {
            return Err(anyhow!("invalid tree object: missing 20-byte object hash"));
        }

        let raw_hash = &data[index..index + 20];
        let hash = hex::encode(raw_hash);

        index += 20;

        entries.push(TreeEntry { mode, name, hash });
    }

    Ok(entries)
}