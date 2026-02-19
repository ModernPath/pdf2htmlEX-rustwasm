use sha2::{Digest, Sha256};

#[derive(Debug, Clone)]
pub struct ContentHasher;

impl ContentHasher {
    pub fn hash_bytes(data: &[u8]) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data);
        let result = hasher.finalize();
        hex::encode(result)
    }

    pub fn hash_string(s: &str) -> String {
        Self::hash_bytes(s.as_bytes())
    }

    pub fn generate_content_addressed_filename(content: &[u8], extension: &str) -> String {
        let hash = Self::hash_bytes(content);
        format!("{}.{}", hash, extension)
    }

    pub fn generate_short_hash(data: &[u8], length: usize) -> String {
        let full_hash = Self::hash_bytes(data);
        full_hash.chars().take(length).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_bytes() {
        let data = b"test data";
        let hash = ContentHasher::hash_bytes(data);
        assert_eq!(hash.len(), 64);
        assert!(hash.chars().all(|c| c.is_ascii_hexdigit()));
    }

    #[test]
    fn test_hash_string() {
        let hash1 = ContentHasher::hash_string("hello");
        let hash2 = ContentHasher::hash_string("hello");
        let hash3 = ContentHasher::hash_string("world");
        assert_eq!(hash1, hash2);
        assert_ne!(hash1, hash3);
    }

    #[test]
    fn test_generate_content_addressed_filename() {
        let data = b"test content";
        let filename = ContentHasher::generate_content_addressed_filename(data, "woff2");
        assert_eq!(filename.len(), 64 + 1 + 5); // 64 char hash + "." + "woff2"
        assert!(filename.ends_with(".woff2"));
    }

    #[test]
    fn test_short_hash() {
        let data = b"test";
        let short = ContentHasher::generate_short_hash(data, 8);
        assert_eq!(short.len(), 8);
    }
}
