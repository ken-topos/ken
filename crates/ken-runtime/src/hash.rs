//! FNV-1a 64-bit hashing — `spec/40-runtime/41-values.md §3b`,
//! `docs/design/content-addressing.md §2`.
//!
//! Non-cryptographic; used for in-process content addressing only.
//! Collisions resolved by memcmp of the full canonical encoding.

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

/// FNV-1a 64-bit hash of `bytes`.
#[inline]
pub fn fnv1a_64(bytes: &[u8]) -> u64 {
    let mut hash = FNV_OFFSET_BASIS;
    for &byte in bytes {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(FNV_PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv1a_empty() {
        assert_eq!(fnv1a_64(b""), FNV_OFFSET_BASIS);
    }

    #[test]
    fn fnv1a_hello() {
        // Cross-checked against reference FNV-1a-64
        assert_eq!(fnv1a_64(b"hello"), 0xa430d84680aabd0b);
    }

    #[test]
    fn fnv1a_distinct_inputs() {
        assert_ne!(fnv1a_64(b"hello"), fnv1a_64(b"world"));
    }

    #[test]
    fn fnv1a_deterministic() {
        assert_eq!(fnv1a_64(b"test"), fnv1a_64(b"test"));
    }
}
