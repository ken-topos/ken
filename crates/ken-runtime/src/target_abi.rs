//! Target-ABI identity bound into native artifacts.
//!
//! PX2 binds and checks the identity at native-execution entry. Native host
//! syscalls remain unavailable; PX-B must retain this assertion when it makes
//! that lane executable. This is validated runtime evidence, not a Ken proof.

pub const NATIVE_TARGET_ABI_MANIFEST_HASH: [u8; 32] = ken_host::TARGET_ABI_MANIFEST_HASH;

pub fn assert_native_target_abi() -> Result<(), ken_host::TargetAbiIdentityError> {
    assert_native_target_abi_hash(NATIVE_TARGET_ABI_MANIFEST_HASH)
}

pub fn assert_native_target_abi_hash(
    artifact_hash: [u8; 32],
) -> Result<(), ken_host::TargetAbiIdentityError> {
    ken_host::assert_target_abi_identity(artifact_hash)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[cfg(target_os = "linux")]
    #[test]
    fn native_identity_accepts_match_and_rejects_mismatch() {
        assert_native_target_abi().expect("matching native ABI identity");
        let mut mismatch = NATIVE_TARGET_ABI_MANIFEST_HASH;
        mismatch[0] ^= 1;
        assert_eq!(
            assert_native_target_abi_hash(mismatch),
            Err(ken_host::TargetAbiIdentityError::HashMismatch)
        );
    }
}
