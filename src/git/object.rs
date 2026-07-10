use anyhow::{anyhow, Result};
use flate2::read::ZlibDecoder;
use flate2::write::ZlibEncoder;
use flate2::Compression;
use sha1::{Digest, Sha1};
use std::fs;
use std::io::{Read, Write};
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ObjectType {
    Blob,
    Tree,
    Commit,
    Tag,
}

impl ObjectType {
    pub fn as_str(&self) -> &'static str {
        match self {
            ObjectType::Blob => "blob",
            ObjectType::Tree => "tree",
            ObjectType::Commit => "commit",
            ObjectType::Tag => "tag",
        }
    }

    pub fn from_str(value: &str) -> Result<Self> {
        match value {
            "blob" => Ok(ObjectType::Blob),
            "tree" => Ok(ObjectType::Tree),
            "commit" => Ok(ObjectType::Commit),
            "tag" => Ok(ObjectType::Tag),
            _ => Err(anyhow!("unknown object type: {}", value)),
        }
    }
}

#[derive(Debug, Clone)]
pub struct GitObject {
    pub object_type: ObjectType,
    pub content: Vec<u8>,
}

pub fn serialize_object(object_type: &ObjectType, content: &[u8]) -> Vec<u8> {
    let header = format!("{} {}\0", object_type.as_str(), content.len());

    let mut result = Vec::new();
    result.extend_from_slice(header.as_bytes());
    result.extend_from_slice(content);

    result
}

pub fn hash_object_data(data: &[u8]) -> String {
    let mut hasher = Sha1::new();
    hasher.update(data);
    let result = hasher.finalize();

    hex::encode(result)
}

pub fn write_object(objects_dir: &Path, object_type: ObjectType, content: &[u8]) -> Result<String> {
    let serialized = serialize_object(&object_type, content);
    let hash = hash_object_data(&serialized);

    let object_dir = objects_dir.join(&hash[..2]);
    let object_file = object_dir.join(&hash[2..]);

    fs::create_dir_all(&object_dir)?;

    let mut encoder = ZlibEncoder::new(Vec::new(), Compression::default());
    encoder.write_all(&serialized)?;
    let compressed = encoder.finish()?;

    fs::write(object_file, compressed)?;

    Ok(hash)
}

pub fn read_object(objects_dir: &Path, hash: &str) -> Result<GitObject> {
    if hash.len() < 3 {
        return Err(anyhow!("invalid object hash"));
    }

    let object_path = objects_dir.join(&hash[..2]).join(&hash[2..]);

    let compressed = fs::read(object_path)?;

    let mut decoder = ZlibDecoder::new(&compressed[..]);
    let mut data = Vec::new();
    decoder.read_to_end(&mut data)?;

    parse_object(&data)
}

pub fn parse_object(data: &[u8]) -> Result<GitObject> {
    let null_position = data
        .iter()
        .position(|byte| *byte == 0)
        .ok_or_else(|| anyhow!("invalid object: missing null byte"))?;

    let header = std::str::from_utf8(&data[..null_position])?;
    let content = data[null_position + 1..].to_vec();

    let mut header_parts = header.split(' ');

    let object_type_text = header_parts
        .next()
        .ok_or_else(|| anyhow!("invalid object header: missing type"))?;

    let size_text = header_parts
        .next()
        .ok_or_else(|| anyhow!("invalid object header: missing size"))?;

    let object_type = ObjectType::from_str(object_type_text)?;
    let declared_size: usize = size_text.parse()?;

    if declared_size != content.len() {
        return Err(anyhow!(
            "object size mismatch: header says {}, actual content is {}",
            declared_size,
            content.len()
        ));
    }

    Ok(GitObject {
        object_type,
        content,
    })
}