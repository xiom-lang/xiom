//! Centralized content hashing for incremental compilation.
//!
//! Provides SHA-256 hashing for source files, AST structures, and IR output.
//! All hashes are 64-char hex strings (full SHA-256). For cache key usage,
//! the full hash may be truncated to 16 chars for filesystem-friendliness.

use sha2::{Digest, Sha256};
use std::io::Read;

/// Compute the full SHA-256 hash of a file's contents.
/// Returns a 64-character lowercase hex string.
pub fn file_sha256(path: &std::path::Path) -> Result<String, std::io::Error> {
    let mut file = std::fs::File::open(path)?;
    let mut data = Vec::new();
    file.read_to_end(&mut data)?;
    Ok(hash_bytes(&data))
}

/// Compute SHA-256 hash of arbitrary bytes.
/// Returns a 64-character lowercase hex string.
pub fn hash_bytes(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    format!("{:x}", hasher.finalize())
}

/// Compute SHA-256 hash of a string.
/// Returns a 64-character lowercase hex string.
pub fn hash_str(data: &str) -> String {
    hash_bytes(data.as_bytes())
}

/// Truncate a full SHA-256 hex hash to a shorter cache key.
/// Default: first 16 characters (64 bits of entropy -- sufficient for cache keys).
pub fn short_hash(full_hash: &str) -> &str {
    &full_hash[..full_hash.len().min(16)]
}

/// A content fingerprint that combines a source hash with structural information.
/// Used to determine if recompilation is needed when dependencies change.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Fingerprint {
    /// SHA-256 of the source file content.
    pub source_hash: String,
    /// SHA-256 of the module's exported type signatures (for transitive invalidation).
    /// When this changes, all dependents must be recompiled even if their own source
    /// hasn't changed.
    pub signature_hash: String,
}

impl Fingerprint {
    /// Create a new fingerprint from source content and type signature text.
    pub fn new(source: &str, signatures: &str) -> Self {
        Fingerprint {
            source_hash: hash_str(source),
            signature_hash: hash_str(signatures),
        }
    }

    /// Create a fingerprint from just source content (no signatures yet).
    pub fn from_source(source: &str) -> Self {
        Fingerprint {
            source_hash: hash_str(source),
            signature_hash: String::new(),
        }
    }

    /// Check if the fingerprint has changed from a previous version.
    /// Returns true if the source changed OR the exported signatures changed.
    pub fn differs_from(&self, previous: &Fingerprint) -> bool {
        self.source_hash != previous.source_hash
            || (self.signature_hash != previous.signature_hash && !self.signature_hash.is_empty() && !previous.signature_hash.is_empty())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_known_vector() {
        // SHA-256("") = e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855
        assert_eq!(
            hash_bytes(b""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn short_hash_truncates() {
        let full = "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2";
        assert_eq!(short_hash(full), "a1b2c3d4e5f6a7b8");
    }

    #[test]
    fn fingerprint_differs_source_change() {
        let f1 = Fingerprint::from_source("hello");
        let f2 = Fingerprint::from_source("world");
        assert!(f1.differs_from(&f2));
    }

    #[test]
    fn fingerprint_same_source_no_diff() {
        let f1 = Fingerprint::from_source("same");
        let f2 = Fingerprint::from_source("same");
        assert!(!f1.differs_from(&f2));
    }

    #[test]
    fn fingerprint_differs_signature() {
        let mut f1 = Fingerprint::new("source", "sig_v1");
        let f2 = Fingerprint::new("source", "sig_v2");
        // Need non-empty signature hashes for comparison
        f1.signature_hash = hash_str("sig_v1");
        let mut f2 = f2;
        f2.signature_hash = hash_str("sig_v2");
        assert!(f1.differs_from(&f2));
    }
}
