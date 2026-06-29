//! FNV-1a 64-bit hashing — per `spec/40-runtime/41-values.md §3b`
//! and `docs/design/content-addressing.md §2`.
//!
//! Constants from the FNV specification.

const FNV_OFFSET_BASIS: u64 = 0xcbf29ce484222325;
const FNV_PRIME: u64 = 0x100000001b3;

/// Compute the FNV-1a 64-bit hash of a byte slice.
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
    fn fnv1a_known_empty() {
        // FNV-1a of empty input
        let hash = fnv1a_64(b"");
        assert_eq!(hash, 0xcbf29ce484222325);
    }

    #[test]
    fn fnv1a_known_value() {
        // FNV-1a of "hello" — cross-checked against reference impl
        let hash = fnv1a_64(b"hello");
        // expected: FNV-1a 64-bit of "hello"
        assert_eq!(hash, 0xa430d84680aabd0b);
    }

    #[test]
    fn fnv1a_different_inputs_different_hashes() {
        let h1 = fnv1a_64(b"hello");
        let h2 = fnv1a_64(b"world");
        assert_ne!(h1, h2);
    }

    #[test]
    fn fnv1a_deterministic() {
        let h1 = fnv1a_64(b"test");
        let h2 = fnv1a_64(b"test");
        assert_eq!(h1, h2);
    }
}
