//! Subresource Integrity (SRI) implementation

use std::fmt;

/// SRI hash algorithms
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SriAlgorithm {
    Sha256,
    Sha384,
    Sha512,
}

impl SriAlgorithm {
    pub fn as_str(&self) -> &'static str {
        match self {
            SriAlgorithm::Sha256 => "sha256",
            SriAlgorithm::Sha384 => "sha384",
            SriAlgorithm::Sha512 => "sha512",
        }
    }

    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "sha256" => Some(SriAlgorithm::Sha256),
            "sha384" => Some(SriAlgorithm::Sha384),
            "sha512" => Some(SriAlgorithm::Sha512),
            _ => None,
        }
    }

    /// Get hash length in bytes
    pub fn hash_length(&self) -> usize {
        match self {
            SriAlgorithm::Sha256 => 32,
            SriAlgorithm::Sha384 => 48,
            SriAlgorithm::Sha512 => 64,
        }
    }
}

/// SRI hash value
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SriHash {
    pub algorithm: SriAlgorithm,
    pub hash: String,
}

impl SriHash {
    /// Create a new SRI hash
    pub fn new(algorithm: SriAlgorithm, hash: &str) -> Self {
        Self {
            algorithm,
            hash: hash.to_string(),
        }
    }

    /// Parse from integrity attribute value (e.g., "sha384-abc123...")
    pub fn parse(value: &str) -> Option<Self> {
        let parts: Vec<&str> = value.splitn(2, '-').collect();
        if parts.len() != 2 {
            return None;
        }

        let algorithm = SriAlgorithm::from_str(parts[0])?;
        Some(Self {
            algorithm,
            hash: parts[1].to_string(),
        })
    }
}

impl fmt::Display for SriHash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}-{}", self.algorithm.as_str(), self.hash)
    }
}

/// Subresource Integrity checker
pub struct SubresourceIntegrity {
    expected_hashes: Vec<SriHash>,
}

impl SubresourceIntegrity {
    /// Create a new SRI checker
    pub fn new() -> Self {
        Self {
            expected_hashes: Vec::new(),
        }
    }

    /// Parse integrity attribute (can contain multiple hashes)
    pub fn parse(integrity: &str) -> Self {
        let hashes: Vec<SriHash> = integrity
            .split_whitespace()
            .filter_map(SriHash::parse)
            .collect();

        Self {
            expected_hashes: hashes,
        }
    }

    /// Add an expected hash
    pub fn add_hash(&mut self, hash: SriHash) {
        self.expected_hashes.push(hash);
    }

    /// Verify content against expected hashes
    /// Returns true if any hash matches (or if no hashes are specified)
    pub fn verify(&self, content: &[u8]) -> bool {
        if self.expected_hashes.is_empty() {
            return true;
        }

        for expected in &self.expected_hashes {
            if self.verify_hash(content, expected) {
                return true;
            }
        }

        false
    }

    /// Verify content against a specific hash
    fn verify_hash(&self, content: &[u8], expected: &SriHash) -> bool {
        use ring::digest;

        let algorithm = match expected.algorithm {
            SriAlgorithm::Sha256 => &digest::SHA256,
            SriAlgorithm::Sha384 => &digest::SHA384,
            SriAlgorithm::Sha512 => &digest::SHA512,
        };

        let actual = digest::digest(algorithm, content);
        let actual_base64 = base64_encode(actual.as_ref());

        actual_base64 == expected.hash
    }

    /// Get expected hashes
    pub fn expected_hashes(&self) -> &[SriHash] {
        &self.expected_hashes
    }
}

impl Default for SubresourceIntegrity {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple base64 encoding
fn base64_encode(data: &[u8]) -> String {
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    STANDARD.encode(data)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sri_algorithm_from_str() {
        assert_eq!(SriAlgorithm::from_str("sha256"), Some(SriAlgorithm::Sha256));
        assert_eq!(SriAlgorithm::from_str("sha384"), Some(SriAlgorithm::Sha384));
        assert_eq!(SriAlgorithm::from_str("sha512"), Some(SriAlgorithm::Sha512));
        assert_eq!(SriAlgorithm::from_str("md5"), None);
    }

    #[test]
    fn test_sri_hash_parse() {
        let hash = SriHash::parse("sha384-abc123").unwrap();
        assert_eq!(hash.algorithm, SriAlgorithm::Sha384);
        assert_eq!(hash.hash, "abc123");
    }

    #[test]
    fn test_sri_parse_multiple() {
        let sri = SubresourceIntegrity::parse("sha256-abc sha384-def");
        assert_eq!(sri.expected_hashes().len(), 2);
    }

    #[test]
    fn test_sri_empty_allows_all() {
        let sri = SubresourceIntegrity::new();
        assert!(sri.verify(b"any content"));
    }
}

